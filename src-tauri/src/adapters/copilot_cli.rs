use super::{
    Checkpoint, ConversationTurn, ResumeAction, SessionDetail, SessionSource, SessionSummary,
    SourceError,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

/// Source adapter for GitHub Copilot CLI sessions.
///
/// Data sources:
/// - ~/.copilot/session-store.db (SQLite) — sessions, turns, checkpoints, files
/// - ~/.copilot/session-state/{id}/workspace.yaml — summary, cwd, timestamps
/// - ~/.copilot/session-state/{id}/events.jsonl — full event stream (lazy-loaded)
pub struct CopilotCliSource {
    copilot_dir: PathBuf,
}

impl CopilotCliSource {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            copilot_dir: home.join(".copilot"),
        }
    }

    pub fn set_copilot_dir(&mut self, path: PathBuf) {
        self.copilot_dir = path;
    }

    pub fn copilot_dir(&self) -> &PathBuf {
        &self.copilot_dir
    }

    fn db_path(&self) -> PathBuf {
        self.copilot_dir.join("session-store.db")
    }

    fn session_state_dir(&self) -> PathBuf {
        self.copilot_dir.join("session-state")
    }

    fn open_db(&self) -> Result<Connection, SourceError> {
        let path = self.db_path();
        if !path.exists() {
            return Err(SourceError::Fatal(format!(
                "session-store.db not found at {}",
                path.display()
            )));
        }
        // Open read-only to avoid locking issues with running Copilot
        Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| SourceError::Warning(format!("Failed to open session-store.db: {}", e)))
    }

    /// Parse workspace.yaml for a session to get summary and cwd.
    fn read_workspace_yaml(&self, session_id: &str) -> Option<(Option<String>, Option<String>)> {
        let yaml_path = self
            .session_state_dir()
            .join(session_id)
            .join("workspace.yaml");
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
        let dir = self.session_state_dir().join(session_id);
        if !dir.exists() {
            return None;
        }
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
        let checkpoint_dir = self.session_state_dir().join(session_id).join("checkpoints");
        checkpoint_dir.exists()
            && std::fs::read_dir(&checkpoint_dir)
                .map(|mut entries| entries.next().is_some())
                .unwrap_or(false)
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
        self.db_path().exists()
    }

    fn scan(&self) -> Result<Vec<SessionSummary>, SourceError> {
        let conn = self.open_db()?;
        let mut sessions = Vec::new();

        // Query sessions with turn count and first message
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.cwd, s.branch, s.summary, s.created_at, s.updated_at,
                        (SELECT COUNT(*) FROM turns t WHERE t.session_id = s.id) as turn_count,
                        (SELECT substr(t2.user_message, 1, 200) FROM turns t2
                         WHERE t2.session_id = s.id AND t2.turn_index = 0) as first_msg
                 FROM sessions s
                 ORDER BY s.updated_at DESC",
            )
            .map_err(|e| SourceError::Warning(format!("SQL prepare failed: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| SourceError::Warning(format!("SQL query failed: {}", e)))?;

        for row in rows {
            match row {
                Ok((id, cwd, branch, summary, created_at, updated_at, turn_count, first_msg)) => {
                    // Enrich with workspace.yaml data (may have more recent summary/cwd)
                    let (yaml_summary, yaml_cwd) =
                        self.read_workspace_yaml(&id).unwrap_or((None, None));

                    let title = yaml_summary.or(summary);
                    let effective_cwd = yaml_cwd.or(cwd);
                    let exists = self.session_state_dir().join(&id).exists();

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
                        storage_path: Some(self.session_state_dir().join(&id).to_string_lossy().into_owned()),
                        status: super::compute_status(&updated_at),
                        extra: HashMap::new(),
                    });
                }
                Err(e) => {
                    // Skip individual broken rows, don't fail entire scan
                    eprintln!("[copilot-cli] Warning: skipping row: {}", e);
                }
            }
        }

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
                storage_path: Some(self.session_state_dir().join(id).to_string_lossy().into_owned()),
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
        let yaml_path = self
            .session_state_dir()
            .join(id)
            .join("workspace.yaml");

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
        let session_dir = self.session_state_dir().join(id);
        if !session_dir.exists() {
            return Err(SourceError::Warning(format!(
                "Session folder not found: {}",
                id
            )));
        }

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
        // Watch session-state directory (folder changes, workspace.yaml, deletions)
        // AND session-store.db directly (turn count updates, new sessions).
        // The DB file lives in the parent copilot_dir, not under session-state/.
        vec![self.session_state_dir(), self.db_path()]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
