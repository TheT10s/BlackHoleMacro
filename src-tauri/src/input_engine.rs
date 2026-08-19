use enigo::{
    Button, Coordinate, Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputResult {
    pub success: bool,
    pub message: String,
}

fn create_enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize input engine: {}", e))
}

fn to_button(btn: &MouseButton) -> Button {
    match btn {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
    }
}

fn parse_key(name: &str) -> Result<Key, String> {
    match name.to_lowercase().as_str() {
        "a" => Ok(Key::Unicode('a')), "b" => Ok(Key::Unicode('b')),
        "c" => Ok(Key::Unicode('c')), "d" => Ok(Key::Unicode('d')),
        "e" => Ok(Key::Unicode('e')), "f" => Ok(Key::Unicode('f')),
        "g" => Ok(Key::Unicode('g')), "h" => Ok(Key::Unicode('h')),
        "i" => Ok(Key::Unicode('i')), "j" => Ok(Key::Unicode('j')),
        "k" => Ok(Key::Unicode('k')), "l" => Ok(Key::Unicode('l')),
        "m" => Ok(Key::Unicode('m')), "n" => Ok(Key::Unicode('n')),
        "o" => Ok(Key::Unicode('o')), "p" => Ok(Key::Unicode('p')),
        "q" => Ok(Key::Unicode('q')), "r" => Ok(Key::Unicode('r')),
        "s" => Ok(Key::Unicode('s')), "t" => Ok(Key::Unicode('t')),
        "u" => Ok(Key::Unicode('u')), "v" => Ok(Key::Unicode('v')),
        "w" => Ok(Key::Unicode('w')), "x" => Ok(Key::Unicode('x')),
        "y" => Ok(Key::Unicode('y')), "z" => Ok(Key::Unicode('z')),
        "0" => Ok(Key::Unicode('0')), "1" => Ok(Key::Unicode('1')),
        "2" => Ok(Key::Unicode('2')), "3" => Ok(Key::Unicode('3')),
        "4" => Ok(Key::Unicode('4')), "5" => Ok(Key::Unicode('5')),
        "6" => Ok(Key::Unicode('6')), "7" => Ok(Key::Unicode('7')),
        "8" => Ok(Key::Unicode('8')), "9" => Ok(Key::Unicode('9')),
        "space" | " " => Ok(Key::Unicode(' ')),
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "insert" | "ins" => Ok(Key::Insert),
        "up" | "uparrow" => Ok(Key::UpArrow),
        "down" | "downarrow" => Ok(Key::DownArrow),
        "left" | "leftarrow" => Ok(Key::LeftArrow),
        "right" | "rightarrow" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home), "end" => Ok(Key::End),
        "pageup" | "pgup" => Ok(Key::PageUp),
        "pagedown" | "pgdn" => Ok(Key::PageDown),
        "f1" => Ok(Key::F1), "f2" => Ok(Key::F2), "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4), "f5" => Ok(Key::F5), "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7), "f8" => Ok(Key::F8), "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10), "f11" => Ok(Key::F11), "f12" => Ok(Key::F12),
        "shift" => Ok(Key::Shift),
        "control" | "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "win" | "super" | "meta" => Ok(Key::Meta),
        "-" | "minus" => Ok(Key::Unicode('-')),
        "=" | "equals" => Ok(Key::Unicode('=')),
        "[" => Ok(Key::Unicode('[')),
        "]" => Ok(Key::Unicode(']')),
        "\\" | "backslash" => Ok(Key::Unicode('\\')),
        ";" | "semicolon" => Ok(Key::Unicode(';')),
        "'" | "quote" => Ok(Key::Unicode('\'')),
        "," | "comma" => Ok(Key::Unicode(',')),
        "." | "period" => Ok(Key::Unicode('.')),
        "/" | "slash" => Ok(Key::Unicode('/')),
        "`" | "backtick" => Ok(Key::Unicode('`')),
        _ => Err(format!("Unknown key: '{}'", name)),
    }
}

// ─── Mouse Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn mouse_move(x: i32, y: i32) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    enigo.move_mouse(x, y, Coordinate::Abs)
        .map_err(|e| format!("Mouse move failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Mouse moved to ({}, {})", x, y) })
}

#[tauri::command]
pub fn mouse_move_relative(dx: i32, dy: i32) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    enigo.move_mouse(dx, dy, Coordinate::Rel)
        .map_err(|e| format!("Mouse move relative failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Mouse moved by ({}, {})", dx, dy) })
}

#[tauri::command]
pub fn mouse_click(button: MouseButton) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let btn = to_button(&button);
    enigo.button(btn, Click)
        .map_err(|e| format!("Mouse click failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Clicked {:?}", button) })
}

#[tauri::command]
pub fn mouse_press(button: MouseButton) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let btn = to_button(&button);
    enigo.button(btn, Press)
        .map_err(|e| format!("Mouse press failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Pressed {:?}", button) })
}

#[tauri::command]
pub fn mouse_release(button: MouseButton) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let btn = to_button(&button);
    enigo.button(btn, Release)
        .map_err(|e| format!("Mouse release failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Released {:?}", button) })
}

// ─── Keyboard Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn key_tap(key: String) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Click)
        .map_err(|e| format!("Key tap failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Tapped key: {}", key) })
}

#[tauri::command]
pub fn key_hold(key: String) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Press)
        .map_err(|e| format!("Key hold failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Holding key: {}", key) })
}

#[tauri::command]
pub fn key_release(key: String) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    let k = parse_key(&key)?;
    enigo.key(k, Release)
        .map_err(|e| format!("Key release failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Released key: {}", key) })
}

#[tauri::command]
pub fn key_type_text(text: String) -> Result<InputResult, String> {
    let mut enigo = create_enigo()?;
    enigo.text(&text)
        .map_err(|e| format!("Text type failed: {}", e))?;
    Ok(InputResult { success: true, message: format!("Typed: {}", text) })
}