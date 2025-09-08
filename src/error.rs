/// Error type for tokau operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokauError {
    /// The token ID is out of the valid range for this token space
    OutOfRange {
        /// The value that was out of range
        value: u32,
        /// The maximum valid value (exclusive)
        max: u32,
    },
}

impl std::fmt::Display for TokauError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokauError::OutOfRange { value, max } => {
                write!(f, "Token ID {} is out of valid range [0, {})", value, max)
            }
        }
    }
}

impl std::error::Error for TokauError {}
