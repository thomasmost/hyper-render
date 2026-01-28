//! Example showing how to render HTML from a file.
//!
//! Run with: `cargo run --example from_file -- input.html output.png`
//! Or for PDF: `cargo run --example from_file -- input.html output.pdf`

use hyper_render::{render, Config, OutputFormat};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input.html> <output.png|pdf>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} page.html screenshot.png", args[0]);
        eprintln!("  {} page.html document.pdf", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Determine output format from extension
    let format = match Path::new(output_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("pdf") => OutputFormat::Pdf,
        Some("png") => OutputFormat::Png,
        _ => {
            eprintln!("Error: Output file must have .png or .pdf extension");
            std::process::exit(1);
        }
    };

    // Read input HTML
    println!("Reading {}...", input_path);
    let html = fs::read_to_string(input_path)?;

    // Configure rendering
    let config = Config::new()
        .size(800, 1200)
        .scale(2.0)
        .format(format)
        .auto_height(true); // Automatically adjust height to content

    // Render
    println!("Rendering to {:?}...", format);
    let output_bytes = render(&html, config)?;

    // Write output
    fs::write(output_path, &output_bytes)?;
    println!("Saved {} ({} bytes)", output_path, output_bytes.len());

    Ok(())
}
