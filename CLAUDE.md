# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

hyper-render is a Chromium-free HTML rendering engine in pure Rust that generates PNG and PDF outputs. It uses Firefox's Stylo for CSS and Taffy for flexbox/grid layout.

## Build Commands

```bash
cargo build              # Development build
cargo build --release    # Release build
cargo test               # Run all tests (unit, integration, doc)
cargo test render_png    # Run specific test file
cargo clippy             # Lint
cargo fmt                # Format
cargo run --example simple                           # Run basic demo
cargo run --example from_file -- input.html out.png  # Render file to PNG/PDF
```

## Feature Flags

Both formats enabled by default. Build with specific features:
```bash
cargo build --no-default-features --features png  # PNG only
cargo build --no-default-features --features pdf  # PDF only
```

## Architecture

```
HTML → html5ever (parse) → Stylo (CSS cascade) → Taffy (layout) → Blitz (DOM)
                                                                      ↓
                                                    ┌─────────────────┴─────────────────┐
                                                    │                                   │
                                              Vello CPU (PNG)                    Krilla (PDF)
```

### Key Modules

- `src/lib.rs` - Public API: `render()`, `render_to_png()`, `render_to_pdf()`
- `src/config.rs` - Builder-pattern `Config` struct (width, height, scale, format, color scheme, auto_height)
- `src/render/png.rs` - Vello CPU rasterization with PNG encoding
- `src/render/pdf.rs` - Vector PDF generation via Krilla with font embedding

### PDF Rendering Notes

PDF rendering (`src/render/pdf.rs`) involves coordinate system transformation (PDF origin is bottom-left, web is top-left). The implementation:
- Recursively traverses the DOM for backgrounds and text
- Caches fonts by ID to avoid redundant processing
- Extracts colors from Stylo computed styles with sRGB conversion
- Handles glyph clustering for ligatures

## Testing

Integration tests are in `tests/`:
- `tests/render_png.rs` - PNG output validation (headers, dimensions, scaling)
- `tests/render_pdf.rs` - PDF output validation (magic bytes, structure)
- `tests/error_handling.rs` - Error conditions and edge cases
- `tests/config.rs` - Configuration combinations

Tests validate actual output (PNG headers, PDF structure) rather than just smoke testing.

## Current Limitations

- No JavaScript support (by design)
- System fonts only (`@font-face` not yet supported)
- External images not yet implemented
- HTML parser emits stderr warnings for non-standard CSS (e.g., `mso-font-alt`) but rendering works
