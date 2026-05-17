use thiserror::Error;

/// Errors that can occur during MCRAW decoding.
#[derive(Error, Debug)]
pub enum McRawError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid container header")]
    InvalidHeader,
    #[error("unsupported container version: {0}")]
    UnsupportedVersion(u8),
    #[error("invalid item type: {0}")]
    InvalidItemType(u32),
    #[error("corrupted index: invalid magic number")]
    CorruptedIndex,
    #[error("frame not found (timestamp: {0})")]
    FrameNotFound(i64),
    #[error("invalid compression type: {0}")]
    InvalidCompressionType(i32),
    #[error("decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("metadata parse error: {0}")]
    MetadataParse(#[from] serde_json::Error),
    #[error("unexpected EOF or truncated data")]
    TruncatedData,
}

/// Convenience alias for `Result<T, McRawError>`.
pub type Result<T> = std::result::Result<T, McRawError>;

/// Read `N` bytes from `data` starting at `start` as a little-endian array.
#[inline]
pub fn read_le_bytes<const N: usize>(data: &[u8], start: usize) -> Result<[u8; N]> {
    data.get(start..start + N)
        .ok_or(McRawError::TruncatedData)?
        .try_into()
        .map_err(|_| McRawError::TruncatedData)
}
