use serde::{Deserialize, Serialize};
use xcap::Monitor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelColor {
    pub x: u32,
    pub y: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureInfo {
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
}

/// Get info about all monitors
#[tauri::command]
pub fn list_monitors() -> Result<Vec<CaptureInfo>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    Ok(monitors
        .iter()
        .map(|m| CaptureInfo {
            width: m.width().unwrap_or(0),
            height: m.height().unwrap_or(0),
            monitor_name: m.name().unwrap_or_else(|_| "Unknown".to_string()),
        })
        .collect())
}

/// Capture the primary monitor and read a pixel color at (x, y)
#[tauri::command]
pub fn get_pixel_color(x: u32, y: u32) -> Result<PixelColor, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    let monitor = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or(monitors.first())
        .ok_or_else(|| "No monitors found".to_string())?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    if x >= image.width() || y >= image.height() {
        return Err(format!(
            "Pixel ({}, {}) is out of bounds (screen is {}x{})",
            x, y, image.width(), image.height()
        ));
    }

    let pixel = image.get_pixel(x, y);
    let rgba = pixel.0;
    let (r, g, b) = (rgba[0], rgba[1], rgba[2]);

    Ok(PixelColor {
        x,
        y,
        r,
        g,
        b,
        hex: format!("#{:02X}{:02X}{:02X}", r, g, b),
    })
}

/// Capture a region of the primary monitor and return its dimensions
#[tauri::command]
pub fn capture_region(x: u32, y: u32, width: u32, height: u32) -> Result<Vec<Vec<PixelColor>>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    let monitor = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or(monitors.first())
        .ok_or_else(|| "No monitors found".to_string())?;

    let image = monitor
        .capture_region(x, y, width, height)
        .map_err(|e| format!("Region capture failed: {}", e))?;

    let mut rows = Vec::new();
    for row_y in 0..height.min(image.height()) {
        let mut row = Vec::new();
        for col_x in 0..width.min(image.width()) {
            let pixel = image.get_pixel(col_x, row_y);
            let rgba = pixel.0;
            let (r, g, b) = (rgba[0], rgba[1], rgba[2]);
            row.push(PixelColor {
                x: x + col_x,
                y: y + row_y,
                r,
                g,
                b,
                hex: format!("#{:02X}{:02X}{:02X}", r, g, b),
            });
        }
        rows.push(row);
    }
    Ok(rows)
}