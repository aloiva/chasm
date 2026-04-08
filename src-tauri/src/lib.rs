pub mod adapters;

use adapters::{
    copilot_cli::CopilotCliSource, vscode_copilot::VsCodeCopilotSource, ResumeAction,
    SessionDetail, SessionSummary, SourceRegistry,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
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
        ResumeAction::SpawnTerminal { command, args } => {
            let mut cmd = StdCommand::new(&command);
            cmd.args(&args);
            #[cfg(windows)]
            cmd.creation_flags(0x00000010); // CREATE_NEW_CONSOLE
            cmd.spawn()
                .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
            Ok(format!("Spawned terminal: {} {:?}", command, args))
        }
        ResumeAction::OpenApplication { command, args } => {
            StdCommand::new(&command)
                .args(&args)
                .spawn()
                .map_err(|e| format!("Failed to open application: {}", e))?;
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
            get_available_sources,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
