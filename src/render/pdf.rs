//! PDF rendering implementation using Krilla.
//!
//! This module converts the rendered HTML content to a PDF document.
//! It uses the Krilla library which provides a high-level API for PDF generation.
//!
//! **Note**: PDF rendering is currently in early development. Full HTML-to-PDF
//! conversion with text and complex layouts is a work in progress.

use crate::config::Config;
use crate::error::{Error, Result};

#[cfg(feature = "pdf")]
use blitz_html::HtmlDocument;
#[cfg(feature = "pdf")]
use krilla::geom::Size;
#[cfg(feature = "pdf")]
use krilla::page::PageSettings;
#[cfg(feature = "pdf")]
use krilla::Document;

/// Render a Blitz document to PDF bytes.
///
/// This function creates a PDF document with the rendered HTML content.
///
/// **Note**: This is an early implementation. Full HTML rendering support
/// including text and complex layouts is in development. Currently generates
/// a PDF with the correct page dimensions.
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
    let size = Size::from_wh(width, height).ok_or_else(|| {
        Error::PdfCreate("Invalid page dimensions".to_string())
    })?;
    let page_settings = PageSettings::new(size);
    let mut page = pdf_doc.start_page_with(page_settings);

    // Get the drawing surface
    let surface = page.surface();

    // TODO: Implement full HTML-to-PDF rendering
    // This requires:
    // 1. Walking the layout tree
    // 2. Converting backgrounds to filled paths
    // 3. Embedding fonts and rendering text
    // 4. Handling borders, images, etc.
    //
    // For now, we just create a blank page with correct dimensions.
    // The full implementation will traverse document.as_ref() and
    // render each node's visual representation.

    // Finish the surface and page
    surface.finish();
    page.finish();

    // Generate the PDF bytes
    pdf_doc
        .finish()
        .map_err(|e| Error::PdfCreate(format!("{:?}", e)))
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
