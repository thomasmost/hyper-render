//! Integration tests for error handling.

#![cfg(feature = "png")]

use hyper_render::{render, Config, Error};

#[test]
fn test_empty_html() {
    let html = "";
    let config = Config::new();

    // Empty HTML should still render (produces blank output)
    let result = render(html, config);
    assert!(result.is_ok(), "empty HTML should render gracefully");
}

#[test]
fn test_whitespace_only_html() {
    let html = "   \n\t  ";
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "whitespace-only HTML should render");
}

#[test]
fn test_minimal_html() {
    let html = "<p>x</p>";
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "minimal HTML should render");
}

#[test]
fn test_malformed_html() {
    // Missing closing tags
    let html = "<html><body><div><p>unclosed";
    let config = Config::new();

    // HTML parser should handle malformed HTML gracefully
    let result = render(html, config);
    assert!(result.is_ok(), "malformed HTML should render gracefully");
}

#[test]
fn test_deeply_nested_html() {
    let mut html = String::from("<html><body>");
    for _ in 0..50 {
        html.push_str("<div>");
    }
    html.push_str("Content");
    for _ in 0..50 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    let config = Config::new();
    let result = render(&html, config);
    assert!(result.is_ok(), "deeply nested HTML should render");
}

#[test]
fn test_invalid_config_zero_width() {
    let html = "<html><body></body></html>";
    let config = Config::new().width(0);

    let result = render(html, config);
    assert!(result.is_err(), "zero width should error");

    let err = result.unwrap_err();
    assert!(
        matches!(err, Error::InvalidConfig(_)),
        "should be InvalidConfig error"
    );
}

#[test]
fn test_invalid_config_zero_height() {
    let html = "<html><body></body></html>";
    let config = Config::new().height(0);

    let result = render(html, config);
    assert!(result.is_err(), "zero height should error");

    let err = result.unwrap_err();
    assert!(
        matches!(err, Error::InvalidConfig(_)),
        "should be InvalidConfig error"
    );
}

#[test]
fn test_invalid_config_zero_scale() {
    let html = "<html><body></body></html>";
    let config = Config::new().scale(0.0);

    let result = render(html, config);
    assert!(result.is_err(), "zero scale should error");
}

#[test]
fn test_invalid_config_negative_scale() {
    let html = "<html><body></body></html>";
    let config = Config::new().scale(-1.0);

    let result = render(html, config);
    assert!(result.is_err(), "negative scale should error");
}

#[test]
fn test_invalid_config_infinite_scale() {
    let html = "<html><body></body></html>";
    let config = Config::new().scale(f32::INFINITY);

    let result = render(html, config);
    assert!(result.is_err(), "infinite scale should error");
}

#[test]
fn test_invalid_config_nan_scale() {
    let html = "<html><body></body></html>";
    let config = Config::new().scale(f32::NAN);

    let result = render(html, config);
    assert!(result.is_err(), "NaN scale should error");
}

#[test]
fn test_error_display_invalid_config() {
    let config = Config::new().width(0);
    let err = config.validate().unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("width") || msg.contains("config"),
        "error message should be descriptive: {}",
        msg
    );
}

#[test]
fn test_html_with_special_characters() {
    let html = r#"<html><body><p>&lt;script&gt;alert('xss')&lt;/script&gt;</p></body></html>"#;
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "HTML with special characters should render");
}

#[test]
fn test_html_with_comments() {
    let html = r#"
        <html>
        <!-- This is a comment -->
        <body>
            <!-- Another comment -->
            <p>Content</p>
        </body>
        </html>
    "#;
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "HTML with comments should render");
}

#[test]
fn test_html_with_doctype() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body><p>Content</p></body>
        </html>
    "#;
    let config = Config::new();

    let result = render(html, config);
    assert!(result.is_ok(), "HTML with DOCTYPE should render");
}
