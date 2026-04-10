pub mod adapters;

use adapters::{
    copilot_cli::CopilotCliSource, vscode_copilot::VsCodeCopilotSource, ResumeAction,
    SessionDetail, SessionSummary, SourceRegistry,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

struct AppState {
    registry: Mutex<SourceRegistry>,
}

#[tauri::command]
fn list_sessions(state: State<AppState>) -> Result<Vec<SessionSummary>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let (sessions, warnings) = registry.scan_all();
    for w in &warnings {
        eprintln!("{}", w);
    }
    Ok(sessions)
}

#[tauri::command]
fn get_session_detail(
    state: State<AppState>,
    source: String,
    id: String,
) -> Result<SessionDetail, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let adapter = registry
        .get_source(&source)
        .ok_or_else(|| format!("Source '{}' not found", source))?;
    adapter.load_detail(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_session(
    state: State<AppState>,
    source: String,
    id: String,
    name: String,
) -> Result<(), String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let adapter = registry
        .get_source(&source)
        .ok_or_else(|| format!("Source '{}' not found", source))?;
    adapter.rename(&id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_sessions(
    state: State<AppState>,
    source: String,
    ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let adapter = registry
        .get_source(&source)
        .ok_or_else(|| format!("Source '{}' not found", source))?;

    let mut errors = Vec::new();
    for id in &ids {
        if let Err(e) = adapter.delete(id) {
            errors.push(format!("{}: {}", id, e));
        }
    }
    Ok(errors)
}

#[tauri::command]
fn resume_session(
    state: State<AppState>,
    source: String,
    id: String,
) -> Result<String, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let adapter = registry
        .get_source(&source)
        .ok_or_else(|| format!("Source '{}' not found", source))?;
    let action = adapter.resume(&id).map_err(|e| e.to_string())?;

    match action {
        ResumeAction::SpawnTerminal { command, args, cwd } => {
            // Use PowerShell's Start-Process to launch a fully interactive terminal.
            // Direct spawning from a GUI app (Tauri) leaves stdin piped to the parent,
            // making the terminal non-interactive. cmd /c start has quoting issues with
            // Rust's MSVCRT arg encoding. Start-Process avoids both problems.
            #[cfg(windows)]
            {
                let escaped_args: Vec<String> = args.iter()
                    .map(|a| format!("'{}'", a.replace("'", "''")))
                    .collect();
                let arg_list = escaped_args.join(", ");

                let mut ps_cmd = format!(
                    "Start-Process -FilePath '{}'",
                    command.replace("'", "''")
                );
                if let Some(ref dir) = cwd {
                    ps_cmd.push_str(&format!(
                        " -WorkingDirectory '{}'",
                        dir.replace("'", "''")
                    ));
                }
                ps_cmd.push_str(&format!(" -ArgumentList @({})", arg_list));

                StdCommand::new("pwsh")
                    .args(["-NoProfile", "-Command", &ps_cmd])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW for launcher
                    .spawn()
                    .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
            }
            #[cfg(not(windows))]
            {
                let mut cmd = StdCommand::new(&command);
                cmd.args(&args);
                if let Some(ref dir) = cwd {
                    cmd.current_dir(dir);
                }
                cmd.spawn()
                    .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
            }
            Ok(format!("Spawned terminal: {} {:?}", command, args))
        }
        ResumeAction::OpenApplication { command, args } => {
            // On Windows, apps like "code" are actually .cmd scripts (code.cmd).
            // Rust's Command::new can find them via PATHEXT but they must run through
            // cmd.exe. Use Start-Process for reliability and to hide the launcher window.
            #[cfg(windows)]
            {
                let escaped_args: Vec<String> = args.iter()
                    .map(|a| format!("'{}'", a.replace("'", "''")))
                    .collect();
                let arg_list = escaped_args.join(", ");

                let ps_cmd = if args.is_empty() {
                    format!("Start-Process -FilePath '{}'", command.replace("'", "''"))
                } else {
                    format!(
                        "Start-Process -FilePath '{}' -ArgumentList @({})",
                        command.replace("'", "''"),
                        arg_list
                    )
                };

                StdCommand::new("pwsh")
                    .args(["-NoProfile", "-Command", &ps_cmd])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .spawn()
                    .map_err(|e| format!("Failed to open application: {}", e))?;
            }
            #[cfg(not(windows))]
            {
                StdCommand::new(&command)
                    .args(&args)
                    .spawn()
                    .map_err(|e| format!("Failed to open application: {}", e))?;
            }
            Ok(format!("Opened application: {} {:?}", command, args))
        }
        ResumeAction::NotSupported { reason } => Err(reason),
    }
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        StdCommand::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        StdCommand::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        StdCommand::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

/// Run `/chronicle reindex` inside a Copilot CLI session to rebuild the session index.
/// This removes stale entries (e.g. deleted sessions) from session-store.db.
#[tauri::command]
fn reindex_sessions() -> Result<String, String> {
    #[cfg(windows)]
    {
        // Use Start-Process to launch a fully interactive terminal
        let ps_cmd = r#"Start-Process -FilePath 'pwsh' -ArgumentList @('-NoExit', '-Command', 'copilot -i ''/chronicle reindex''')"#;
        StdCommand::new("pwsh")
            .args(["-NoProfile", "-Command", ps_cmd])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW for launcher
            .spawn()
            .map_err(|e| format!("Failed to spawn reindex terminal: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        StdCommand::new(&shell)
            .args(["-c", "copilot -i '/chronicle reindex'"])
            .spawn()
            .map_err(|e| format!("Failed to spawn reindex terminal: {}", e))?;
    }
    Ok("Reindex started".to_string())
}

/// Spawn a new session terminal in a given path.
/// `session_type` is "cli" (Copilot CLI) or "dobby".
/// If `path` is empty, defaults to the current user's home directory.
#[tauri::command]
fn search_messages(state: State<AppState>, query: String) -> Result<Vec<String>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.search_turns(&query))
}

#[tauri::command]
fn new_session(path: String, session_type: String) -> Result<String, String> {
    let mut work_dir = if path.is_empty() {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    } else {
        path.clone()
    };

    let launch_cmd = match session_type.as_str() {
        "cli" => "copilot".to_string(),
        "dobby" => {
            // Validate by checking for Start-Copilot.ps1 in parent directory
            let parent = std::path::Path::new(&work_dir)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from(&work_dir));
            let script = parent.join("Start-Copilot.ps1");
            if !script.exists() {
                return Err(format!(
                    "No Start-Copilot.ps1 found in {}. Is this a Dobby agents directory?",
                    parent.display()
                ));
            }
            work_dir = parent.to_string_lossy().to_string();
            ".\\Start-Copilot.ps1".to_string()
        }
        other => return Err(format!("Unknown session type: {}", other)),
    };

    #[cfg(windows)]
    {
        // Use Start-Process to launch a fully interactive terminal
        let escaped_cmd = launch_cmd.replace("'", "''");
        let ps_cmd = format!(
            "Start-Process -FilePath 'pwsh' -WorkingDirectory '{}' -ArgumentList @('-NoExit', '-Command', '{}')",
            work_dir.replace("'", "''"),
            escaped_cmd
        );
        StdCommand::new("pwsh")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW for launcher
            .spawn()
            .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        let mut cmd = StdCommand::new(&shell);
        cmd.args(["-c", &launch_cmd]);
        cmd.current_dir(&work_dir);
        cmd.spawn()
            .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
    }
    Ok(format!("Started {} session in {}", session_type, work_dir))
}

#[tauri::command]
fn get_available_sources(state: State<AppState>) -> Result<Vec<SourceInfo>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    Ok(registry
        .all_sources()
        .iter()
        .map(|s| SourceInfo {
            name: s.name().to_string(),
            display_name: s.display_name().to_string(),
            icon: s.icon().to_string(),
            available: s.is_available(),
        })
        .collect())
}

#[derive(serde::Serialize)]
struct SourceInfo {
    name: String,
    display_name: String,
    icon: String,
    available: bool,
}

#[tauri::command]
fn get_copilot_cli_path(state: State<AppState>) -> Result<String, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let source = registry.get_source("copilot-cli").ok_or("Copilot CLI source not found")?;
    let cli = source
        .as_any()
        .downcast_ref::<CopilotCliSource>()
        .ok_or("Downcast failed")?;
    let joined = cli.session_state_dirs().iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(joined)
}

#[tauri::command]
fn set_copilot_cli_path(state: State<AppState>, path: String) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    let source = registry
        .get_source_mut("copilot-cli")
        .ok_or("Copilot CLI source not found")?;
    let cli = source
        .as_any_mut()
        .downcast_mut::<CopilotCliSource>()
        .ok_or("Downcast failed")?;
    let paths: Vec<PathBuf> = path.split(',')
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    if paths.is_empty() {
        return Err("No valid paths provided".to_string());
    }
    for p in &paths {
        if !p.exists() {
            return Err(format!("Path does not exist: {}", p.display()));
        }
    }
    cli.set_session_state_dirs(paths);
    Ok(())
}

#[tauri::command]
fn get_copilot_db_path(state: State<AppState>) -> Result<String, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let source = registry.get_source("copilot-cli").ok_or("Copilot CLI source not found")?;
    let cli = source
        .as_any()
        .downcast_ref::<CopilotCliSource>()
        .ok_or("Downcast failed")?;
    let joined = cli.db_files().iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(joined)
}

#[tauri::command]
fn set_copilot_db_path(state: State<AppState>, path: String) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    let source = registry
        .get_source_mut("copilot-cli")
        .ok_or("Copilot CLI source not found")?;
    let cli = source
        .as_any_mut()
        .downcast_mut::<CopilotCliSource>()
        .ok_or("Downcast failed")?;
    let paths: Vec<PathBuf> = path.split(',')
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    if paths.is_empty() {
        return Err("No valid paths provided".to_string());
    }
    for p in &paths {
        if !p.exists() {
            return Err(format!("Path does not exist: {}", p.display()));
        }
    }
    cli.set_db_files(paths);
    Ok(())
}

#[tauri::command]
fn is_dobby_path(path: String) -> bool {
    if path.is_empty() {
        return false;
    }
    let parent = std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new(&path));
    parent.join("Start-Copilot.ps1").exists()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut registry = SourceRegistry::new();
    registry.register(Box::new(CopilotCliSource::new()));
    registry.register(Box::new(VsCodeCopilotSource::new()));

    // Collect watch paths before moving registry into state
    let watch_paths: Vec<std::path::PathBuf> = registry
        .all_sources()
        .iter()
        .filter(|s| s.is_available())
        .flat_map(|s| s.watch_paths())
        .filter(|p| p.exists())
        .collect();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            registry: Mutex::new(registry),
        })
        .setup(move |app| {
            // Set up filesystem watcher with debounce
            let handle = app.handle().clone();
            let last_emit = std::sync::Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10)));
            let last_emit_clone = last_emit.clone();

            let mut watcher: RecommendedWatcher = notify::recommended_watcher(
                move |res: Result<notify::Event, notify::Error>| {
                    if res.is_ok() {
                        let mut last = last_emit_clone.lock().unwrap();
                        if last.elapsed() >= Duration::from_secs(2) {
                            *last = Instant::now();
                            let _ = handle.emit("sessions-changed", ());
                        }
                    }
                },
            )
            .expect("Failed to create file watcher");

            for path in &watch_paths {
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    eprintln!("[watcher] Failed to watch {}: {}", path.display(), e);
                }
            }

            // Keep watcher alive for the app's lifetime
            std::mem::forget(watcher);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_session_detail,
            rename_session,
            delete_sessions,
            resume_session,
            open_folder,
            search_messages,
            new_session,
            get_available_sources,
            reindex_sessions,
            get_copilot_cli_path,
            set_copilot_cli_path,
            get_copilot_db_path,
            set_copilot_db_path,
            is_dobby_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
