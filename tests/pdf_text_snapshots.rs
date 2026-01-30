//! Text snapshot tests for PDF rendering.
//!
//! These tests verify that HTML renders to PDF with the correct text content
//! by extracting text from the generated PDF and comparing against snapshots.
//!
//! To update snapshots, run: `UPDATE_SNAPSHOTS=1 cargo test --test pdf_text_snapshots`

#![cfg(feature = "pdf")]

use hyper_render::{render, Config, OutputFormat};
use std::path::Path;

/// Extract text content from PDF bytes.
fn extract_pdf_text(pdf_bytes: &[u8]) -> String {
    pdf_extract::extract_text_from_mem(pdf_bytes)
        .unwrap_or_else(|e| panic!("Failed to extract text from PDF: {}", e))
}

/// Normalize text for comparison (trim whitespace, normalize line endings).
fn normalize_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Load or update a snapshot file.
fn check_snapshot(name: &str, actual: &str) {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{}.txt", name));

    let normalized_actual = normalize_text(actual);

    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&snapshot_path, &normalized_actual).unwrap_or_else(|e| {
            panic!(
                "Failed to write snapshot {}: {}",
                snapshot_path.display(),
                e
            )
        });
        println!("Updated snapshot: {}", snapshot_path.display());
        return;
    }

    if !snapshot_path.exists() {
        panic!(
            "Snapshot file does not exist: {}\n\
             Run with UPDATE_SNAPSHOTS=1 to create it.\n\
             Actual text:\n{}",
            snapshot_path.display(),
            normalized_actual
        );
    }

    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|e| panic!("Failed to read snapshot {}: {}", snapshot_path.display(), e));
    let normalized_expected = normalize_text(&expected);

    if normalized_actual != normalized_expected {
        panic!(
            "Snapshot mismatch for '{}':\n\n\
             === Expected ===\n{}\n\n\
             === Actual ===\n{}\n\n\
             Run with UPDATE_SNAPSHOTS=1 to update the snapshot.",
            name, normalized_expected, normalized_actual
        );
    }
}

/// Render HTML to PDF and extract text.
fn render_and_extract(html: &str) -> String {
    let config = Config::new()
        .width(800)
        .height(600)
        .format(OutputFormat::Pdf);

    let pdf_bytes = render(html, config).expect("Failed to render PDF");
    extract_pdf_text(&pdf_bytes)
}

// =============================================================================
// Test Cases
// =============================================================================

#[test]
fn test_simple_paragraph() {
    let html = r#"
        <html>
        <body>
            <p>Hello, World!</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("simple_paragraph", &text);
}

#[test]
fn test_multiple_paragraphs() {
    let html = r#"
        <html>
        <body>
            <p>First paragraph.</p>
            <p>Second paragraph.</p>
            <p>Third paragraph.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("multiple_paragraphs", &text);
}

#[test]
fn test_headings() {
    let html = r#"
        <html>
        <body>
            <h1>Main Title</h1>
            <h2>Subtitle</h2>
            <h3>Section Header</h3>
            <p>Body text.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("headings", &text);
}

#[test]
fn test_nested_elements() {
    let html = r#"
        <html>
        <body>
            <div>
                <p>Outer paragraph.</p>
                <div>
                    <p>Inner paragraph.</p>
                    <span>Inline text.</span>
                </div>
            </div>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("nested_elements", &text);
}

#[test]
fn test_styled_text() {
    let html = r#"
        <html>
        <body>
            <p><strong>Bold text</strong> and <em>italic text</em>.</p>
            <p>Normal text with <span style="color: red;">colored</span> word.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("styled_text", &text);
}

#[test]
fn test_list_items() {
    let html = r#"
        <html>
        <body>
            <ul>
                <li>First item</li>
                <li>Second item</li>
                <li>Third item</li>
            </ul>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("list_items", &text);
}

#[test]
fn test_ordered_list() {
    let html = r#"
        <html>
        <body>
            <ol>
                <li>Step one</li>
                <li>Step two</li>
                <li>Step three</li>
            </ol>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("ordered_list", &text);
}

#[test]
fn test_table_content() {
    let html = r#"
        <html>
        <body>
            <table>
                <tr>
                    <th>Name</th>
                    <th>Value</th>
                </tr>
                <tr>
                    <td>Item A</td>
                    <td>100</td>
                </tr>
                <tr>
                    <td>Item B</td>
                    <td>200</td>
                </tr>
            </table>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("table_content", &text);
}

#[test]
fn test_special_characters() {
    let html = r#"
        <html>
        <body>
            <p>Special: &amp; &lt; &gt; &quot;</p>
            <p>Unicode: Hello</p>
            <p>Symbols: $100 + 50% = profit</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("special_characters", &text);
}

#[test]
fn test_multiline_content() {
    let html = r#"
        <html>
        <body>
            <p>Line one of the paragraph that contains
            multiple lines of text that should be
            rendered properly in the PDF output.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("multiline_content", &text);
}

// Skip on Linux due to platform-specific font rendering differences affecting PDF text extraction.
// Linux renders list items without spacing between them ("Point APoint B") while macOS
// includes spacing ("Point A Point B"). This is a pdf-extract behavior difference, not a bug.
#[test]
#[cfg_attr(target_os = "linux", ignore)]
fn test_mixed_content() {
    let html = r#"
        <html>
        <body>
            <h1>Document Title</h1>
            <p>Introduction paragraph with some text.</p>
            <h2>Section One</h2>
            <ul>
                <li>Point A</li>
                <li>Point B</li>
            </ul>
            <h2>Section Two</h2>
            <p>Conclusion paragraph.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("mixed_content", &text);
}

#[test]
fn test_whitespace_handling() {
    let html = r#"
        <html>
        <body>
            <p>Normal   spacing   here.</p>
            <pre>Preformatted    text.</pre>
            <p>Back to normal.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("whitespace_handling", &text);
}

#[test]
fn test_empty_elements() {
    let html = r#"
        <html>
        <body>
            <p>Before empty.</p>
            <div></div>
            <p></p>
            <p>After empty.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("empty_elements", &text);
}

#[test]
fn test_inline_elements() {
    let html = r#"
        <html>
        <body>
            <p>Text with <b>bold</b>, <i>italic</i>, <u>underline</u>, and <code>code</code>.</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("inline_elements", &text);
}

#[test]
fn test_receipt_like_document() {
    let html = r#"
        <html>
        <body style="font-family: sans-serif;">
            <h1>RECEIPT</h1>
            <p>Order #12345</p>
            <p>Date: 2024-01-15</p>
            <hr>
            <table>
                <tr><td>Widget</td><td>$10.00</td></tr>
                <tr><td>Gadget</td><td>$25.00</td></tr>
                <tr><td>Service Fee</td><td>$5.00</td></tr>
            </table>
            <hr>
            <p><strong>Total: $40.00</strong></p>
            <p>Thank you for your purchase!</p>
        </body>
        </html>
    "#;

    let text = render_and_extract(html);
    check_snapshot("receipt_like_document", &text);
}
