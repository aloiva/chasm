# Data Sources Reference

Detailed format documentation for each AI tool's session storage.

## 1. Copilot CLI

### Base Path
`~/.copilot/` (fixed, no config override)

### session-store.db (SQLite, ~896 KB)

**sessions table**:
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Session GUID |
| cwd | TEXT | Working directory at session start |
| repository | TEXT | Git repository |
| branch | TEXT | Git branch |
| summary | TEXT | Auto-generated summary |
| created_at | TEXT | ISO 8601 timestamp |
| updated_at | TEXT | ISO 8601 timestamp |
| host_type | TEXT | Host type |

**turns table**:
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment |
| session_id | TEXT FK | References sessions.id |
| turn_index | INTEGER | 0-based turn number |
| user_message | TEXT | Full user message |
| assistant_response | TEXT | Full assistant response |
| timestamp | TEXT | ISO 8601 timestamp |

**checkpoints table**:
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto-increment |
| session_id | TEXT FK | References sessions.id |
| checkpoint_number | INTEGER | Sequential number |
| title | TEXT | Checkpoint title |
| overview | TEXT | Summary of work done |
| history, work_done, technical_details, important_files, next_steps | TEXT | Detailed sections |

**session_files table**: `session_id, file_path, tool_name (edit/create), turn_index`
**session_refs table**: `session_id, ref_type (commit/pr/issue), ref_value, turn_index`
**search_index**: FTS5 virtual table for full-text search

### session-state/{id}/ (per-session folder)

| File | Format | Description |
|------|--------|-------------|
| `workspace.yaml` | YAML | `id`, `cwd`, `summary`, `summary_count`, `created_at`, `updated_at` |
| `events.jsonl` | JSONL | Full event stream (can be large, 5KB-60MB) |
| `plan.md` | Markdown | Implementation plan |
| `session.db` | SQLite | Per-session database (todos, custom tables) |
| `inuse.{pid}.lock` | Lock file | Present while session is active |
| `checkpoints/` | Directory | Checkpoint markdown files |
| `files/` | Directory | Persistent artifacts |
| `vscode.metadata.json` | JSON | VS Code integration metadata |

### workspace.yaml example
```yaml
id: 20db16f9-0a7f-4c88-bb21-dd0f1d14220b
cwd: C:\Users\prgaddam
summary: Identify Scale Controller Classes
summary_count: 2
created_at: 2026-03-05T10:54:34.329Z
updated_at: 2026-04-08T13:26:36.704Z
```

### events.jsonl line format
```json
{"type":"session.start","data":{"sessionId":"...","version":1,"producer":"copilot-agent","copilotVersion":"0.0.420","startTime":"...","context":{"cwd":"..."}},"id":"...","timestamp":"...","parentId":null}
```

---

## 2. VS Code Copilot Chat

### Base Path
`%APPDATA%/Code/User/workspaceStorage/` (Windows)

Each workspace has a hash-named directory (e.g., `06030a2da987f22a151fd8a7dbd7770d/`).

### state.vscdb (SQLite, per-workspace)

Single table: `ItemTable` with `key TEXT PRIMARY KEY, value TEXT`

**Key: `chat.ChatSessionStore.index`**
```json
{
  "version": 1,
  "entries": {
    "c5ff71e7-c48d-434f-8c8a-4d835fdd7b0b": {
      "sessionId": "c5ff71e7-c48d-434f-8c8a-4d835fdd7b0b",
      "title": "Fixing RabbitMQOutput attribute usage in Azure Functions",
      "lastMessageDate": 1761704533416,
      "isImported": false,
      "initialLocation": "panel",
      "isEmpty": false
    }
  }
}
```

**Key: `memento/interactive-session`**
Contains user prompt history per mode (copilot, editor).

**Key: `workspace.json`** (file, not DB key)
```json
{"folder": "file:///q%3A/src/AAPT-Antares-ScaleController"}
```
Maps workspace hash → project folder URI.

### Stats (measured on this machine)
- 89 workspaces with chat data
- 215 total Copilot Chat sessions
- Top workspace: 52 sessions (AAPT-Antares-ScaleController)

---

## 3. Claude Code (Future Reference)

### Base Path
`~/.claude/`

### Directory Structure
```
~/.claude/
├── CLAUDE.md                           # Global instructions
├── settings.json                       # Global config
├── history.jsonl                       # Global prompt history
└── projects/
    └── -home-user-myproject/           # Path encoded with - replacing /
        ├── session-2025-05-05.jsonl    # One session per file
        └── attachments/                # Pasted files
```

### Session JSONL line format
```json
{"type":"user","uuid":"abc-123","sessionId":"session-456","timestamp":"2025-01-17T10:00:00Z","message":{"text":"How do I implement X?"}}
{"type":"assistant","uuid":"def-789","sessionId":"session-456","timestamp":"2025-01-17T10:01:00Z","message":{"text":"Here is how..."}}
```
