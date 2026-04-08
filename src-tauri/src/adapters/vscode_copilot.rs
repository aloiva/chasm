use super::{
    ConversationTurn, ResumeAction, SessionDetail, SessionSource, SessionSummary, SourceError,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Source adapter for VS Code Copilot Chat sessions.
///
/// Data lives in VS Code's workspace storage:
/// - %APPDATA%/Code/User/workspaceStorage/{hash}/state.vscdb (SQLite)
/// - Key: chat.ChatSessionStore.index → session index JSON
/// - Key: memento/interactive-session → user prompt history
/// - %APPDATA%/Code/User/workspaceStorage/{hash}/workspace.json → folder mapping
pub struct VsCodeCopilotSource {
    workspace_storage_dir: PathBuf,
}

impl VsCodeCopilotSource {
    pub fn new() -> Self {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        Self {
            workspace_storage_dir: PathBuf::from(appdata)
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
    fn read_chat_index(db_path: &Path) -> Result<Vec<VsCodeSession>, SourceError> {
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

            sessions.push(VsCodeSession {
                session_id,
                title,
                last_message_date: last_date,
                is_empty,
            });
        }

        Ok(sessions)
    }
}

#[derive(Debug)]
struct VsCodeSession {
    session_id: String,
    title: Option<String>,
    last_message_date: Option<String>,
    is_empty: bool,
}

/// Decode VS Code folder URI to a Windows path.
fn decode_vscode_folder_uri(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file:///")?;
    // URL decode: %3A → :, %20 → space, etc.
    let decoded = urlish_decode(path);
    // Convert forward slashes to backslashes for Windows
    Some(decoded.replace('/', "\\"))
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
            let sessions = match Self::read_chat_index(&db_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for session in sessions {
                let turn_count = if session.is_empty { 0 } else { 1 }; // We can't know exact count without deep parsing

                let mut extra = HashMap::new();
                extra.insert("workspace_hash".to_string(), ws_hash.clone());
                if let Some(ref f) = folder {
                    extra.insert("workspace_folder".to_string(), f.clone());
                }

                all_sessions.push(SessionSummary {
                    id: session.session_id,
                    source: self.name().to_string(),
                    title: session.title,
                    turn_count,
                    cwd: folder.clone(),
                    branch: None,
                    created_at: session.last_message_date.clone(),
                    updated_at: session.last_message_date,
                    first_message: None, // Would need deeper DB parsing
                    size_bytes: None,
                    has_checkpoints: false,
                    exists_on_disk: true,
                    extra,
                });
            }
        }

        Ok(all_sessions)
    }

    fn load_detail(&self, id: &str) -> Result<SessionDetail, SourceError> {
        // For VS Code, we need to find which workspace contains this session
        // and extract the conversation data from the state.vscdb
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

        // Try to load conversation from state.vscdb
        let db_path = self.workspace_storage_dir.join(&ws_hash).join("state.vscdb");
        let mut turns = Vec::new();

        if db_path.exists() {
            if let Ok(conn) =
                Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            {
                // VS Code stores full session data under session-specific keys
                // The exact key format may vary, try common patterns
                let session_key = format!("interactive.sessions.{}", id);
                if let Ok(mut stmt) =
                    conn.prepare("SELECT value FROM ItemTable WHERE key = ?1")
                {
                    if let Ok(json_str) = stmt.query_row([&session_key], |row| row.get::<_, String>(0)) {
                        if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            // Parse turns from session data
                            if let Some(requests) = session_data.get("requests").and_then(|v| v.as_array()) {
                                for (i, req) in requests.iter().enumerate() {
                                    let user_msg = req.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    let response = req.get("response").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    turns.push(ConversationTurn {
                                        turn_index: i as u32,
                                        user_message: user_msg,
                                        assistant_response: response,
                                        timestamp: None,
                                    });
                                }
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
            Ok(ResumeAction::OpenApplication {
                command: "code".to_string(),
                args: vec!["--folder-uri".to_string(), format!("file:///{}", folder.replace('\\', "/"))],
            })
        } else {
            Err(SourceError::Warning(
                "Cannot determine workspace folder for this session".to_string(),
            ))
        }
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        vec![self.workspace_storage_dir.clone()]
    }
}
