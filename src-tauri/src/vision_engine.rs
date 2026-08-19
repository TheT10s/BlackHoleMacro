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

/// Template match: capture a region and compare against a reference image.
/// Returns a score 0.0-1.0 (1.0 = perfect match) using normalized cross-correlation.
pub fn template_match(
    rx: u32, ry: u32, rw: u32, rh: u32,
    template_path: &str,
) -> Result<f64, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    let monitor = monitors.iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or(monitors.first())
        .ok_or_else(|| "No monitors found".to_string())?;

    let screen_img = monitor.capture_region(rx, ry, rw, rh)
        .map_err(|e| format!("Region capture failed: {}", e))?;

    let template = image::open(template_path)
        .map_err(|e| format!("Failed to load template '{}': {}", template_path, e))?;
    let template = template.to_rgb8();

    let sw = screen_img.width() as usize;
    let sh = screen_img.height() as usize;
    let tw = template.width() as usize;
    let th = template.height() as usize;

    if tw > sw || th > sh {
        return Err(format!("Template ({}x{}) larger than region ({}x{})", tw, th, sw, sh));
    }

    // Convert to grayscale
    let mut screen_gray = vec![0.0f64; sw * sh];
    for y in 0..sh {
        for x in 0..sw {
            let p = screen_img.get_pixel(x as u32, y as u32).0;
            screen_gray[y * sw + x] = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
        }
    }

    let mut template_gray = vec![0.0f64; tw * th];
    for y in 0..th {
        for x in 0..tw {
            let p = template.get_pixel(x as u32, y as u32).0;
            template_gray[y * tw + x] = 0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64;
        }
    }

    // Compute template mean
    let t_mean: f64 = template_gray.iter().sum::<f64>() / (tw * th) as f64;

    // Slide template over screen region
    let mut best_score = -1.0_f64;
    for sy in 0..=(sh - th) {
        for sx in 0..=(sw - tw) {
            // Compute NCC for this position
            let mut sum_st = 0.0_f64;
            let mut sum_ss = 0.0_f64;
            let mut sum_tt = 0.0_f64;
            let mut s_sum = 0.0_f64;

            for ty in 0..th {
                for tx in 0..tw {
                    let sp = screen_gray[(sy + ty) * sw + (sx + tx)];
                    s_sum += sp;
                }
            }
            let s_mean = s_sum / (tw * th) as f64;

            for ty in 0..th {
                for tx in 0..tw {
                    let sp = screen_gray[(sy + ty) * sw + (sx + tx)];
                    let tp = template_gray[ty * tw + tx];
                    let sd = sp - s_mean;
                    let td = tp - t_mean;
                    sum_st += sd * td;
                    sum_ss += sd * sd;
                    sum_tt += td * td;
                }
            }

            let denom = (sum_ss * sum_tt).sqrt();
            let score = if denom < 1e-10 { 0.0 } else { sum_st / denom };
            if score > best_score { best_score = score; }
        }
    }

    Ok(best_score)
}
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