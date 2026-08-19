mod window_manager;
mod input_engine;
mod vision_engine;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod interpreter;

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

#[tauri::command]
fn run_script(script: String) -> Result<Vec<interpreter::LogEvent>, String> {
    let controller = interpreter::get_global_controller();
    interpreter::run_script_with_events(&script, controller)
}

#[tauri::command]
fn stop_script() -> Result<String, String> {
    interpreter::stop_script();
    Ok("Script stopped".into())
}

#[tauri::command]
fn pause_script() -> Result<String, String> {
    interpreter::pause_script();
    Ok("Script paused".into())
}

#[tauri::command]
fn resume_script() -> Result<String, String> {
    interpreter::resume_script();
    Ok("Script resumed".into())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            // Window management
            list_windows,
            find_window,
            get_foreground_window,
            // Mouse input
            input_engine::mouse_move,
            input_engine::mouse_move_relative,
            input_engine::mouse_click,
            input_engine::mouse_press,
            input_engine::mouse_release,
            // Keyboard input
            input_engine::key_tap,
            input_engine::key_hold,
            input_engine::key_release,
            input_engine::key_type_text,
            // Vision / pixel capture
            vision_engine::list_monitors,
            vision_engine::get_pixel_color,
            vision_engine::capture_region,
            // Script execution
            run_script,
            stop_script,
            pause_script,
            resume_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BlackHoleMacro");
}
