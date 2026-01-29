//! Simple example demonstrating basic HTML rendering.
//!
//! Run with: `cargo run --example simple`

use hyper_render::{render, Config, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {
                    font-family: system-ui, -apple-system, sans-serif;
                    padding: 40px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                    margin: 0;
                }
                .card {
                    background: white;
                    border-radius: 16px;
                    padding: 32px;
                    max-width: 500px;
                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                }
                h1 {
                    color: #1a1a2e;
                    margin: 0 0 16px 0;
                    font-size: 28px;
                }
                p {
                    color: #4a4a68;
                    line-height: 1.6;
                    margin: 0 0 24px 0;
                }
                .badge {
                    display: inline-block;
                    background: #667eea;
                    color: white;
                    padding: 8px 16px;
                    border-radius: 20px;
                    font-size: 14px;
                    font-weight: 500;
                }
            </style>
        </head>
        <body>
            <div class="card">
                <h1>Hello, hyper-render!</h1>
                <p>
                    This HTML was rendered to an image without using Chromium.
                    The rendering is powered by Blitz, Stylo, and Taffy.
                </p>
                <span class="badge">Pure Rust</span>
            </div>
        </body>
        </html>
    "#;

    // Render to PNG
    println!("Rendering to PNG...");
    let png_config = Config::new().size(800, 600).scale(2.0); // 2x for retina quality

    let png_bytes = render(html, png_config)?;
    std::fs::write("output.png", &png_bytes)?;
    println!("Saved output.png ({} bytes)", png_bytes.len());

    // Render to PDF
    println!("Rendering to PDF...");
    let pdf_config = Config::new().size(800, 600).format(OutputFormat::Pdf);

    let pdf_bytes = render(html, pdf_config)?;
    std::fs::write("output.pdf", &pdf_bytes)?;
    println!("Saved output.pdf ({} bytes)", pdf_bytes.len());

    println!("Done!");
    Ok(())
}
