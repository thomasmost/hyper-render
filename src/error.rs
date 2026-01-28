//! Error types for hyper-render.

use thiserror::Error;

/// Result type alias for hyper-render operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during HTML rendering.
#[derive(Debug, Error)]
pub enum Error {
    /// The requested output format feature is not enabled.
    #[error("output format '{0}' is not enabled; enable the '{0}' feature in Cargo.toml")]
    FormatNotEnabled(&'static str),

    /// Failed to render to PNG format.
    #[error("PNG rendering failed: {0}")]
    PngRender(String),

    /// Failed to render to PDF format.
    #[error("PDF rendering failed: {0}")]
    PdfRender(String),

    /// Failed to encode PNG image.
    #[error("PNG encoding failed: {0}")]
    PngEncode(String),

    /// Failed to create PDF document.
    #[error("PDF creation failed: {0}")]
    PdfCreate(String),

    /// Layout computation failed.
    #[error("layout computation failed: {0}")]
    Layout(String),

    /// Font loading or rendering failed.
    #[error("font error: {0}")]
    Font(String),

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
