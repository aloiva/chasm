pub mod copilot_cli;
pub mod vscode_copilot;

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

/// Unified session summary — common across all AI tools.
/// Each source adapter converts its native format into this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Unique identifier (GUID for Copilot CLI, session ID for VS Code)
    pub id: String,
    /// Source adapter name ("copilot-cli", "vscode-copilot")
    pub source: String,
    /// Human-readable title (from summary, first message, or VS Code title)
    pub title: Option<String>,
    /// Number of conversation turns
    pub turn_count: u32,
    /// Working directory / project path
    pub cwd: Option<String>,
    /// Git branch (if available)
    pub branch: Option<String>,
    /// When the session was created
    pub created_at: Option<String>,
    /// When the session was last updated
    pub updated_at: Option<String>,
    /// Preview of the first user message
    pub first_message: Option<String>,
    /// Total size of session data on disk (bytes)
    pub size_bytes: Option<u64>,
    /// Whether the session has checkpoints (Copilot CLI specific)
    pub has_checkpoints: bool,
    /// Whether this session still exists on disk
    pub exists_on_disk: bool,
    /// Path to the session's storage directory on disk
    pub storage_path: Option<String>,
    /// Session activity status ("recent" if updated within 5 minutes, null otherwise)
    pub status: Option<String>,
    /// Source-specific metadata (extensible without changing the struct)
    pub extra: HashMap<String, String>,
}

/// A single conversation turn (user message + assistant response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub turn_index: u32,
    pub user_message: Option<String>,
    pub assistant_response: Option<String>,
    pub timestamp: Option<String>,
}

/// A checkpoint marker in the conversation (Copilot CLI specific).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub number: u32,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub after_turn: Option<u32>,
}

/// Full session detail — loaded on demand when user clicks a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub turns: Vec<ConversationTurn>,
    pub checkpoints: Vec<Checkpoint>,
    /// Files touched during the session
    pub files_touched: Vec<String>,
}

/// What happens when the user clicks "Resume" on a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResumeAction {
    /// Spawn a terminal process (e.g., pwsh with copilot resume)
    SpawnTerminal { command: String, args: Vec<String>, cwd: Option<String> },
    /// Open an application (e.g., VS Code with folder)
    OpenApplication { command: String, args: Vec<String> },
    /// Not supported for this source
    NotSupported { reason: String },
}

/// Source adapter errors — designed for graceful degradation.
/// Warnings are logged and skipped. Fatal errors disable the source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceError {
    /// Non-fatal: missing file, locked DB, deleted session. Log and continue.
    Warning(String),
    /// Fatal: source directory doesn't exist. Disable this source.
    Fatal(String),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::Warning(msg) => write!(f, "Warning: {}", msg),
            SourceError::Fatal(msg) => write!(f, "Fatal: {}", msg),
        }
    }
}

impl std::error::Error for SourceError {}

/// Compute session status based on updated_at timestamp.
/// Returns "recent" if updated within 5 minutes, None otherwise.
pub fn compute_status(updated_at: &Option<String>) -> Option<String> {
    let ts = updated_at.as_deref()?;
    let parsed = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S"))
        .ok()?;
    let now = chrono::Local::now().naive_local();
    if (now - parsed).num_minutes() < 5 {
        Some("recent".to_string())
    } else {
        None
    }
}

/// The core abstraction — every AI tool implements this trait.
/// Maximum abstraction so new tools can be added by implementing one trait.
pub trait SessionSource: Send + Sync {
    /// Internal identifier (e.g., "copilot-cli", "vscode-copilot")
    fn name(&self) -> &str;

    /// Human-readable display name (e.g., "Copilot CLI", "VS Code Copilot Chat")
    fn display_name(&self) -> &str;

    /// Emoji or icon identifier for the UI
    fn icon(&self) -> &str;

    /// Check if this tool is installed / data directory exists on this machine
    fn is_available(&self) -> bool;

    /// Scan all sessions from this source. Never panic — return errors per session.
    fn scan(&self) -> Result<Vec<SessionSummary>, SourceError>;

    /// Load full conversation detail for a specific session (on demand).
    fn load_detail(&self, id: &str) -> Result<SessionDetail, SourceError>;

    /// Rename a session (update title/summary). Not all sources support this.
    fn rename(&self, id: &str, name: &str) -> Result<(), SourceError>;

    /// Delete a session. Not all sources support this.
    fn delete(&self, id: &str) -> Result<(), SourceError>;

    /// Get the resume action for a session.
    fn resume(&self, id: &str) -> Result<ResumeAction, SourceError>;

    /// Directories to watch for filesystem changes (for live updates).
    fn watch_paths(&self) -> Vec<PathBuf>;

    /// Search turn messages for a query string, returning matching session IDs.
    /// Default: returns empty (source doesn't support message search).
    fn search_turns(&self, _query: &str) -> Vec<String> {
        Vec::new()
    }

    /// Downcast support for source-specific configuration.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Registry of all enabled source adapters.
pub struct SourceRegistry {
    sources: Vec<Box<dyn SessionSource>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn register(&mut self, source: Box<dyn SessionSource>) {
        self.sources.push(source);
    }

    pub fn available_sources(&self) -> Vec<&dyn SessionSource> {
        self.sources.iter().filter(|s| s.is_available()).map(|s| s.as_ref()).collect()
    }

    pub fn all_sources(&self) -> Vec<&dyn SessionSource> {
        self.sources.iter().map(|s| s.as_ref()).collect()
    }

    pub fn get_source(&self, name: &str) -> Option<&dyn SessionSource> {
        self.sources.iter().find(|s| s.name() == name).map(|s| s.as_ref())
    }

    pub fn get_source_mut(&mut self, name: &str) -> Option<&mut Box<dyn SessionSource>> {
        self.sources.iter_mut().find(|s| s.name() == name)
    }

    /// Scan all enabled sources, collecting sessions and warnings.
    pub fn scan_all(&self) -> (Vec<SessionSummary>, Vec<String>) {
        let mut all_sessions = Vec::new();
        let mut warnings = Vec::new();

        for source in &self.sources {
            if !source.is_available() {
                continue;
            }
            match source.scan() {
                Ok(sessions) => all_sessions.extend(sessions),
                Err(SourceError::Warning(msg)) => {
                    warnings.push(format!("[{}] {}", source.name(), msg));
                }
                Err(SourceError::Fatal(msg)) => {
                    warnings.push(format!("[{}] FATAL: {}", source.name(), msg));
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        all_sessions.sort_by(|a, b| {
            b.updated_at.as_deref().unwrap_or("").cmp(a.updated_at.as_deref().unwrap_or(""))
        });

        (all_sessions, warnings)
    }

    /// Search turn messages across all sources that support it.
    pub fn search_turns(&self, query: &str) -> Vec<String> {
        let mut ids = Vec::new();
        for source in &self.sources {
            if source.is_available() {
                ids.extend(source.search_turns(query));
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }
}
