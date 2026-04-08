pub mod adapters;

use adapters::{
    copilot_cli::CopilotCliSource, vscode_copilot::VsCodeCopilotSource, ResumeAction,
    SessionDetail, SessionSummary, SourceRegistry,
};
use std::sync::Mutex;
use tauri::State;

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
) -> Result<ResumeAction, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let adapter = registry
        .get_source(&source)
        .ok_or_else(|| format!("Source '{}' not found", source))?;
    adapter.resume(&id).map_err(|e| e.to_string())
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            registry: Mutex::new(registry),
        })
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            get_session_detail,
            rename_session,
            delete_sessions,
            resume_session,
            get_available_sources,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
