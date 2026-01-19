use std::fmt;

/// Errors that can occur during framebuffer operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FramebufferError {
    /// Region dimensions are invalid (zero width/height)
    InvalidRegionDimensions { width: u32, height: u32 },

    /// Region extends beyond screen boundaries
    RegionOutOfBounds {
        region: String,
        screen_width: u32,
        screen_height: u32,
    },

    /// I/O operation failed
    IoError(String),

    /// Refresh operation failed
    RefreshFailed { marker: u32, reason: String },

    /// Update wait timed out
    WaitTimeout { marker: u32 },

    /// Generic error with message
    Other(String),
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FramebufferError::InvalidRegionDimensions { width, height } => {
                write!(
                    f,
                    "Invalid region dimensions: {}x{} (must be non-zero)",
                    width, height
                )
            }
            FramebufferError::RegionOutOfBounds {
                region,
                screen_width,
                screen_height,
            } => {
                write!(
                    f,
                    "Region {} extends beyond screen boundaries ({}x{})",
                    region, screen_width, screen_height
                )
            }
            FramebufferError::IoError(msg) => write!(f, "I/O error: {}", msg),
            FramebufferError::RefreshFailed { marker, reason } => {
                write!(f, "Refresh failed for marker {}: {}", marker, reason)
            }
            FramebufferError::WaitTimeout { marker } => {
                write!(f, "Wait timeout for update marker {}", marker)
            }
            FramebufferError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for FramebufferError {}

/// Convenience type alias for Results with FramebufferError
pub type Result<T> = std::result::Result<T, FramebufferError>;

// Conversion from &'static str for backwards compatibility
impl From<&'static str> for FramebufferError {
    fn from(s: &'static str) -> Self {
        FramebufferError::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formatting() {
        let err = FramebufferError::InvalidRegionDimensions {
            width: 0,
            height: 10,
        };
        assert_eq!(
            err.to_string(),
            "Invalid region dimensions: 0x10 (must be non-zero)"
        );
    }

    #[test]
    fn error_from_static_str() {
        let err: FramebufferError = "test error".into();
        assert!(matches!(err, FramebufferError::Other(_)));
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FramebufferError>();
    }
}
