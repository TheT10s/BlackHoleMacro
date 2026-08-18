mod window_manager;

use window_manager::{WindowInfo, WindowManager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("BlackHoleMacro ready — targeting: {}", name)
}

#[tauri::command]
fn list_windows() -> Vec<WindowInfo> {
    let manager = WindowManager::new();
    manager.list_windows()
}

#[tauri::command]
fn find_window(title_pattern: String) -> Option<WindowInfo> {
    let manager = WindowManager::new();
    manager.find_window(&title_pattern)
}

#[tauri::command]
fn get_foreground_window() -> Option<WindowInfo> {
    let manager = WindowManager::new();
    manager.get_foreground_window()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            list_windows,
            find_window,
            get_foreground_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running BlackHoleMacro");
}
