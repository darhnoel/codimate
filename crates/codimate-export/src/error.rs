use std::io;
use std::io::Write;

/// Errors from the export pipeline.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportError {
    Io(io::Error),
    EncoderNotFound,
    EncoderFailed(std::process::ExitStatus),
}

impl From<io::Error> for ExportError {
    fn from(e: io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "I/O error: {}", e),
            ExportError::EncoderNotFound => {
                write!(
                    f,
                    "video encoder (ffmpeg) not found — install it, or point \
                     CODIMATE_FFMPEG at a copy"
                )
            }
            ExportError::EncoderFailed(status) => {
                write!(f, "video encoder failed with exit status: {}", status)
            }
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::Io(e) => Some(e),
            ExportError::EncoderNotFound | ExportError::EncoderFailed(_) => None,
        }
    }
}
