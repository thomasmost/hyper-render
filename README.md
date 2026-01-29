# hyper-render

A Chromium-free HTML rendering engine for generating PNG and PDF outputs in pure Rust.

## Features

- **No browser required** — Pure Rust implementation, no Chromium/WebKit dependency
- **PNG output** — High-quality raster images via CPU-based rendering
- **PDF output** — Vector PDF documents with embedded fonts
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
| `OutputFormat::Pdf` | ✅ Full | Vector PDF with embedded fonts and backgrounds |

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

## Performance

### Benchmarks

Run benchmarks locally:

```bash
# Full benchmark suite
cargo bench --all-features

# Specific benchmark group
cargo bench full_pipeline

# Generate HTML reports
cargo bench --all-features  # Reports saved to target/criterion/
```

#### Sample Results (M1 MacBook Pro)

| Benchmark | Time |
|-----------|------|
| PNG render (small HTML) | ~10 ms |
| PNG render (medium HTML) | ~13 ms |
| PNG render (large HTML, 50 items) | ~13 ms |
| PDF render (small HTML) | ~9 ms |
| PDF render (medium HTML) | ~9 ms |
| PDF render (large HTML, 50 items) | ~12 ms |

**Scaling impact (800x600 base):**
| Scale | Time | Pixels |
|-------|------|--------|
| 1x | ~13 ms | 480K |
| 2x | ~21 ms | 1.9M |
| 3x | ~32 ms | 4.3M |

**Dimension impact:**
| Size | Time | Throughput |
|------|------|------------|
| 400x300 | ~11 ms | 10M px/s |
| 800x600 | ~13 ms | 35M px/s |
| 1920x1080 | ~18 ms | 115M px/s |

### Build Size

| Features | Library Size |
|----------|--------------|
| none | 250 KB |
| png | 544 KB |
| pdf | 513 KB |
| png,pdf (default) | 767 KB |

Check detailed size breakdown:

```bash
cargo install cargo-bloat
cargo bloat --release --all-features -n 20
```

### Performance Characteristics

- **PNG rendering** scales with pixel count: O(width × height × scale²)
- **PDF rendering** scales with DOM complexity: O(nodes × text_length)
- **HTML parsing** is generally fast; layout (Stylo/Taffy) dominates small documents
- **Font caching** in PDF reduces repeated text rendering overhead

## Known Issues

### HTML Parser Warnings

You may see `ERROR: Unexpected token` messages when rendering HTML with non-standard CSS properties (e.g., `mso-font-alt` for Microsoft Office). These warnings come from the HTML parser and **do not affect rendering** — the output is still generated correctly.

To suppress these warnings, redirect stderr:
```bash
cargo run --example from_file -- input.html output.png 2>/dev/null
```

## Limitations

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
