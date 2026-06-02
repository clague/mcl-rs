// Font Configuration Module
// Platform-specific font selection for the launcher

use iced::Font;
use log::{info, debug, warn};

/// Get the default font configuration for the current platform
pub fn get_system_font() -> Font {
    #[cfg(target_os = "linux")]
    {
        get_linux_font()
    }
    
    #[cfg(target_os = "windows")]
    {
        Font::with_name("Microsoft YaHei")
    }
    
    #[cfg(target_os = "macos")]
    {
        Font::with_name("PingFang SC")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Font::DEFAULT
    }
}

/// Get font file path from fontconfig
fn get_fontconfig_font_file(pattern: &str) -> Option<String> {
    let output = std::process::Command::new("fc-match")
        .args(["-f", "%{file}", pattern])
        .output()
        .ok()?;
    
    let font_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if font_path.is_empty() || !std::path::Path::new(&font_path).exists() {
        None
    } else {
        Some(font_path)
    }
}

/// Get font on Linux using fontconfig
fn get_linux_font() -> Font {
    // Try to load system font file
    if let Some(font_path) = get_fontconfig_font_file("sans-serif") {
        info!("Loading system font: {}", font_path);
        match std::fs::read(&font_path) {
            Ok(font_data) => {
                // Leak the data to get 'static lifetime
                let static_data: &'static [u8] = Box::leak(font_data.into_boxed_slice());
                iced::font::load(static_data);
                info!("System font loaded successfully");
            }
            Err(e) => {
                warn!("Failed to read font file: {}: {}", font_path, e);
            }
        }
    }
    
    // Return font with detected name
    if let Some(font_name) = get_fontconfig_font_name("sans-serif") {
        info!("Using font: {}", font_name);
        let static_name: &'static str = Box::leak(font_name.into_boxed_str());
        return Font::with_name(static_name);
    }
    
    // Fallback
    info!("Using fallback font: sans-serif");
    Font::with_name("sans-serif")
}

/// Query fontconfig for font family name
fn get_fontconfig_font_name(pattern: &str) -> Option<String> {
    let output = std::process::Command::new("fc-match")
        .args(["-f", "%{family}", pattern])
        .output()
        .ok()?;
    
    let font_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if font_name.is_empty() {
        return None;
    }
    
    // Take the first font family
    let first_font = font_name.split(',')
        .next()
        .unwrap_or(&font_name)
        .trim()
        .to_string();
    
    if first_font.is_empty() {
        None
    } else {
        Some(first_font)
    }
}

/// Get the monospace font for code/technical text
pub fn get_monospace_font() -> Font {
    #[cfg(target_os = "linux")]
    {
        // Try to load monospace font
        if let Some(font_path) = get_fontconfig_font_file("monospace") {
            info!("Loading monospace font: {}", font_path);
            if let Ok(font_data) = std::fs::read(&font_path) {
                let static_data: &'static [u8] = Box::leak(font_data.into_boxed_slice());
                iced::font::load(static_data);
            }
        }
        
        if let Some(font_name) = get_fontconfig_font_name("monospace") {
            let static_name: &'static str = Box::leak(font_name.into_boxed_str());
            return Font::with_name(static_name);
        }
        Font::with_name("Monospace")
    }
    
    #[cfg(target_os = "windows")]
    {
        Font::with_name("Consolas")
    }
    
    #[cfg(target_os = "macos")]
    {
        Font::with_name("Menlo")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Font::MONOSPACE
    }
}

/// Get the default font family name for logging
pub fn get_default_font_family() -> String {
    #[cfg(target_os = "linux")]
    {
        get_fontconfig_font_name("sans-serif").unwrap_or_else(|| "sans-serif".to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        "Microsoft YaHei".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        "PingFang SC".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "sans-serif".to_string()
    }
}