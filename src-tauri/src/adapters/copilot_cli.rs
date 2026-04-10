use super::{
    Checkpoint, ConversationTurn, ResumeAction, SessionDetail, SessionSource, SessionSummary,
    SourceError,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

/// Source adapter for GitHub Copilot CLI sessions.
///
/// Data sources (supports multiple comma-separated paths):
/// - session_state_dirs (default: ~/.copilot/session-state/) — session folders with workspace.yaml, events.jsonl
/// - db_files (default: ~/.copilot/session-store.db) — SQLite DBs with sessions, turns, checkpoints, files
pub struct CopilotCliSource {
    session_state_dirs: Vec<PathBuf>,
    db_files: Vec<PathBuf>,
}

impl CopilotCliSource {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let copilot_dir = home.join(".copilot");
        Self {
            session_state_dirs: vec![copilot_dir.join("session-state")],
            db_files: vec![copilot_dir.join("session-store.db")],
        }
    }

    pub fn set_session_state_dirs(&mut self, paths: Vec<PathBuf>) {
        self.session_state_dirs = paths;
    }

    pub fn set_db_files(&mut self, paths: Vec<PathBuf>) {
        self.db_files = paths;
    }

    pub fn session_state_dirs(&self) -> &Vec<PathBuf> {
        &self.session_state_dirs
    }

    pub fn db_files(&self) -> &Vec<PathBuf> {
        &self.db_files
    }

    /// Open all available DB connections (read-only).
    fn open_dbs(&self) -> Vec<Connection> {
        self.db_files
            .iter()
            .filter(|p| p.exists())
            .filter_map(|p| {
                Connection::open_with_flags(p, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
            })
            .collect()
    }

    /// Open the first available DB (for detail/resume lookups).
    fn open_db(&self) -> Result<Connection, SourceError> {
        for path in &self.db_files {
            if path.exists() {
                return Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|e| SourceError::Warning(format!("Failed to open {}: {}", path.display(), e)));
            }
        }
        Err(SourceError::Fatal("No session-store.db found in any configured path".to_string()))
    }

    /// Find the session_state_dir that contains a given session ID.
    fn find_session_dir(&self, session_id: &str) -> Option<PathBuf> {
        for dir in &self.session_state_dirs {
            let candidate = dir.join(session_id);
            if candidate.exists() {
                return Some(dir.clone());
            }
        }
        None
    }

    /// Search turn messages for a query string, returning matching session IDs.
    pub fn search_turns(&self, query: &str) -> Vec<String> {
        let conns = self.open_dbs();
        let pattern = format!("%{}%", query);
        let mut ids = Vec::new();
        for conn in &conns {
            let mut stmt = match conn.prepare(
                "SELECT DISTINCT session_id FROM turns
                 WHERE user_message LIKE ?1 OR assistant_response LIKE ?1"
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let rows: Vec<String> = stmt
                .query_map([&pattern], |row| row.get::<_, String>(0))
                .into_iter()
                .flatten()
                .flatten()
                .collect();
            ids.extend(rows);
        }
        ids
    }

    /// Get the primary session_state_dir (first in list, used for storage_path fallback).
    fn primary_session_state_dir(&self) -> PathBuf {
        self.session_state_dirs.first().cloned().unwrap_or_default()
    }

    /// Parse workspace.yaml for a session to get summary and cwd.
    fn read_workspace_yaml(&self, session_id: &str) -> Option<(Option<String>, Option<String>)> {
        let state_dir = self.find_session_dir(session_id)?;
        let yaml_path = state_dir.join(session_id).join("workspace.yaml");
        let content = std::fs::read_to_string(&yaml_path).ok()?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;

        let summary = yaml
            .get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let cwd = yaml
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Some((summary, cwd))
    }

    /// Get folder size on disk for a session.
    fn session_size(&self, session_id: &str) -> Option<u64> {
        let state_dir = self.find_session_dir(session_id)?;
        let dir = state_dir.join(session_id);
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        Some(total)
    }

    /// Check if session has checkpoints directory with files.
    fn has_checkpoints(&self, session_id: &str) -> bool {
        if let Some(state_dir) = self.find_session_dir(session_id) {
            let checkpoint_dir = state_dir.join(session_id).join("checkpoints");
            checkpoint_dir.exists()
                && std::fs::read_dir(&checkpoint_dir)
                    .map(|mut entries| entries.next().is_some())
                    .unwrap_or(false)
        } else {
            false
        }
    }

    /// Look up session cwd from workspace.yaml first, then DB.
    fn get_session_cwd(&self, session_id: &str) -> Option<String> {
        // Try workspace.yaml first (may have more recent data)
        if let Some((_, cwd)) = self.read_workspace_yaml(session_id) {
            if cwd.is_some() {
                return cwd;
            }
        }
        // Fall back to DB
        if let Ok(conn) = self.open_db() {
            let mut stmt = conn
                .prepare("SELECT cwd FROM sessions WHERE id = ?1")
                .ok()?;
            stmt.query_row([session_id], |row| row.get::<_, Option<String>>(0))
                .ok()
                .flatten()
        } else {
            None
        }
    }
}

impl SessionSource for CopilotCliSource {
    fn name(&self) -> &str {
        "copilot-cli"
    }

    fn display_name(&self) -> &str {
        "Copilot CLI"
    }

    fn icon(&self) -> &str {
        "CLI"
    }

    fn is_available(&self) -> bool {
        self.db_files.iter().any(|p| p.exists())
    }

    fn scan(&self) -> Result<Vec<SessionSummary>, SourceError> {
        let conns = self.open_dbs();
        if conns.is_empty() {
            return Err(SourceError::Fatal("No session-store.db found in any configured path".to_string()));
        }

        let mut sessions = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for conn in &conns {

        // Query sessions with turn count and first message
        let stmt = conn
            .prepare(
                "SELECT s.id, s.cwd, s.branch, s.summary, s.created_at, s.updated_at,
                        (SELECT COUNT(*) FROM turns t WHERE t.session_id = s.id) as turn_count,
                        (SELECT substr(t2.user_message, 1, 200) FROM turns t2
                         WHERE t2.session_id = s.id AND t2.turn_index = 0) as first_msg
                 FROM sessions s
                 ORDER BY s.updated_at DESC",
            );

        let mut stmt = match stmt {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[copilot-cli] Warning: SQL prepare failed for a DB: {}", e);
                continue;
            }
        };

        let rows = match stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,           // id
                    row.get::<_, Option<String>>(1)?,    // cwd
                    row.get::<_, Option<String>>(2)?,    // branch
                    row.get::<_, Option<String>>(3)?,    // summary
                    row.get::<_, Option<String>>(4)?,    // created_at
                    row.get::<_, Option<String>>(5)?,    // updated_at
                    row.get::<_, u32>(6)?,               // turn_count
                    row.get::<_, Option<String>>(7)?,    // first_msg
                ))
            }) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[copilot-cli] Warning: SQL query failed for a DB: {}", e);
                continue;
            }
        };

        for row in rows {
            match row {
                Ok((id, cwd, branch, summary, created_at, updated_at, turn_count, first_msg)) => {
                    if !seen_ids.insert(id.clone()) {
                        continue; // skip duplicate session IDs from other DBs
                    }
                    // Enrich with workspace.yaml data (may have more recent summary/cwd)
                    let (yaml_summary, yaml_cwd) =
                        self.read_workspace_yaml(&id).unwrap_or((None, None));

                    let title = yaml_summary.or(summary);
                    let effective_cwd = yaml_cwd.or(cwd);
                    let found_dir = self.find_session_dir(&id);
                    let exists = found_dir.is_some();
                    let storage_dir = found_dir.unwrap_or_else(|| self.primary_session_state_dir());

                    sessions.push(SessionSummary {
                        id: id.clone(),
                        source: self.name().to_string(),
                        title,
                        turn_count,
                        cwd: effective_cwd,
                        branch,
                        created_at,
                        updated_at: updated_at.clone(),
                        first_message: first_msg,
                        size_bytes: self.session_size(&id),
                        has_checkpoints: self.has_checkpoints(&id),
                        exists_on_disk: exists,
                        storage_path: Some(storage_dir.join(&id).to_string_lossy().into_owned()),
                        status: super::compute_status(&updated_at),
                        extra: HashMap::new(),
                    });
                }
                Err(e) => {
                    eprintln!("[copilot-cli] Warning: skipping row: {}", e);
                }
            }
        }

        } // end for conn in &conns

        Ok(sessions)
    }

    fn load_detail(&self, id: &str) -> Result<SessionDetail, SourceError> {
        let conn = self.open_db()?;

        // Load turns
        let mut turns = Vec::new();
        let mut stmt = conn
            .prepare(
                "SELECT turn_index, user_message, assistant_response, timestamp
                 FROM turns WHERE session_id = ?1 ORDER BY turn_index",
            )
            .map_err(|e| SourceError::Warning(format!("SQL prepare failed: {}", e)))?;

        let rows = stmt
            .query_map([id], |row| {
                Ok(ConversationTurn {
                    turn_index: row.get(0)?,
                    user_message: row.get(1)?,
                    assistant_response: row.get(2)?,
                    timestamp: row.get(3)?,
                })
            })
            .map_err(|e| SourceError::Warning(format!("SQL query failed: {}", e)))?;

        for row in rows.flatten() {
            turns.push(row);
        }

        // Load checkpoints
        let mut checkpoints = Vec::new();
        let mut ckpt_stmt = conn
            .prepare(
                "SELECT checkpoint_number, title, overview
                 FROM checkpoints WHERE session_id = ?1 ORDER BY checkpoint_number",
            )
            .map_err(|e| SourceError::Warning(format!("SQL prepare failed: {}", e)))?;

        let ckpt_rows = ckpt_stmt
            .query_map([id], |row| {
                Ok(Checkpoint {
                    number: row.get(0)?,
                    title: row.get(1)?,
                    overview: row.get(2)?,
                    after_turn: None,
                })
            })
            .map_err(|e| SourceError::Warning(format!("SQL query failed: {}", e)))?;

        for row in ckpt_rows.flatten() {
            checkpoints.push(row);
        }

        // Load files touched
        let mut files = Vec::new();
        let mut files_stmt = conn
            .prepare("SELECT file_path FROM session_files WHERE session_id = ?1")
            .map_err(|e| SourceError::Warning(format!("SQL prepare failed: {}", e)))?;

        let file_rows = files_stmt
            .query_map([id], |row| row.get::<_, String>(0))
            .map_err(|e| SourceError::Warning(format!("SQL query failed: {}", e)))?;

        for row in file_rows.flatten() {
            files.push(row);
        }

        // Build summary from scan data
        let scan_result = self.scan()?;
        let summary = scan_result
            .into_iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| SessionSummary {
                id: id.to_string(),
                source: self.name().to_string(),
                title: None,
                turn_count: turns.len() as u32,
                cwd: None,
                branch: None,
                created_at: None,
                updated_at: None,
                first_message: turns.first().and_then(|t| t.user_message.clone()),
                size_bytes: None,
                has_checkpoints: !checkpoints.is_empty(),
                exists_on_disk: true,
                storage_path: Some(self.find_session_dir(id).unwrap_or_else(|| self.primary_session_state_dir()).join(id).to_string_lossy().into_owned()),
                status: None,
                extra: HashMap::new(),
            });

        Ok(SessionDetail {
            summary,
            turns,
            checkpoints,
            files_touched: files,
        })
    }

    fn rename(&self, id: &str, new_name: &str) -> Result<(), SourceError> {
        let state_dir = self.find_session_dir(id).ok_or_else(|| {
            SourceError::Warning(format!("Session folder not found for {}", id))
        })?;
        let yaml_path = state_dir.join(id).join("workspace.yaml");

        if !yaml_path.exists() {
            return Err(SourceError::Warning(format!(
                "workspace.yaml not found for session {}",
                id
            )));
        }

        let content = std::fs::read_to_string(&yaml_path)
            .map_err(|e| SourceError::Warning(format!("Failed to read workspace.yaml: {}", e)))?;

        // Parse, update summary, write back
        let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| SourceError::Warning(format!("Failed to parse workspace.yaml: {}", e)))?;

        yaml["summary"] = serde_yaml::Value::String(new_name.to_string());

        let new_content = serde_yaml::to_string(&yaml)
            .map_err(|e| SourceError::Warning(format!("Failed to serialize YAML: {}", e)))?;

        std::fs::write(&yaml_path, new_content)
            .map_err(|e| SourceError::Warning(format!("Failed to write workspace.yaml: {}", e)))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), SourceError> {
        let state_dir = self.find_session_dir(id).ok_or_else(|| {
            SourceError::Warning(format!("Session folder not found: {}", id))
        })?;
        let session_dir = state_dir.join(id);

        std::fs::remove_dir_all(&session_dir)
            .map_err(|e| SourceError::Warning(format!("Failed to delete session folder: {}", e)))?;

        Ok(())
    }

    fn resume(&self, id: &str) -> Result<ResumeAction, SourceError> {
        let cwd = self.get_session_cwd(id);
        let resume_cmd = format!("copilot --resume={}", id);

        #[cfg(windows)]
        {
            Ok(ResumeAction::SpawnTerminal {
                command: "pwsh".to_string(),
                args: vec![
                    "-NoExit".to_string(),
                    "-Command".to_string(),
                    resume_cmd,
                ],
                cwd,
            })
        }
        #[cfg(not(windows))]
        {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
            Ok(ResumeAction::SpawnTerminal {
                command: shell,
                args: vec!["-c".to_string(), resume_cmd],
                cwd,
            })
        }
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for dir in &self.session_state_dirs {
            if dir.exists() {
                paths.push(dir.clone());
            }
        }
        for db in &self.db_files {
            if db.exists() {
                paths.push(db.clone());
            }
        }
        paths
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
