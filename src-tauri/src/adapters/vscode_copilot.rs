use super::{
    ConversationTurn, ResumeAction, SessionDetail, SessionSource, SessionSummary, SourceError,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Source adapter for VS Code Copilot Chat sessions.
///
/// Data lives in VS Code's workspace storage:
/// - Windows: %APPDATA%/Code/User/workspaceStorage/
/// - macOS:   ~/Library/Application Support/Code/User/workspaceStorage/
/// - Linux:   ~/.config/Code/User/workspaceStorage/
pub struct VsCodeCopilotSource {
    workspace_storage_dir: PathBuf,
}

impl VsCodeCopilotSource {
    pub fn new() -> Self {
        let base = dirs::config_dir().unwrap_or_default();
        Self {
            workspace_storage_dir: base
                .join("Code")
                .join("User")
                .join("workspaceStorage"),
        }
    }

    /// Read workspace.json to find which folder this workspace maps to.
    fn read_workspace_folder(ws_dir: &Path) -> Option<String> {
        let ws_json_path = ws_dir.join("workspace.json");
        let content = std::fs::read_to_string(&ws_json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let folder = json.get("folder")?.as_str()?;

        // Decode URI-encoded path (e.g., file:///q%3A/src/repo → Q:\src\repo)
        decode_vscode_folder_uri(folder)
    }

    /// Read chat session index from a state.vscdb file.
    fn read_chat_index(db_path: &Path, ws_dir: &Path) -> Result<Vec<VsCodeSession>, SourceError> {
        let conn = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                SourceError::Warning(format!(
                    "Failed to open {}: {}",
                    db_path.display(),
                    e
                ))
            })?;

        let mut stmt = conn
            .prepare("SELECT value FROM ItemTable WHERE key = 'chat.ChatSessionStore.index'")
            .map_err(|e| SourceError::Warning(format!("SQL prepare: {}", e)))?;

        let json_str: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .ok();

        let json_str = match json_str {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(Vec::new()),
        };

        let index: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| SourceError::Warning(format!("JSON parse error: {}", e)))?;

        let entries = match index.get("entries") {
            Some(serde_json::Value::Object(map)) => map,
            _ => return Ok(Vec::new()),
        };

        // Collect session IDs first, then batch-read turn counts
        let mut sessions = Vec::new();
        for (_key, entry) in entries {
            let session_id = entry
                .get("sessionId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            if session_id.is_empty() {
                continue;
            }

            let title = entry
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let last_date = entry
                .get("lastMessageDate")
                .and_then(|v| v.as_i64())
                .map(|ms| {
                    chrono::DateTime::from_timestamp_millis(ms)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                });

            let is_empty = entry
                .get("isEmpty")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Try to get actual turn count from session data
            let turn_count = Self::count_session_turns(&conn, &session_id, ws_dir)
                .unwrap_or(if is_empty { 0 } else { 1 });

            sessions.push(VsCodeSession {
                session_id,
                title,
                last_message_date: last_date,
                is_empty,
                turn_count,
            });
        }

        Ok(sessions)
    }

    /// Count the number of turns in a VS Code chat session.
    /// Checks chatSessions/ files first (new format), then falls back to DB key (legacy).
    fn count_session_turns(conn: &Connection, session_id: &str, ws_dir: &Path) -> Option<u32> {
        // Try file-based storage first (chatSessions/<id>.jsonl or .json)
        let chat_dir = ws_dir.join("chatSessions");
        if chat_dir.is_dir() {
            let jsonl = chat_dir.join(format!("{}.jsonl", session_id));
            if jsonl.exists() {
                return Self::count_turns_jsonl(&jsonl);
            }
            let json = chat_dir.join(format!("{}.json", session_id));
            if json.exists() {
                return Self::count_turns_json(&json);
            }
        }

        // Legacy: read from state.vscdb
        let session_key = format!("interactive.sessions.{}", session_id);
        let mut stmt = conn
            .prepare("SELECT value FROM ItemTable WHERE key = ?1")
            .ok()?;
        let json_str: String = stmt.query_row([&session_key], |row| row.get(0)).ok()?;
        let data: serde_json::Value = serde_json::from_str(&json_str).ok()?;
        let requests = data.get("requests")?.as_array()?;
        Some(requests.len() as u32)
    }

    /// Count turns from a .json chat session file.
    fn count_turns_json(path: &Path) -> Option<u32> {
        let content = std::fs::read_to_string(path).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;
        let requests = data.get("requests")?.as_array()?;
        Some(requests.len() as u32)
    }

    /// Count turns from a .jsonl chat session file (count kind:2 patches that append to "requests").
    fn count_turns_jsonl(path: &Path) -> Option<u32> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut count = 0u32;
        for line in content.lines() {
            if let Ok(patch) = serde_json::from_str::<serde_json::Value>(line) {
                if patch.get("kind")?.as_u64()? == 2 {
                    // kind:2 = append. Check if appending to "requests"
                    if let Some(keys) = patch.get("k").and_then(|v| v.as_array()) {
                        if keys.first().and_then(|k| k.as_str()) == Some("requests") {
                            // v is an array of appended requests
                            if let Some(arr) = patch.get("v").and_then(|v| v.as_array()) {
                                count += arr.len() as u32;
                            }
                        }
                    }
                }
            }
        }
        // If we found kind:2 patches, use that count
        // If zero, check initial snapshot for pre-populated requests
        if count == 0 {
            for line in content.lines() {
                if let Ok(patch) = serde_json::from_str::<serde_json::Value>(line) {
                    if patch.get("kind").and_then(|v| v.as_u64()) == Some(0) {
                        if let Some(reqs) = patch.get("v")
                            .and_then(|v| v.get("requests"))
                            .and_then(|v| v.as_array())
                        {
                            return Some(reqs.len() as u32);
                        }
                    }
                }
                break; // kind:0 is always line 0
            }
        }
        Some(count)
    }

    /// Extract the first user message from a chat session file.
    fn first_message_from_file(ws_dir: &Path, session_id: &str) -> Option<String> {
        let chat_dir = ws_dir.join("chatSessions");
        if !chat_dir.is_dir() {
            return None;
        }

        let jsonl = chat_dir.join(format!("{}.jsonl", session_id));
        if jsonl.exists() {
            return Self::first_message_jsonl(&jsonl);
        }
        let json = chat_dir.join(format!("{}.json", session_id));
        if json.exists() {
            return Self::first_message_json(&json);
        }
        None
    }

    /// Extract first user message from a .json session file.
    fn first_message_json(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;
        let requests = data.get("requests")?.as_array()?;
        let first = requests.first()?;
        let text = first.get("message")?.get("text")?.as_str()?;
        Some(truncate_message(text, 200))
    }

    /// Extract first user message from a .jsonl session file.
    fn first_message_jsonl(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        // First try kind:2 patches (requests appended after snapshot)
        for line in content.lines() {
            if let Ok(patch) = serde_json::from_str::<serde_json::Value>(line) {
                if patch.get("kind").and_then(|v| v.as_u64()) == Some(2) {
                    if let Some(keys) = patch.get("k").and_then(|v| v.as_array()) {
                        if keys.first().and_then(|k| k.as_str()) == Some("requests") {
                            if let Some(arr) = patch.get("v").and_then(|v| v.as_array()) {
                                if let Some(first) = arr.first() {
                                    if let Some(text) = first.get("message")
                                        .and_then(|m| m.get("text"))
                                        .and_then(|t| t.as_str())
                                    {
                                        return Some(truncate_message(text, 200));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Fallback: check initial snapshot requests
        if let Some(first_line) = content.lines().next() {
            if let Ok(patch) = serde_json::from_str::<serde_json::Value>(first_line) {
                if patch.get("kind").and_then(|v| v.as_u64()) == Some(0) {
                    if let Some(reqs) = patch.get("v")
                        .and_then(|v| v.get("requests"))
                        .and_then(|v| v.as_array())
                    {
                        if let Some(first) = reqs.first() {
                            if let Some(text) = first.get("message")
                                .and_then(|m| m.get("text"))
                                .and_then(|t| t.as_str())
                            {
                                return Some(truncate_message(text, 200));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Load full conversation turns from a chat session file.
    fn load_turns_from_file(ws_dir: &Path, session_id: &str) -> Option<Vec<ConversationTurn>> {
        let chat_dir = ws_dir.join("chatSessions");
        if !chat_dir.is_dir() {
            return None;
        }

        let jsonl = chat_dir.join(format!("{}.jsonl", session_id));
        if jsonl.exists() {
            return Self::load_turns_jsonl(&jsonl);
        }
        let json = chat_dir.join(format!("{}.json", session_id));
        if json.exists() {
            return Self::load_turns_json(&json);
        }
        None
    }

    /// Parse conversation turns from a .json session file.
    fn load_turns_json(path: &Path) -> Option<Vec<ConversationTurn>> {
        let content = std::fs::read_to_string(path).ok()?;
        let data: serde_json::Value = serde_json::from_str(&content).ok()?;
        let requests = data.get("requests")?.as_array()?;
        Some(requests_to_turns(requests))
    }

    /// Parse conversation turns from a .jsonl session file by replaying patches.
    fn load_turns_jsonl(path: &Path) -> Option<Vec<ConversationTurn>> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut all_requests: Vec<serde_json::Value> = Vec::new();

        for line in content.lines() {
            if let Ok(patch) = serde_json::from_str::<serde_json::Value>(line) {
                match patch.get("kind").and_then(|v| v.as_u64()) {
                    Some(0) => {
                        // Initial snapshot — seed requests from v.requests
                        if let Some(reqs) = patch.get("v")
                            .and_then(|v| v.get("requests"))
                            .and_then(|v| v.as_array())
                        {
                            all_requests.extend(reqs.iter().cloned());
                        }
                    }
                    Some(2) => {
                        // Append operation
                        if let Some(keys) = patch.get("k").and_then(|v| v.as_array()) {
                            if keys.first().and_then(|k| k.as_str()) == Some("requests") {
                                if let Some(arr) = patch.get("v").and_then(|v| v.as_array()) {
                                    all_requests.extend(arr.iter().cloned());
                                }
                            }
                        }
                    }
                    _ => {} // kind:1 (set) — skip for now, not needed for turn extraction
                }
            }
        }

        Some(requests_to_turns(&all_requests))
    }
}

#[derive(Debug)]
struct VsCodeSession {
    session_id: String,
    title: Option<String>,
    last_message_date: Option<String>,
    is_empty: bool,
    turn_count: u32,
}

/// Decode VS Code folder URI to a native path.
fn decode_vscode_folder_uri(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file:///")?;
    let decoded = urlish_decode(path);
    #[cfg(windows)]
    {
        Some(decoded.replace('/', "\\"))
    }
    #[cfg(not(windows))]
    {
        // On Unix, file URIs use file:///path so the path starts after the third slash
        Some(format!("/{}", decoded))
    }
}

/// Simple URL decoding (handles %XX sequences).
fn urlish_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Truncate a message to the given max length, appending "…" if truncated.
fn truncate_message(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// Convert an array of VS Code request objects into ConversationTurns.
fn requests_to_turns(requests: &[serde_json::Value]) -> Vec<ConversationTurn> {
    requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let user_msg = req
                .get("message")
                .and_then(|m| m.get("text"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());

            // Response is an array of parts; collect text values
            let response = req
                .get("response")
                .and_then(|r| r.as_array())
                .map(|parts| {
                    parts
                        .iter()
                        .filter_map(|part| {
                            // Parts with a string "value" key are text content
                            part.get("value")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect::<Vec<_>>()
                        .join("")
                })
                .filter(|s| !s.is_empty());

            let timestamp = req
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .and_then(|ms| chrono::DateTime::from_timestamp_millis(ms))
                .map(|dt| dt.to_rfc3339());

            ConversationTurn {
                turn_index: i as u32,
                user_message: user_msg,
                assistant_response: response,
                timestamp,
            }
        })
        .collect()
}

impl SessionSource for VsCodeCopilotSource {
    fn name(&self) -> &str {
        "vscode-copilot"
    }

    fn display_name(&self) -> &str {
        "VS Code Copilot"
    }

    fn icon(&self) -> &str {
        "VSC"
    }

    fn is_available(&self) -> bool {
        self.workspace_storage_dir.exists()
    }

    fn scan(&self) -> Result<Vec<SessionSummary>, SourceError> {
        if !self.workspace_storage_dir.exists() {
            return Err(SourceError::Fatal(format!(
                "VS Code workspace storage not found at {}",
                self.workspace_storage_dir.display()
            )));
        }

        let mut all_sessions = Vec::new();

        let entries = std::fs::read_dir(&self.workspace_storage_dir).map_err(|e| {
            SourceError::Warning(format!("Failed to read workspace storage dir: {}", e))
        })?;

        for entry in entries.flatten() {
            let ws_dir = entry.path();
            if !ws_dir.is_dir() {
                continue;
            }

            let db_path = ws_dir.join("state.vscdb");
            if !db_path.exists() {
                continue;
            }

            // Read workspace folder mapping
            let folder = Self::read_workspace_folder(&ws_dir);
            let ws_hash = ws_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Read chat sessions from this workspace (skip on any error)
            let sessions = match Self::read_chat_index(&db_path, &ws_dir) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for session in sessions {
                let mut extra = HashMap::new();
                extra.insert("workspace_hash".to_string(), ws_hash.clone());
                if let Some(ref f) = folder {
                    extra.insert("workspace_folder".to_string(), f.clone());
                }

                let first_message = Self::first_message_from_file(&ws_dir, &session.session_id);

                all_sessions.push(SessionSummary {
                    id: session.session_id,
                    source: self.name().to_string(),
                    title: session.title,
                    turn_count: session.turn_count,
                    cwd: folder.clone(),
                    branch: None,
                    created_at: session.last_message_date.clone(),
                    updated_at: session.last_message_date.clone(),
                    first_message,
                    size_bytes: None,
                    has_checkpoints: false,
                    exists_on_disk: true,
                    storage_path: Some(db_path.parent().unwrap_or(db_path.as_path()).to_string_lossy().into_owned()),
                    status: super::compute_status(&session.last_message_date),
                    extra,
                });
            }
        }

        Ok(all_sessions)
    }

    fn load_detail(&self, id: &str) -> Result<SessionDetail, SourceError> {
        let scan = self.scan()?;
        let summary = scan
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| SourceError::Warning(format!("Session {} not found", id)))?;

        let ws_hash = summary
            .extra
            .get("workspace_hash")
            .cloned()
            .unwrap_or_default();

        let ws_dir = self.workspace_storage_dir.join(&ws_hash);

        // Try file-based loading first (chatSessions/ directory)
        if let Some(turns) = Self::load_turns_from_file(&ws_dir, id) {
            return Ok(SessionDetail {
                summary,
                turns,
                checkpoints: Vec::new(),
                files_touched: Vec::new(),
            });
        }

        // Fallback: try legacy DB key
        let db_path = ws_dir.join("state.vscdb");
        let mut turns = Vec::new();

        if db_path.exists() {
            if let Ok(conn) =
                Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            {
                let session_key = format!("interactive.sessions.{}", id);
                if let Ok(mut stmt) =
                    conn.prepare("SELECT value FROM ItemTable WHERE key = ?1")
                {
                    if let Ok(json_str) = stmt.query_row([&session_key], |row| row.get::<_, String>(0)) {
                        if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            if let Some(requests) = session_data.get("requests").and_then(|v| v.as_array()) {
                                turns = requests_to_turns(requests);
                            }
                        }
                    }
                }
            }
        }

        Ok(SessionDetail {
            summary,
            turns,
            checkpoints: Vec::new(),
            files_touched: Vec::new(),
        })
    }

    fn rename(&self, _id: &str, _name: &str) -> Result<(), SourceError> {
        // VS Code state.vscdb is an internal database — modifying it risks corruption.
        // Rename is handled via app-local metadata instead.
        Err(SourceError::Warning(
            "Renaming VS Code sessions is handled via app-local metadata only".to_string(),
        ))
    }

    fn delete(&self, _id: &str) -> Result<(), SourceError> {
        Err(SourceError::Warning(
            "Deleting VS Code sessions is not supported — state.vscdb is an internal VS Code database".to_string(),
        ))
    }

    fn resume(&self, id: &str) -> Result<ResumeAction, SourceError> {
        // Find the workspace folder for this session
        let scan = self.scan()?;
        let session = scan
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| SourceError::Warning(format!("Session {} not found", id)))?;

        if let Some(folder) = session.extra.get("workspace_folder") {
            // Open VS Code to the workspace folder.
            // VS Code doesn't expose a CLI/URI to open a specific Copilot Chat session,
            // so the best we can do is open the workspace where the chat occurred.
            Ok(ResumeAction::OpenApplication {
                command: "code".to_string(),
                args: vec![folder.clone()],
            })
        } else {
            Err(SourceError::Warning(
                "Cannot determine workspace folder for this session".to_string(),
            ))
        }
    }

    fn search_turns(&self, query: &str) -> Vec<String> {
        if !self.workspace_storage_dir.exists() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        let mut matching_ids = Vec::new();

        let entries = match std::fs::read_dir(&self.workspace_storage_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        for entry in entries.flatten() {
            let ws_dir = entry.path();
            let chat_dir = ws_dir.join("chatSessions");
            if !chat_dir.is_dir() {
                continue;
            }

            if let Ok(files) = std::fs::read_dir(&chat_dir) {
                for file in files.flatten() {
                    let path = file.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext != "json" && ext != "jsonl" {
                        continue;
                    }

                    // Cheap: read file and do case-insensitive contains check
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.to_lowercase().contains(&query_lower) {
                            // Extract session ID from filename (strip extension)
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                matching_ids.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }

        matching_ids
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        vec![self.workspace_storage_dir.clone()]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
