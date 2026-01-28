# hyper-render

A Chromium-free HTML rendering engine for generating PNG and PDF outputs in pure Rust.

## Features

- **No browser required** — Pure Rust implementation, no Chromium/WebKit dependency
- **PNG output** — High-quality raster images via CPU-based rendering
- **PDF output** — Vector PDF documents (work in progress)
- **Modern CSS** — Flexbox, Grid, and common CSS properties via Stylo (Firefox's CSS engine)
- **Simple API** — Single function call to render HTML to bytes

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hyper-render = "0.1"
```

Or with specific features:

```toml
[dependencies]
hyper-render = { version = "0.1", default-features = false, features = ["png"] }
```

## Quick Start

```rust
use hyper_render::{render, Config, OutputFormat};

fn main() -> Result<(), hyper_render::Error> {
    let html = r#"
        <html>
        <body style="font-family: sans-serif; padding: 20px;">
            <h1 style="color: navy;">Hello, World!</h1>
            <p>Rendered without Chromium.</p>
        </body>
        </html>
    "#;

    // Render to PNG
    let png_bytes = render(html, Config::default())?;
    std::fs::write("output.png", png_bytes)?;

    // Render to PDF
    let pdf_bytes = render(html, Config::new().format(OutputFormat::Pdf))?;
    std::fs::write("output.pdf", pdf_bytes)?;

    Ok(())
}
```

## API

### Main Functions

```rust
// Render to any format based on config
render(html: &str, config: Config) -> Result<Vec<u8>>

// Convenience functions
render_to_png(html: &str, config: Config) -> Result<Vec<u8>>
render_to_pdf(html: &str, config: Config) -> Result<Vec<u8>>
```

### Configuration

```rust
use hyper_render::{Config, OutputFormat, ColorScheme};

let config = Config::new()
    .width(1200)              // Viewport width in pixels
    .height(800)              // Viewport height in pixels
    .size(1200, 800)          // Set both at once
    .scale(2.0)               // Scale factor (2.0 for retina)
    .format(OutputFormat::Png) // Output format: Png or Pdf
    .color_scheme(ColorScheme::Light) // Light or Dark mode
    .auto_height(true)        // Auto-detect content height
    .background([255, 255, 255, 255]) // RGBA background color
    .transparent();           // Transparent background
```

### Output Formats

| Format | Status | Description |
|--------|--------|-------------|
| `OutputFormat::Png` | ✅ Full | Raster image via Vello CPU renderer |
| `OutputFormat::Pdf` | ⚠️ WIP | Vector PDF (dimensions only, content rendering in progress) |

## Try It Yourself

### Clone and Build

```bash
git clone https://github.com/versa-protocol/hyper-render
cd hyper-render
cargo build
```

### Run the Example

```bash
cargo run --example simple
```

This generates `output.png` and `output.pdf` in the current directory.

### Render Your Own HTML

```bash
# Create a test HTML file
echo '<html><body style="background: #667eea; color: white; padding: 40px; font-family: system-ui;"><h1>My Page</h1><p>Hello from hyper-render!</p></body></html>' > test.html

# Render to PNG
cargo run --example from_file -- test.html output.png

# Render to PDF
cargo run --example from_file -- test.html output.pdf
```

### Run Tests

```bash
cargo test
```

### Generate Documentation

```bash
cargo doc --open
```

## Architecture

hyper-render composes several best-in-class Rust crates:

```
HTML String
    ↓
html5ever (HTML parsing)
    ↓
Stylo (CSS parsing & cascade - from Firefox)
    ↓
Taffy (Flexbox/Grid layout)
    ↓
Blitz (DOM + rendering coordination)
    ↓
┌─────────────────┬─────────────────┐
│   Vello CPU     │     Krilla      │
│  (rasterizer)   │  (PDF writer)   │
└────────┬────────┴────────┬────────┘
         ↓                 ↓
       PNG               PDF
```

## Limitations

- **PDF text rendering** — Not yet implemented; PDFs currently have correct dimensions but no content
- **JavaScript** — Not supported (by design)
- **Web fonts** — System fonts only; `@font-face` not yet supported
- **Images** — External image loading not yet implemented
- **Some CSS** — Advanced features like `position: sticky`, complex transforms may not work

## Dependencies

Core rendering stack:
- [Blitz](https://github.com/DioxusLabs/blitz) — HTML/CSS rendering engine
- [Stylo](https://github.com/servo/stylo) — Firefox's CSS engine
- [Taffy](https://github.com/DioxusLabs/taffy) — Flexbox/Grid layout
- [Vello](https://github.com/linebender/vello) — 2D graphics (CPU renderer)
- [Krilla](https://github.com/LaurenzV/krilla) — PDF generation

## License

MIT OR Apache-2.0
