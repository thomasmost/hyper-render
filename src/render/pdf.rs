//! PDF rendering implementation using Krilla.
//!
//! This module converts the rendered HTML content to a PDF document.
//! It uses the Krilla library which provides a high-level API for PDF generation.
//!
//! Supports:
//! - Background colors on all elements
//! - Border-radius (rounded corners via clip paths)
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
use krilla::geom::{Path, PathBuilder, Point, Size};
#[cfg(feature = "pdf")]
use krilla::num::NormalizedF32;
#[cfg(feature = "pdf")]
use krilla::page::PageSettings;
#[cfg(feature = "pdf")]
use krilla::paint::{Fill, FillRule};
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
#[cfg(feature = "pdf")]
use krilla::geom::Transform;
#[cfg(feature = "pdf")]
use krilla::paint::{LinearGradient, SpreadMethod, Stop};
#[cfg(feature = "pdf")]
use style::color::AbsoluteColor;
#[cfg(feature = "pdf")]
use style::values::computed::{BorderCornerRadius, CSSPixelLength};
#[cfg(feature = "pdf")]
use style::values::generics::image::{GenericGradient, GenericGradientItem, GradientFlags};
#[cfg(feature = "pdf")]
use style::values::specified::position::{HorizontalPositionKeyword, VerticalPositionKeyword};

/// RGB color for PDF rendering.
#[cfg(feature = "pdf")]
#[derive(Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[cfg(feature = "pdf")]
impl Rgb {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Border radii for each corner of a rounded rectangle.
/// Each corner has separate horizontal (x) and vertical (y) radii.
#[cfg(feature = "pdf")]
#[derive(Clone, Copy, Default)]
struct BorderRadii {
    top_left: (f32, f32),
    top_right: (f32, f32),
    bottom_right: (f32, f32),
    bottom_left: (f32, f32),
}

#[cfg(feature = "pdf")]
impl BorderRadii {
    /// Check if any corner has a non-zero radius.
    fn has_any_radius(&self) -> bool {
        self.top_left != (0.0, 0.0)
            || self.top_right != (0.0, 0.0)
            || self.bottom_right != (0.0, 0.0)
            || self.bottom_left != (0.0, 0.0)
    }
}

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

    // Krilla uses a top-left origin coordinate system (like web graphics),
    // so no transform is needed - coordinates map directly.

    // Draw page background
    let [r, g, b, _a] = config.background;
    draw_rect(&mut surface, 0.0, 0.0, width, height, Rgb::new(r, g, b));

    // Font cache to reuse fonts across the document
    let mut font_cache = FontCache::new();

    // Render the document tree (backgrounds and text)
    let doc = document.as_ref();
    let root = doc.root_element();
    render_node(&mut surface, doc, root, 0.0, 0.0, &mut font_cache)?;

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
fn draw_rect(surface: &mut Surface, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
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
        let fill = Fill {
            paint: rgb::Color::new(color.r, color.g, color.b).into(),
            opacity: NormalizedF32::ONE,
            rule: FillRule::NonZero,
        };

        // Set fill and draw
        surface.set_fill(Some(fill));
        surface.draw_path(&path);
    }
}

/// Extract border-radius values from Stylo computed styles.
#[cfg(feature = "pdf")]
fn extract_border_radii(
    style: &style::properties::ComputedValues,
    width: f32,
    height: f32,
) -> BorderRadii {
    let border = style.get_border();

    // Resolution references for percentage-based radii
    let resolve_w = CSSPixelLength::new(width);
    let resolve_h = CSSPixelLength::new(height);

    let resolve = |radius: &BorderCornerRadius| -> (f32, f32) {
        (
            radius.0.width.0.resolve(resolve_w).px(),
            radius.0.height.0.resolve(resolve_h).px(),
        )
    };

    BorderRadii {
        top_left: resolve(&border.border_top_left_radius),
        top_right: resolve(&border.border_top_right_radius),
        bottom_right: resolve(&border.border_bottom_right_radius),
        bottom_left: resolve(&border.border_bottom_left_radius),
    }
}

/// Build a rounded rectangle path using cubic bezier curves at corners.
/// The constant KAPPA (0.5522847498) approximates a quarter circle with a cubic bezier.
#[cfg(feature = "pdf")]
fn build_rounded_rect_path(x: f32, y: f32, w: f32, h: f32, radii: &BorderRadii) -> Option<Path> {
    // Kappa constant for approximating quarter circles with cubic beziers
    const KAPPA: f32 = 0.5522847498;

    let mut builder = PathBuilder::new();

    // Clamp radii to half of dimensions to avoid overlapping
    let clamp_x = |r: f32| r.min(w / 2.0).max(0.0);
    let clamp_y = |r: f32| r.min(h / 2.0).max(0.0);

    let tl = (clamp_x(radii.top_left.0), clamp_y(radii.top_left.1));
    let tr = (clamp_x(radii.top_right.0), clamp_y(radii.top_right.1));
    let br = (clamp_x(radii.bottom_right.0), clamp_y(radii.bottom_right.1));
    let bl = (clamp_x(radii.bottom_left.0), clamp_y(radii.bottom_left.1));

    // Start at top-left, after the corner arc
    builder.move_to(x + tl.0, y);

    // Top edge
    builder.line_to(x + w - tr.0, y);

    // Top-right corner (cubic bezier)
    if tr.0 > 0.0 && tr.1 > 0.0 {
        builder.cubic_to(
            x + w - tr.0 * (1.0 - KAPPA),
            y,
            x + w,
            y + tr.1 * (1.0 - KAPPA),
            x + w,
            y + tr.1,
        );
    }

    // Right edge
    builder.line_to(x + w, y + h - br.1);

    // Bottom-right corner
    if br.0 > 0.0 && br.1 > 0.0 {
        builder.cubic_to(
            x + w,
            y + h - br.1 * (1.0 - KAPPA),
            x + w - br.0 * (1.0 - KAPPA),
            y + h,
            x + w - br.0,
            y + h,
        );
    }

    // Bottom edge
    builder.line_to(x + bl.0, y + h);

    // Bottom-left corner
    if bl.0 > 0.0 && bl.1 > 0.0 {
        builder.cubic_to(
            x + bl.0 * (1.0 - KAPPA),
            y + h,
            x,
            y + h - bl.1 * (1.0 - KAPPA),
            x,
            y + h - bl.1,
        );
    }

    // Left edge
    builder.line_to(x, y + tl.1);

    // Top-left corner
    if tl.0 > 0.0 && tl.1 > 0.0 {
        builder.cubic_to(
            x,
            y + tl.1 * (1.0 - KAPPA),
            x + tl.0 * (1.0 - KAPPA),
            y,
            x + tl.0,
            y,
        );
    }

    builder.close();
    builder.finish()
}

/// Convert a Stylo linear gradient to a Krilla LinearGradient.
#[cfg(feature = "pdf")]
fn convert_linear_gradient(
    direction: &style::values::computed::LineDirection,
    items: &[GenericGradientItem<
        style::values::generics::color::GenericColor<style::values::computed::Percentage>,
        style::values::computed::LengthPercentage,
    >],
    flags: GradientFlags,
    rect_width: f32,
    rect_height: f32,
    current_color: &AbsoluteColor,
) -> Option<LinearGradient> {
    use style::values::computed::LineDirection;

    // Calculate start and end points based on direction
    // CSS gradients go from start to end in the direction specified
    let (x1, y1, x2, y2) = match direction {
        LineDirection::Angle(angle) => {
            // CSS angle: 0deg = to top, 90deg = to right, etc.
            // We need to convert to standard math angle (counter-clockwise from right)
            let radians = -angle.radians() + std::f32::consts::PI;
            let center_x = rect_width / 2.0;
            let center_y = rect_height / 2.0;
            // Calculate offset to reach corners
            let offset_len = rect_width / 2.0 * radians.sin().abs()
                + rect_height / 2.0 * radians.cos().abs();
            (
                center_x - offset_len * radians.sin(),
                center_y - offset_len * radians.cos(),
                center_x + offset_len * radians.sin(),
                center_y + offset_len * radians.cos(),
            )
        }
        LineDirection::Horizontal(horizontal) => {
            let mid_y = rect_height / 2.0;
            match horizontal {
                HorizontalPositionKeyword::Right => (0.0, mid_y, rect_width, mid_y),
                HorizontalPositionKeyword::Left => (rect_width, mid_y, 0.0, mid_y),
            }
        }
        LineDirection::Vertical(vertical) => {
            let mid_x = rect_width / 2.0;
            match vertical {
                VerticalPositionKeyword::Top => (mid_x, rect_height, mid_x, 0.0),
                VerticalPositionKeyword::Bottom => (mid_x, 0.0, mid_x, rect_height),
            }
        }
        LineDirection::Corner(horizontal, vertical) => {
            let (start_x, end_x) = match horizontal {
                HorizontalPositionKeyword::Right => (0.0, rect_width),
                HorizontalPositionKeyword::Left => (rect_width, 0.0),
            };
            let (start_y, end_y) = match vertical {
                VerticalPositionKeyword::Top => (rect_height, 0.0),
                VerticalPositionKeyword::Bottom => (0.0, rect_height),
            };
            (start_x, start_y, end_x, end_y)
        }
    };

    // Calculate gradient length for position resolution
    let gradient_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let gradient_length_css = CSSPixelLength::new(gradient_length);

    // Convert color stops
    let stops = convert_gradient_stops(items, gradient_length_css, current_color);
    if stops.is_empty() {
        return None;
    }

    let repeating = flags.contains(GradientFlags::REPEATING);

    Some(LinearGradient {
        x1,
        y1,
        x2,
        y2,
        transform: Transform::identity(),
        spread_method: if repeating {
            SpreadMethod::Repeat
        } else {
            SpreadMethod::Pad
        },
        stops,
        anti_alias: true,
    })
}

/// Convert Stylo gradient color stops to Krilla stops.
#[cfg(feature = "pdf")]
fn convert_gradient_stops(
    items: &[GenericGradientItem<
        style::values::generics::color::GenericColor<style::values::computed::Percentage>,
        style::values::computed::LengthPercentage,
    >],
    gradient_length: CSSPixelLength,
    current_color: &AbsoluteColor,
) -> Vec<Stop> {
    use style::values::specified::percentage::ToPercentage;

    let mut stops = Vec::new();
    let num_items = items
        .iter()
        .filter(|item| !matches!(item, GenericGradientItem::InterpolationHint(_)))
        .count();

    let mut color_stop_idx = 0;
    for item in items.iter() {
        match item {
            GenericGradientItem::SimpleColorStop(color) => {
                // Simple stop: evenly distributed
                let offset = if num_items > 1 {
                    color_stop_idx as f32 / (num_items - 1) as f32
                } else {
                    0.0
                };
                color_stop_idx += 1;

                if let Some(stop) = color_to_krilla_stop(color, offset, current_color) {
                    stops.push(stop);
                }
            }
            GenericGradientItem::ComplexColorStop { color, position } => {
                // Complex stop: has explicit position
                if let Some(percentage) = position.to_percentage_of(gradient_length) {
                    let offset = percentage.to_percentage();
                    color_stop_idx += 1;

                    if let Some(stop) = color_to_krilla_stop(color, offset, current_color) {
                        stops.push(stop);
                    }
                }
            }
            GenericGradientItem::InterpolationHint(_) => {
                // Interpolation hints are not directly supported; skip for now
            }
        }
    }

    stops
}

/// Convert a Stylo color to a Krilla gradient stop.
#[cfg(feature = "pdf")]
fn color_to_krilla_stop(
    color: &style::values::generics::color::GenericColor<style::values::computed::Percentage>,
    offset: f32,
    current_color: &AbsoluteColor,
) -> Option<Stop> {
    let abs_color = color.resolve_to_absolute(current_color);
    let srgb = abs_color.to_color_space(style::color::ColorSpace::Srgb);

    let r = (srgb.components.0.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (srgb.components.1.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (srgb.components.2.clamp(0.0, 1.0) * 255.0) as u8;
    let alpha = srgb.alpha.clamp(0.0, 1.0);

    Some(Stop {
        offset: NormalizedF32::new(offset.clamp(0.0, 1.0))?,
        color: rgb::Color::new(r, g, b).into(),
        opacity: NormalizedF32::new(alpha).unwrap_or(NormalizedF32::ONE),
    })
}

/// Draw a gradient-filled rectangle.
#[cfg(feature = "pdf")]
fn draw_gradient_rect(
    surface: &mut Surface,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    gradient: LinearGradient,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    // Translate gradient coordinates to absolute position
    let translated_gradient = LinearGradient {
        x1: x + gradient.x1,
        y1: y + gradient.y1,
        x2: x + gradient.x2,
        y2: y + gradient.y2,
        ..gradient
    };

    // Create path for rectangle
    let mut builder = PathBuilder::new();
    builder.move_to(x, y);
    builder.line_to(x + w, y);
    builder.line_to(x + w, y + h);
    builder.line_to(x, y + h);
    builder.close();

    if let Some(path) = builder.finish() {
        let fill = Fill {
            paint: translated_gradient.into(),
            opacity: NormalizedF32::ONE,
            rule: FillRule::NonZero,
        };

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
) -> Result<()> {
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
                render_node(surface, doc, child, x, y, font_cache)?;
            }
        }
        return Ok(());
    }

    // Extract border radii and check if we need clipping
    let radii = node
        .primary_styles()
        .map(|style| extract_border_radii(&*style, width, height))
        .unwrap_or_default();
    let has_radius = radii.has_any_radius();

    // Apply clip path for rounded corners
    if has_radius {
        if let Some(clip_path) = build_rounded_rect_path(x, y, width, height, &radii) {
            surface.push_clip_path(&clip_path, &FillRule::NonZero);
        }
    }

    // Draw backgrounds (color first, then gradients on top)
    if let Some(style) = node.primary_styles() {
        // Get current color for resolving `currentColor` in gradients
        let current_color = style
            .get_inherited_text()
            .color
            .to_color_space(style::color::ColorSpace::Srgb);

        // 1. Draw background color
        let bg_color = style.clone_background_color();
        if let Some((r, g, b, a)) = extract_color(&bg_color) {
            if a > 0.0 {
                let color = Rgb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
                draw_rect(surface, x, y, width, height, color);
            }
        }

        // 2. Draw background gradients (on top of color)
        let bg = style.get_background();
        for bg_image in bg.background_image.0.iter() {
            if let style::values::generics::image::GenericImage::Gradient(gradient) = bg_image {
                match gradient.as_ref() {
                    GenericGradient::Linear {
                        direction,
                        items,
                        flags,
                        ..
                    } => {
                        if let Some(linear_grad) = convert_linear_gradient(
                            direction,
                            items,
                            *flags,
                            width,
                            height,
                            &current_color,
                        ) {
                            draw_gradient_rect(surface, x, y, width, height, linear_grad);
                        }
                    }
                    // TODO: Support radial and conic gradients
                    _ => {}
                }
            }
        }
    }

    // Check for inline text layout data
    if let Some(element_data) = node.element_data() {
        if let Some(text_layout) = &element_data.inline_layout_data {
            render_text(surface, doc, text_layout, x, y, font_cache)?;
        }
    }

    // Render children
    for child_id in node.children.iter() {
        if let Some(child) = doc.get_node(*child_id) {
            render_node(surface, doc, child, x, y, font_cache)?;
        }
    }

    // Pop clip path if we applied one
    if has_radius {
        surface.pop();
    }

    Ok(())
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
) -> Result<()> {
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
                let krilla_font = if let Some(font) = font_cache.get(&font_id) {
                    font.clone()
                } else {
                    let data: krilla::Data = raw_data.into();
                    let font = Font::new(data, font_data.index)
                        .ok_or_else(|| Error::Font("failed to load font from data".to_string()))?;
                    font_cache.insert(font_id, font.clone());
                    font
                };

                // Get text color from computed styles
                // Note: Alpha is extracted but not used - PDF text opacity would require
                // additional graphics state handling which is not yet implemented.
                let text_color = doc
                    .get_node(style.brush.id)
                    .and_then(|n| n.primary_styles())
                    .map(|styles| {
                        let inherited = styles.get_inherited_text();
                        // inherited.color is an AbsoluteColor, convert to sRGB
                        let srgb = inherited
                            .color
                            .to_color_space(style::color::ColorSpace::Srgb);
                        (
                            srgb.components.0,
                            srgb.components.1,
                            srgb.components.2,
                            srgb.alpha,
                        )
                    })
                    .unwrap_or((0.0, 0.0, 0.0, 1.0)); // Default to opaque black

                // Set fill color for text
                let (r, g, b, _a) = text_color;
                surface.set_fill(Some(Fill {
                    paint: rgb::Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
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
                            GlyphId::new(glyph.id),
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
                        krilla_font,
                        text,
                        font_size,
                        false, // outlined
                    );
                }
            }
        }
    }

    Ok(())
}

/// Extract RGBA color components from a Stylo color value.
#[cfg(feature = "pdf")]
fn extract_color(color: &style::values::computed::color::Color) -> Option<(f32, f32, f32, f32)> {
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
pub fn render_to_pdf(_document: &blitz_html::HtmlDocument, _config: &Config) -> Result<Vec<u8>> {
    Err(Error::FormatNotEnabled("pdf"))
}
