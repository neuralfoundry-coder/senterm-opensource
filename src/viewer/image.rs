//! Terminal image preview support
//! 
//! Supports multiple rendering methods:
//! - ASCII art (universal fallback)
//! - Unicode block characters (better quality, most modern terminals)
//! - Sixel (xterm, mlterm, WezTerm)
//! - Kitty Graphics Protocol (Kitty, WezTerm)
//! - iTerm2 Inline Images (iTerm2)

use std::path::Path;
use std::io::Cursor;
use image::{GenericImageView, ImageFormat, DynamicImage};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Image rendering method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageRenderMethod {
    /// ASCII art using characters like @#%*+=-:. 
    Ascii,
    /// Unicode half-block characters (▀▄█ )
    UnicodeBlocks,
    /// Sixel graphics (for supported terminals)
    Sixel,
    /// Kitty graphics protocol
    Kitty,
    /// iTerm2 inline images protocol
    ITerm2,
}

/// Image preview result
#[derive(Clone)]
pub struct ImagePreview {
    /// Rendered content (ASCII/Unicode string or escape sequences)
    pub content: String,
    /// Original image dimensions
    pub width: u32,
    pub height: u32,
    /// Render method used
    pub method: ImageRenderMethod,
    /// Format description
    pub format: String,
    /// File size in bytes
    pub file_size: u64,
}

impl ImagePreview {
    /// Create an error preview
    pub fn error(message: &str) -> Self {
        Self {
            content: format!("Error: {}", message),
            width: 0,
            height: 0,
            method: ImageRenderMethod::Ascii,
            format: "Error".to_string(),
            file_size: 0,
        }
    }
    
    /// Get metadata string
    pub fn metadata(&self) -> String {
        format!(
            "{}x{} | {} | {:.1} KB",
            self.width,
            self.height,
            self.format,
            self.file_size as f64 / 1024.0
        )
    }
}

/// ASCII brightness characters (dark to light)
const ASCII_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Grayscale to ASCII character
fn gray_to_ascii(gray: u8) -> char {
    let index = (gray as usize * (ASCII_CHARS.len() - 1)) / 255;
    ASCII_CHARS[index]
}

/// Load and render image as ASCII art
pub fn load_image_ascii(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    // Get file metadata
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Try to load image using image crate
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return ImagePreview::error(&format!("Failed to load image: {}", e)),
    };
    
    let (orig_width, orig_height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Calculate scaled dimensions (aspect ratio correction for terminal chars)
    // Terminal chars are typically ~2x taller than wide
    let aspect_ratio = orig_width as f32 / orig_height as f32 * 2.0;
    
    let (render_width, render_height) = if aspect_ratio > max_width as f32 / max_height as f32 {
        let w = max_width;
        let h = (max_width as f32 / aspect_ratio) as u32;
        (w.max(1), h.max(1))
    } else {
        let h = max_height;
        let w = (max_height as f32 * aspect_ratio) as u32;
        (w.max(1), h.max(1))
    };
    
    // Resize image
    let resized = img.resize_exact(
        render_width,
        render_height,
        image::imageops::FilterType::Nearest
    );
    let gray = resized.to_luma8();
    
    // Convert to ASCII
    let mut content = String::with_capacity((render_width as usize + 1) * render_height as usize);
    
    for y in 0..render_height {
        for x in 0..render_width {
            let pixel = gray.get_pixel(x, y);
            content.push(gray_to_ascii(255 - pixel[0])); // Invert for dark background
        }
        content.push('\n');
    }
    
    ImagePreview {
        content,
        width: orig_width,
        height: orig_height,
        method: ImageRenderMethod::Ascii,
        format,
        file_size,
    }
}

/// Load and render image using Unicode half-block characters
/// This provides better resolution than ASCII (2 vertical pixels per character)
pub fn load_image_unicode(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return ImagePreview::error(&format!("Failed to load image: {}", e)),
    };
    
    let (orig_width, orig_height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Each terminal row represents 2 pixel rows
    let render_width = max_width;
    let render_height = max_height * 2;
    
    // Resize image
    let resized = img.resize_exact(
        render_width,
        render_height,
        image::imageops::FilterType::Triangle
    );
    let rgba = resized.to_rgba8();
    
    let mut content = String::with_capacity((render_width as usize * 20 + 1) * (render_height as usize / 2));
    
    // Process 2 rows at a time using half-block characters
    for y in (0..render_height).step_by(2) {
        for x in 0..render_width {
            let top = rgba.get_pixel(x, y);
            let bottom = if y + 1 < render_height {
                rgba.get_pixel(x, y + 1)
            } else {
                top
            };
            
            // Use ANSI 24-bit color
            // Upper half block (▀) with foreground = top pixel, background = bottom pixel
            content.push_str(&format!(
                "\x1b[38;2;{};{};{};48;2;{};{};{}m▀",
                top[0], top[1], top[2],
                bottom[0], bottom[1], bottom[2]
            ));
        }
        content.push_str("\x1b[0m\n");
    }
    
    ImagePreview {
        content,
        width: orig_width,
        height: orig_height,
        method: ImageRenderMethod::UnicodeBlocks,
        format,
        file_size,
    }
}

/// Get image metadata without full rendering
#[allow(dead_code)]
pub fn get_image_info(path: &Path) -> Result<(u32, u32, String), String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    let (width, height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    Ok((width, height, format))
}

// ============================================================================
// Sixel Graphics Implementation
// ============================================================================

/// Sixel color palette (256 colors max)
fn build_sixel_palette(img: &DynamicImage) -> (Vec<(u8, u8, u8)>, Vec<Vec<u8>>) {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    // Simple color quantization: use most common colors (max 256)
    use std::collections::HashMap;
    let mut color_counts: HashMap<(u8, u8, u8), usize> = HashMap::new();
    
    for pixel in rgba.pixels() {
        let key = (pixel[0], pixel[1], pixel[2]);
        *color_counts.entry(key).or_insert(0) += 1;
    }
    
    // Get top 256 colors
    let mut colors: Vec<_> = color_counts.into_iter().collect();
    colors.sort_by(|a, b| b.1.cmp(&a.1));
    let palette: Vec<(u8, u8, u8)> = colors.into_iter()
        .take(256)
        .map(|(c, _)| c)
        .collect();
    
    // Map pixels to palette indices
    let mut indexed = vec![vec![0u8; width as usize]; height as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let rgb = (pixel[0], pixel[1], pixel[2]);
            // Find closest color in palette
            let idx = palette.iter()
                .enumerate()
                .min_by_key(|(_, &c)| {
                    let dr = c.0 as i32 - rgb.0 as i32;
                    let dg = c.1 as i32 - rgb.1 as i32;
                    let db = c.2 as i32 - rgb.2 as i32;
                    dr * dr + dg * dg + db * db
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            indexed[y as usize][x as usize] = idx as u8;
        }
    }
    
    (palette, indexed)
}

/// Encode image as Sixel graphics
/// Sixel encodes 6 vertical pixels per character
pub fn load_image_sixel(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return ImagePreview::error(&format!("Failed to load image: {}", e)),
    };
    
    let (orig_width, orig_height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Resize to fit terminal (Sixel doesn't need aspect correction)
    let resized = img.resize(max_width, max_height * 6, image::imageops::FilterType::Lanczos3);
    let (width, height) = resized.dimensions();
    
    // Build color palette and indexed image
    let (palette, indexed) = build_sixel_palette(&resized);
    
    // Build Sixel output
    let mut content = String::new();
    
    // Sixel start: DCS P1 ; P2 ; P3 q
    // P1=0: pixel aspect ratio 2:1
    // P2=0: background color handling
    // P3=0: horizontal grid size
    content.push_str("\x1bPq");
    
    // Define color palette: #index;2;r;g;b (r,g,b are 0-100 percent)
    for (i, &(r, g, b)) in palette.iter().enumerate() {
        content.push_str(&format!(
            "#{};2;{};{};{}",
            i,
            (r as u32 * 100) / 255,
            (g as u32 * 100) / 255,
            (b as u32 * 100) / 255
        ));
    }
    
    // Encode pixel data in 6-row bands
    for band_y in (0..height).step_by(6) {
        for color_idx in 0..palette.len() {
            let mut has_pixels = false;
            let mut band_data = String::new();
            
            for x in 0..width {
                let mut sixel_char: u8 = 0;
                
                for dy in 0..6 {
                    let y = band_y + dy;
                    if y < height {
                        if indexed[y as usize][x as usize] == color_idx as u8 {
                            sixel_char |= 1 << dy;
                            has_pixels = true;
                        }
                    }
                }
                
                // Sixel characters: ? (0x3F) + sixel_char
                band_data.push((0x3F + sixel_char) as char);
            }
            
            if has_pixels {
                // Select color and output data
                content.push_str(&format!("#{}", color_idx));
                content.push_str(&band_data);
                content.push('$'); // Carriage return (return to start of line)
            }
        }
        content.push('-'); // Line feed (move to next band)
    }
    
    // Sixel end: ST (String Terminator)
    content.push_str("\x1b\\");
    
    ImagePreview {
        content,
        width: orig_width,
        height: orig_height,
        method: ImageRenderMethod::Sixel,
        format,
        file_size,
    }
}

// ============================================================================
// Kitty Graphics Protocol Implementation
// ============================================================================

/// Encode image using Kitty Graphics Protocol
/// Sends PNG data as base64 in chunks
pub fn load_image_kitty(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return ImagePreview::error(&format!("Failed to load image: {}", e)),
    };
    
    let (orig_width, orig_height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Resize to fit terminal
    let resized = img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);
    
    // Encode as PNG
    let mut png_data = Vec::new();
    if let Err(e) = resized.write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png) {
        return ImagePreview::error(&format!("Failed to encode PNG: {}", e));
    }
    
    // Base64 encode
    let b64_data = BASE64.encode(&png_data);
    
    // Kitty graphics protocol format:
    // ESC _ G <control data> ; <payload> ESC \
    // For PNG: a=T (transmit), f=100 (PNG format), t=d (direct transmission)
    
    let mut content = String::new();
    
    // Send in chunks of 4096 bytes (Kitty recommendation)
    let chunk_size = 4096;
    let chunks: Vec<&str> = b64_data.as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();
    
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        let more = if is_last { 0 } else { 1 };
        
        if i == 0 {
            // First chunk: include all control parameters
            content.push_str(&format!(
                "\x1b_Ga=T,f=100,m={};{}\x1b\\",
                more, chunk
            ));
        } else {
            // Subsequent chunks: only m parameter
            content.push_str(&format!(
                "\x1b_Gm={};{}\x1b\\",
                more, chunk
            ));
        }
    }
    
    ImagePreview {
        content,
        width: orig_width,
        height: orig_height,
        method: ImageRenderMethod::Kitty,
        format,
        file_size,
    }
}

// ============================================================================
// iTerm2 Inline Images Implementation
// ============================================================================

/// Encode image using iTerm2 Inline Images Protocol
pub fn load_image_iterm2(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return ImagePreview::error(&format!("Failed to load image: {}", e)),
    };
    
    let (orig_width, orig_height) = img.dimensions();
    let format = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Resize to fit terminal
    let resized = img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);
    
    // Encode as PNG
    let mut png_data = Vec::new();
    if let Err(e) = resized.write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png) {
        return ImagePreview::error(&format!("Failed to encode PNG: {}", e));
    }
    
    // Base64 encode
    let b64_data = BASE64.encode(&png_data);
    
    // iTerm2 inline image format:
    // ESC ] 1337 ; File = [arguments] : base64_data BEL
    // Arguments: inline=1 (display inline), size=N (file size), 
    //            width=N, height=N (in cells or px/%)
    
    let content = format!(
        "\x1b]1337;File=inline=1;size={}:{}\x07",
        png_data.len(),
        b64_data
    );
    
    ImagePreview {
        content,
        width: orig_width,
        height: orig_height,
        method: ImageRenderMethod::ITerm2,
        format,
        file_size,
    }
}

/// Check if terminal supports true color
pub fn supports_true_color() -> bool {
    // Check COLORTERM environment variable
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }
    }
    
    // Check TERM for known true color terminals
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") || term.contains("kitty") || term.contains("alacritty") {
            return true;
        }
    }
    
    // Default to true for modern systems
    true
}

/// Detect best image rendering method for current terminal
pub fn detect_render_method() -> ImageRenderMethod {
    // Check for Kitty terminal
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return ImageRenderMethod::Kitty;
    }
    
    // Check TERM_PROGRAM for known terminals
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            // iTerm2 uses its own inline images protocol
            "iTerm.app" => return ImageRenderMethod::ITerm2,
            // WezTerm supports both Sixel and Kitty, prefer Sixel
            "WezTerm" => return ImageRenderMethod::Sixel,
            // mlterm supports Sixel
            "mlterm" => return ImageRenderMethod::Sixel,
            _ => {}
        }
    }
    
    // Check TERM for xterm with Sixel support
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("xterm") || term.contains("foot") || term.contains("contour") {
            // These terminals may support Sixel
            return ImageRenderMethod::Sixel;
        }
    }
    
    // Use Unicode blocks if true color is supported
    if supports_true_color() {
        return ImageRenderMethod::UnicodeBlocks;
    }
    
    // Fallback to ASCII
    ImageRenderMethod::Ascii
}

/// Load image with automatic method detection
pub fn load_image_auto(path: &Path, max_width: u32, max_height: u32) -> ImagePreview {
    let method = detect_render_method();
    
    match method {
        ImageRenderMethod::Ascii => load_image_ascii(path, max_width, max_height),
        ImageRenderMethod::UnicodeBlocks => load_image_unicode(path, max_width, max_height),
        ImageRenderMethod::Sixel => load_image_sixel(path, max_width, max_height),
        ImageRenderMethod::Kitty => load_image_kitty(path, max_width, max_height),
        ImageRenderMethod::ITerm2 => load_image_iterm2(path, max_width, max_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gray_to_ascii() {
        assert_eq!(gray_to_ascii(0), ' ');
        assert_eq!(gray_to_ascii(255), '@');
    }
    
    #[test]
    fn test_supports_true_color() {
        // Should not panic
        let _ = supports_true_color();
    }
}

