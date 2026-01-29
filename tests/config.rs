//! Integration tests for configuration options.

use hyper_render::{render, ColorScheme, Config, OutputFormat};

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert_eq!(config.scale, 1.0);
    assert_eq!(config.format, OutputFormat::Png);
    assert_eq!(config.color_scheme, ColorScheme::Light);
    assert!(!config.auto_height);
    assert_eq!(config.background, [255, 255, 255, 255]);
}

#[test]
fn test_config_new_equals_default() {
    let new = Config::new();
    let default = Config::default();

    assert_eq!(new.width, default.width);
    assert_eq!(new.height, default.height);
    assert_eq!(new.scale, default.scale);
}

#[test]
fn test_config_builder_chaining() {
    let config = Config::new()
        .width(1920)
        .height(1080)
        .scale(2.0)
        .format(OutputFormat::Pdf)
        .color_scheme(ColorScheme::Dark)
        .auto_height(true)
        .background([100, 100, 100, 255]);

    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.scale, 2.0);
    assert_eq!(config.format, OutputFormat::Pdf);
    assert_eq!(config.color_scheme, ColorScheme::Dark);
    assert!(config.auto_height);
    assert_eq!(config.background, [100, 100, 100, 255]);
}

#[test]
fn test_config_size_convenience() {
    let config = Config::new().size(1280, 720);

    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 720);
}

#[test]
fn test_config_transparent_convenience() {
    let config = Config::new().transparent();

    assert_eq!(config.background, [0, 0, 0, 0]);
}

#[test]
fn test_config_validate_valid() {
    let config = Config::new();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_edge_cases() {
    // Minimum valid dimensions
    assert!(Config::new()
        .width(1)
        .height(1)
        .scale(0.001)
        .validate()
        .is_ok());

    // Large dimensions
    assert!(Config::new().width(10000).height(10000).validate().is_ok());

    // Large scale
    assert!(Config::new().scale(100.0).validate().is_ok());
}

#[test]
fn test_output_format_png_rendering() {
    let html = "<p>Test</p>";
    let config = Config::new().format(OutputFormat::Png);

    let bytes = render(html, config).expect("should render");
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[test]
fn test_output_format_pdf_rendering() {
    let html = "<p>Test</p>";
    let config = Config::new().format(OutputFormat::Pdf);

    let bytes = render(html, config).expect("should render");
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_output_format_display() {
    assert_eq!(format!("{}", OutputFormat::Png), "png");
    assert_eq!(format!("{}", OutputFormat::Pdf), "pdf");
}

#[test]
fn test_color_scheme_light() {
    let html = "<p>Test</p>";
    let config = Config::new().color_scheme(ColorScheme::Light);

    let result = render(html, config);
    assert!(result.is_ok(), "light color scheme should work");
}

#[test]
fn test_color_scheme_dark() {
    let html = "<p>Test</p>";
    let config = Config::new().color_scheme(ColorScheme::Dark);

    let result = render(html, config);
    assert!(result.is_ok(), "dark color scheme should work");
}

#[test]
fn test_scale_factors() {
    let html = "<p>Test</p>";

    // Various scale factors
    for scale in [0.5, 1.0, 1.5, 2.0, 3.0] {
        let config = Config::new().scale(scale);
        let result = render(html, config);
        assert!(result.is_ok(), "scale {} should work", scale);
    }
}

#[test]
fn test_background_colors() {
    let html = "<p>Test</p>";

    let backgrounds = [
        [0, 0, 0, 255],       // Black
        [255, 255, 255, 255], // White
        [255, 0, 0, 255],     // Red
        [0, 255, 0, 255],     // Green
        [0, 0, 255, 255],     // Blue
        [0, 0, 0, 0],         // Transparent
        [128, 128, 128, 128], // Semi-transparent gray
    ];

    for bg in backgrounds {
        let config = Config::new().background(bg);
        let result = render(html, config);
        assert!(result.is_ok(), "background {:?} should work", bg);
    }
}

#[test]
fn test_various_dimensions() {
    let html = "<p>Test</p>";

    let dimensions = [
        (1, 1),
        (10, 10),
        (100, 100),
        (800, 600),
        (1920, 1080),
        (100, 1000), // Tall
        (1000, 100), // Wide
    ];

    for (w, h) in dimensions {
        let config = Config::new().width(w).height(h);
        let result = render(html, config);
        assert!(result.is_ok(), "dimensions {}x{} should work", w, h);
    }
}

#[test]
fn test_config_immutability() {
    // Builder methods return new config, don't mutate
    let config1 = Config::new();
    let config2 = config1.clone().width(1000);

    assert_eq!(config1.width, 800); // Original unchanged
    assert_eq!(config2.width, 1000);
}

#[test]
fn test_config_debug() {
    let config = Config::new();
    let debug = format!("{:?}", config);

    // Should contain field names
    assert!(debug.contains("width"));
    assert!(debug.contains("height"));
    assert!(debug.contains("scale"));
}

#[test]
fn test_output_format_default() {
    let format = OutputFormat::default();
    assert_eq!(format, OutputFormat::Png);
}

#[test]
fn test_color_scheme_default() {
    let scheme = ColorScheme::default();
    assert_eq!(scheme, ColorScheme::Light);
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Png, OutputFormat::Png);
    assert_eq!(OutputFormat::Pdf, OutputFormat::Pdf);
    assert_ne!(OutputFormat::Png, OutputFormat::Pdf);
}

#[test]
fn test_color_scheme_equality() {
    assert_eq!(ColorScheme::Light, ColorScheme::Light);
    assert_eq!(ColorScheme::Dark, ColorScheme::Dark);
    assert_ne!(ColorScheme::Light, ColorScheme::Dark);
}
