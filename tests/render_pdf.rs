//! Integration tests for PDF rendering.

#![cfg(feature = "pdf")]

use hyper_render::{render, render_to_pdf, Config, OutputFormat};

/// PDF magic bytes
const PDF_SIGNATURE: &[u8] = b"%PDF-";

/// Check if bytes represent a valid PDF by looking for key markers.
fn is_valid_pdf(data: &[u8]) -> bool {
    // Check header
    if !data.starts_with(PDF_SIGNATURE) {
        return false;
    }

    // Check for EOF marker (should be near the end)
    let tail = if data.len() > 1024 {
        &data[data.len() - 1024..]
    } else {
        data
    };

    // PDF files end with %%EOF
    tail.windows(5).any(|w| w == b"%%EOF")
}

/// Search for a string pattern in the PDF bytes.
fn pdf_contains(data: &[u8], pattern: &[u8]) -> bool {
    data.windows(pattern.len()).any(|w| w == pattern)
}

#[test]
fn test_pdf_basic_render() {
    let html = "<html><body><h1>Hello</h1></body></html>";
    let config = Config::new().format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "render should succeed");

    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "output should not be empty");
    assert!(
        bytes.starts_with(PDF_SIGNATURE),
        "output should start with PDF magic bytes"
    );
    assert!(is_valid_pdf(&bytes), "output should be valid PDF structure");
}

#[test]
fn test_pdf_render_to_pdf_convenience() {
    let html = "<html><body><p>Test</p></body></html>";
    let config = Config::new();

    let result = render_to_pdf(html, config);
    assert!(result.is_ok(), "render_to_pdf should succeed");

    let bytes = result.unwrap();
    assert!(is_valid_pdf(&bytes), "output should be valid PDF");
}

#[test]
fn test_pdf_contains_page_dimensions() {
    let html = "<html><body></body></html>";
    let config = Config::new()
        .width(800)
        .height(600)
        .format(OutputFormat::Pdf);

    let bytes = render(html, config).expect("render should succeed");

    // PDF should contain MediaBox with dimensions
    // The exact format varies but we check for common patterns
    assert!(
        pdf_contains(&bytes, b"MediaBox") || pdf_contains(&bytes, b"/MediaBox"),
        "PDF should contain page dimensions"
    );
}

#[test]
fn test_pdf_auto_height() {
    let html = r#"
        <html>
        <body style="margin: 0;">
            <div style="height: 1000px;"></div>
        </body>
        </html>
    "#;

    let config = Config::new()
        .width(400)
        .height(100)
        .auto_height(true)
        .format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "auto_height PDF should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_with_text_content() {
    let html = r#"<html><body><p>Hello World</p></body></html>"#;
    let config = Config::new().format(OutputFormat::Pdf);

    let bytes = render(html, config).expect("render should succeed");

    // PDF should contain font information (text rendering)
    assert!(
        pdf_contains(&bytes, b"/Font") || pdf_contains(&bytes, b"Font"),
        "PDF with text should contain font references"
    );
}

#[test]
fn test_pdf_background_colors() {
    let html = r#"
        <html>
        <body style="background: #ff0000;">
            <div style="background: blue; width: 100px; height: 100px;"></div>
        </body>
        </html>
    "#;
    let config = Config::new().format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "PDF with backgrounds should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_custom_page_background() {
    let html = "<html><body></body></html>";
    let config = Config::new()
        .format(OutputFormat::Pdf)
        .background([200, 200, 200, 255]); // Gray background

    let result = render(html, config);
    assert!(result.is_ok(), "custom background should work");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_unicode_content() {
    let html = r#"<html><body><p>Unicode: 日本語 中文 한국어</p></body></html>"#;
    let config = Config::new().format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "unicode content should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_styled_content() {
    let html = r#"
        <html>
        <body style="font-family: sans-serif; padding: 20px;">
            <h1 style="color: navy;">Document Title</h1>
            <p style="color: #333; line-height: 1.5;">
                This is a styled paragraph with various CSS properties.
            </p>
            <ul>
                <li>Item one</li>
                <li>Item two</li>
            </ul>
        </body>
        </html>
    "#;
    let config = Config::new()
        .width(600)
        .height(400)
        .format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "styled content should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_nested_elements() {
    let html = r#"
        <html>
        <body>
            <div style="padding: 10px;">
                <div style="padding: 10px;">
                    <div style="padding: 10px;">
                        <p>Deeply nested</p>
                    </div>
                </div>
            </div>
        </body>
        </html>
    "#;
    let config = Config::new().format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "nested elements should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_flexbox_layout() {
    let html = r#"
        <html>
        <body>
            <div style="display: flex; gap: 10px;">
                <div style="flex: 1; background: red;">A</div>
                <div style="flex: 1; background: green;">B</div>
                <div style="flex: 1; background: blue;">C</div>
            </div>
        </body>
        </html>
    "#;
    let config = Config::new()
        .width(400)
        .height(200)
        .format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "flexbox layout should render");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}

#[test]
fn test_pdf_minimum_dimensions() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(1).height(1).format(OutputFormat::Pdf);

    let result = render(html, config);
    assert!(result.is_ok(), "minimum dimensions should work");
    assert!(is_valid_pdf(&result.unwrap()), "output should be valid PDF");
}
