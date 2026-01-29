//! Integration tests for PNG rendering.

#![cfg(feature = "png")]

use hyper_render::{render, render_to_png, Config, OutputFormat};

/// PNG header magic bytes
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Parse PNG dimensions from the IHDR chunk.
/// Returns (width, height) or None if parsing fails.
fn parse_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // PNG structure: 8-byte signature, then chunks
    // IHDR is always the first chunk after signature
    // Chunk format: 4-byte length, 4-byte type, data, 4-byte CRC
    if data.len() < 24 {
        return None;
    }

    // Skip signature (8 bytes) and length (4 bytes)
    // Check for IHDR type
    if &data[12..16] != b"IHDR" {
        return None;
    }

    // Width and height are the first 8 bytes of IHDR data (big-endian u32s)
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

    Some((width, height))
}

#[test]
fn test_png_basic_render() {
    let html = "<html><body><h1>Hello</h1></body></html>";
    let config = Config::new().width(400).height(300);

    let result = render(html, config);
    assert!(result.is_ok(), "render should succeed");

    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "output should not be empty");
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_render_to_png_convenience() {
    let html = "<html><body><p>Test</p></body></html>";
    let config = Config::new();

    let result = render_to_png(html, config);
    assert!(result.is_ok(), "render_to_png should succeed");

    let bytes = result.unwrap();
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_dimensions_match_config() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(640).height(480).scale(1.0);

    let bytes = render(html, config).expect("render should succeed");
    let (width, height) = parse_png_dimensions(&bytes).expect("should parse PNG dimensions");

    assert_eq!(width, 640, "PNG width should match config");
    assert_eq!(height, 480, "PNG height should match config");
}

#[test]
fn test_png_scale_factor_doubles_dimensions() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(100).height(100).scale(2.0);

    let bytes = render(html, config).expect("render should succeed");
    let (width, height) = parse_png_dimensions(&bytes).expect("should parse PNG dimensions");

    // With scale=2.0, output dimensions should be doubled
    assert_eq!(width, 200, "PNG width should be 2x config width");
    assert_eq!(height, 200, "PNG height should be 2x config height");
}

#[test]
fn test_png_auto_height() {
    // Create HTML with content that should be taller than 100px
    let html = r#"
        <html>
        <body style="margin: 0; padding: 0;">
            <div style="height: 500px; background: red;"></div>
        </body>
        </html>
    "#;

    let config = Config::new().width(200).height(100).auto_height(true);

    let bytes = render(html, config).expect("render should succeed");
    let (_, height) = parse_png_dimensions(&bytes).expect("should parse PNG dimensions");

    // With auto_height, the output should be taller than the configured 100px
    assert!(
        height > 100,
        "auto_height should produce taller output: got {}",
        height
    );
}

#[test]
fn test_png_transparent_background() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(10).height(10).transparent();

    let result = render(html, config);
    assert!(result.is_ok(), "transparent background should work");

    let bytes = result.unwrap();
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_custom_background() {
    let html = "<html><body></body></html>";
    let config = Config::new()
        .width(10)
        .height(10)
        .background([255, 0, 0, 255]); // Red

    let result = render(html, config);
    assert!(result.is_ok(), "custom background should work");

    let bytes = result.unwrap();
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_format_enum() {
    let html = "<html><body></body></html>";
    let config = Config::new().format(OutputFormat::Png);

    let bytes = render(html, config).expect("render should succeed");
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "OutputFormat::Png should produce PNG"
    );
}

#[test]
fn test_png_unicode_content() {
    let html = r#"<html><body><p>Hello 世界 🌍</p></body></html>"#;
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "unicode content should render");

    let bytes = result.unwrap();
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_styled_content() {
    let html = r#"
        <html>
        <body style="background: #f0f0f0; padding: 20px;">
            <div style="background: white; border-radius: 8px; padding: 16px;">
                <h1 style="color: navy;">Styled Content</h1>
                <p style="color: #666;">With CSS styling</p>
            </div>
        </body>
        </html>
    "#;
    let config = Config::new().width(400).height(300);

    let result = render(html, config);
    assert!(result.is_ok(), "styled content should render");

    let bytes = result.unwrap();
    assert!(
        bytes.starts_with(&PNG_SIGNATURE),
        "output should be valid PNG"
    );
}

#[test]
fn test_png_minimum_dimensions() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(1).height(1);

    let result = render(html, config);
    assert!(result.is_ok(), "minimum dimensions should work");

    let bytes = result.unwrap();
    let (width, height) = parse_png_dimensions(&bytes).expect("should parse PNG dimensions");
    assert_eq!(width, 1);
    assert_eq!(height, 1);
}
