//! PDF rendering implementation using Krilla.
//!
//! This module converts the rendered HTML content to a PDF document.
//! It uses the Krilla library which provides a high-level API for PDF generation.
//!
//! Supports:
//! - Background colors on all elements
//! - Text rendering with font embedding
//! - Nested layout positioning

use crate::config::Config;
use crate::error::{Error, Result};

#[cfg(feature = "pdf")]
use blitz_dom::{BaseDocument, Node};
#[cfg(feature = "pdf")]
use blitz_html::HtmlDocument;
#[cfg(feature = "pdf")]
use krilla::color::rgb;
#[cfg(feature = "pdf")]
use krilla::geom::{PathBuilder, Point, Size, Transform};
#[cfg(feature = "pdf")]
use krilla::num::NormalizedF32;
#[cfg(feature = "pdf")]
use krilla::paint::{Fill, FillRule};
#[cfg(feature = "pdf")]
use krilla::page::PageSettings;
#[cfg(feature = "pdf")]
use krilla::surface::Surface;
#[cfg(feature = "pdf")]
use krilla::text::{Font, GlyphId, KrillaGlyph};
#[cfg(feature = "pdf")]
use krilla::Document;
#[cfg(feature = "pdf")]
use parley::PositionedLayoutItem;
#[cfg(feature = "pdf")]
use std::collections::HashMap;

/// Font cache to avoid re-creating fonts for the same font data.
#[cfg(feature = "pdf")]
type FontCache = HashMap<u64, Font>;

/// Render a Blitz document to PDF bytes.
///
/// This function creates a PDF document with the rendered HTML content.
/// Supports:
/// - Page dimensions from config or auto-detected from content
/// - Background colors on all elements
/// - Text rendering with embedded fonts
/// - Nested layout positioning
#[cfg(feature = "pdf")]
pub fn render_to_pdf(document: &HtmlDocument, config: &Config) -> Result<Vec<u8>> {
    let width = config.width as f32;
    let height = if config.auto_height {
        get_content_height(document).unwrap_or(config.height as f32)
    } else {
        config.height as f32
    };

    // Create PDF document
    let mut pdf_doc = Document::new();

    // Create a page with the specified dimensions
    let size = Size::from_wh(width, height)
        .ok_or_else(|| Error::PdfCreate("Invalid page dimensions".to_string()))?;
    let page_settings = PageSettings::new(size);
    let mut page = pdf_doc.start_page_with(page_settings);

    // Get the drawing surface
    let mut surface = page.surface();

    // PDF coordinate system has origin at bottom-left, but we want top-left
    // Apply a transform to flip the Y axis
    let transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, height);
    surface.push_transform(&transform);

    // Draw page background
    let [r, g, b, _a] = config.background;
    draw_rect(&mut surface, 0.0, 0.0, width, height, r, g, b);

    // Font cache to reuse fonts across the document
    let mut font_cache = FontCache::new();

    // Render the document tree (backgrounds and text)
    let doc = document.as_ref();
    let root = doc.root_element();
    render_node(&mut surface, doc, root, 0.0, 0.0, &mut font_cache);

    // Pop the transform
    surface.pop();

    // Finish the surface and page
    surface.finish();
    page.finish();

    // Generate the PDF bytes
    pdf_doc
        .finish()
        .map_err(|e| Error::PdfCreate(format!("{:?}", e)))
}

/// Draw a filled rectangle at the given position with the given color.
#[cfg(feature = "pdf")]
fn draw_rect(surface: &mut Surface, x: f32, y: f32, w: f32, h: f32, r: u8, g: u8, b: u8) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    // Create path for rectangle
    let mut builder = PathBuilder::new();
    builder.move_to(x, y);
    builder.line_to(x + w, y);
    builder.line_to(x + w, y + h);
    builder.line_to(x, y + h);
    builder.close();

    if let Some(path) = builder.finish() {
        // Create fill with color
        let color = rgb::Color::new(r, g, b);
        let fill = Fill {
            paint: color.into(),
            opacity: krilla::num::NormalizedF32::ONE,
            rule: FillRule::NonZero,
        };

        // Set fill and draw
        surface.set_fill(Some(fill));
        surface.draw_path(&path);
    }
}

/// Recursively render a node and its children.
#[cfg(feature = "pdf")]
fn render_node(
    surface: &mut Surface,
    doc: &BaseDocument,
    node: &Node,
    offset_x: f32,
    offset_y: f32,
    font_cache: &mut FontCache,
) {
    // Get layout information
    let layout = &node.final_layout;
    let x = offset_x + layout.location.x;
    let y = offset_y + layout.location.y;
    let width = layout.size.width;
    let height = layout.size.height;

    // Skip nodes with no size
    if width <= 0.0 || height <= 0.0 {
        // Still process children as they might have their own layout
        for child_id in node.children.iter() {
            if let Some(child) = doc.get_node(*child_id) {
                render_node(surface, doc, child, x, y, font_cache);
            }
        }
        return;
    }

    // Check if this node has a background color
    if let Some(style) = node.primary_styles() {
        // Get background color from computed style
        let bg = style.clone_background_color();

        // Extract RGBA color components
        if let Some((r, g, b, a)) = extract_color(&bg) {
            // Only draw if not fully transparent
            if a > 0.0 {
                let r8 = (r * 255.0) as u8;
                let g8 = (g * 255.0) as u8;
                let b8 = (b * 255.0) as u8;
                draw_rect(surface, x, y, width, height, r8, g8, b8);
            }
        }
    }

    // Check for inline text layout data
    if let Some(element_data) = node.element_data() {
        if let Some(text_layout) = &element_data.inline_layout_data {
            render_text(surface, doc, text_layout, x, y, font_cache);
        }
    }

    // Render children
    for child_id in node.children.iter() {
        if let Some(child) = doc.get_node(*child_id) {
            render_node(surface, doc, child, x, y, font_cache);
        }
    }
}

/// Render text from a Parley layout to the PDF surface.
#[cfg(feature = "pdf")]
fn render_text(
    surface: &mut Surface,
    doc: &BaseDocument,
    text_layout: &blitz_dom::node::TextLayout,
    pos_x: f32,
    pos_y: f32,
    font_cache: &mut FontCache,
) {
    use linebender_resource_handle::FontData;

    let text = &text_layout.text;
    let layout = &text_layout.layout;

    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let font_data: FontData = run.font().clone();
                let font_size = run.font_size();
                let style = glyph_run.style();

                // Get or create Krilla font from the Parley font data
                let (raw_data, font_id) = font_data.data.into_raw_parts();
                let krilla_font = font_cache.entry(font_id).or_insert_with(|| {
                    let data: krilla::Data = raw_data.into();
                    Font::new(data, font_data.index).expect("Failed to load font")
                });

                // Get text color from computed styles
                let text_color = doc
                    .get_node(style.brush.id)
                    .and_then(|n| n.primary_styles())
                    .map(|styles| {
                        let inherited = styles.get_inherited_text();
                        // inherited.color is an AbsoluteColor, convert to sRGB
                        let srgb = inherited.color.to_color_space(style::color::ColorSpace::Srgb);
                        (srgb.components.0, srgb.components.1, srgb.components.2, srgb.alpha)
                    })
                    .unwrap_or((0.0, 0.0, 0.0, 1.0));

                // Set fill color for text
                let (r, g, b, _a) = text_color;
                surface.set_fill(Some(Fill {
                    paint: rgb::Color::new(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                    )
                    .into(),
                    opacity: NormalizedF32::ONE,
                    rule: FillRule::NonZero,
                }));

                // Build glyphs for this run using clusters for proper text ranges
                let mut glyphs: Vec<KrillaGlyph> = Vec::new();
                let baseline = glyph_run.baseline();

                for cluster in run.visual_clusters() {
                    if cluster.is_ligature_continuation() {
                        // Ligature continuations have no glyphs of their own
                        if let Some(glyph) = glyphs.last_mut() {
                            glyph.text_range.end = cluster.text_range().end;
                        }
                        continue;
                    }

                    let text_range = cluster.text_range();
                    for glyph in cluster.glyphs() {
                        glyphs.push(KrillaGlyph::new(
                            GlyphId::new(glyph.id as u32),
                            glyph.advance / font_size,
                            glyph.x / font_size,
                            glyph.y / font_size,
                            0.0,
                            text_range.clone(),
                            None,
                        ));
                    }
                }

                if !glyphs.is_empty() {
                    // Position: add node position + glyph run offset
                    let draw_x = pos_x + glyph_run.offset();
                    let draw_y = pos_y + baseline;

                    surface.draw_glyphs(
                        Point::from_xy(draw_x, draw_y),
                        &glyphs,
                        krilla_font.clone(),
                        text,
                        font_size,
                        false, // outlined
                    );
                }
            }
        }
    }
}

/// Extract RGBA color components from a Stylo color value.
#[cfg(feature = "pdf")]
fn extract_color(
    color: &style::values::computed::color::Color,
) -> Option<(f32, f32, f32, f32)> {
    use style::values::generics::color::Color as GenericColor;

    match color {
        GenericColor::Absolute(abs) => {
            // AbsoluteColor has to_color_space method to convert to sRGB
            let srgb = abs.to_color_space(style::color::ColorSpace::Srgb);
            Some((
                srgb.components.0,
                srgb.components.1,
                srgb.components.2,
                srgb.alpha,
            ))
        }
        GenericColor::CurrentColor => {
            // CurrentColor inherits from parent - default to black for now
            Some((0.0, 0.0, 0.0, 1.0))
        }
        _ => None,
    }
}

/// Get the actual content height from the document layout.
#[cfg(feature = "pdf")]
fn get_content_height(document: &HtmlDocument) -> Option<f32> {
    let doc = document.as_ref();
    let root = doc.root_element();
    Some(root.final_layout.size.height)
}

#[cfg(not(feature = "pdf"))]
pub fn render_to_pdf(
    _document: &blitz_html::HtmlDocument,
    _config: &Config,
) -> Result<Vec<u8>> {
    Err(Error::FormatNotEnabled("pdf"))
}
