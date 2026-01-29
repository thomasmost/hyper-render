# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-29

### Added

- Initial release of hyper-render
- PNG rendering via Vello CPU rasterizer
- PDF rendering via Krilla with vector graphics
- HTML/CSS parsing using Blitz (html5ever + Stylo)
- Flexbox and Grid layout support via Taffy
- Configuration options:
  - Viewport dimensions (width, height)
  - Scale factor for HiDPI displays
  - Output format selection (PNG/PDF)
  - Color scheme preference (light/dark)
  - Auto-height detection for content
  - Custom background colors with transparency support
- Feature flags for optional PNG and PDF support
- Comprehensive error handling with descriptive messages
- Configuration validation (dimensions, scale)
- Font embedding in PDF output
- Background color rendering for all elements

### Known Limitations

- No JavaScript support (by design)
- System fonts only (`@font-face` not yet supported)
- External image loading not yet implemented
- HTML parser may emit warnings for non-standard CSS properties
