use serde::{Deserialize, Serialize};
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: i64,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub process_name: String,
}

pub struct WindowManager;

impl WindowManager {
    pub fn new() -> Self {
        Self
    }

    pub fn list_windows(&self) -> Vec<WindowInfo> {
        let mut windows: Vec<WindowInfo> = Vec::new();

        unsafe {
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut windows as *mut _ as isize));
        }

        windows
    }

    pub fn find_window(&self, title_pattern: &str) -> Option<WindowInfo> {
        let windows = self.list_windows();
        let pattern_lower = title_pattern.to_lowercase();

        windows
            .into_iter()
            .find(|w| w.title.to_lowercase().contains(&pattern_lower))
    }

    pub fn get_foreground_window(&self) -> Option<WindowInfo> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }
            self.get_window_info(hwnd)
        }
    }

    fn get_window_info(&self, hwnd: HWND) -> Option<WindowInfo> {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return None;
            }

            let title = self.get_window_title(hwnd);
            if title.is_empty() {
                return None;
            }

            let class_name = self.get_class_name(hwnd);
            let process_id = self.get_window_process_id(hwnd);
            let process_name = self.get_process_name(process_id);

            Some(WindowInfo {
                hwnd: hwnd.0 as i64,
                title,
                class_name,
                process_id,
                process_name,
            })
        }
    }

    fn get_window_title(&self, hwnd: HWND) -> String {
        unsafe {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return String::new();
            }

            let mut buffer = vec![0u16; (len + 1) as usize];
            GetWindowTextW(hwnd, &mut buffer);
            String::from_utf16_lossy(&buffer[..len as usize])
        }
    }

    fn get_class_name(&self, hwnd: HWND) -> String {
        unsafe {
            let mut buffer = vec![0u16; 256];
            GetClassNameW(hwnd, &mut buffer);
            String::from_utf16_lossy(&buffer).trim_end_matches('\0').to_string()
        }
    }

    fn get_window_process_id(&self, hwnd: HWND) -> u32 {
        unsafe {
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));
            process_id
        }
    }

    fn get_process_name(&self, process_id: u32) -> String {
        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);
            if process_handle.is_err() {
                return String::new();
            }

            let handle = process_handle.unwrap();
            let mut buffer = vec![0u16; 260];
            let mut size = buffer.len() as u32;

            let result = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            match result {
                Ok(()) => {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    path.split('\\').last().unwrap_or("").to_string()
                }
                Err(_) => String::new(),
            }
        }
    }
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
    let manager = WindowManager;

    if let Some(info) = manager.get_window_info(hwnd) {
        windows.push(info);
    }

    BOOL(1)
}
