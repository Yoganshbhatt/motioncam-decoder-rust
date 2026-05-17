use crate::error::McRawError;

/// Types of items stored in the MCRAW container.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    BufferIndex = 0,
    BufferIndexData = 1,
    Buffer = 2,
    Metadata = 3,
    AudioIndex = 4,
    AudioData = 5,
    AudioDataMetadata = 6,
}

impl TryFrom<u32> for ItemType {
    type Error = McRawError;
    fn try_from(v: u32) -> std::result::Result<Self, Self::Error> {
        match v {
            0 => Ok(ItemType::BufferIndex),
            1 => Ok(ItemType::BufferIndexData),
            2 => Ok(ItemType::Buffer),
            3 => Ok(ItemType::Metadata),
            4 => Ok(ItemType::AudioIndex),
            5 => Ok(ItemType::AudioData),
            6 => Ok(ItemType::AudioDataMetadata),
            _ => Err(McRawError::InvalidItemType(v)),
        }
    }
}

/// A typed item header in the container stream.
#[derive(Debug, Clone, Copy)]
pub struct Item {
    /// Type of the item.
    pub item_type: ItemType,
    /// Size of the item payload in bytes.
    pub size: u32,
}

/// Offset and timestamp for a buffer (frame or audio) in the file.
#[derive(Debug, Clone, Copy)]
pub struct BufferOffset {
    /// Byte offset in the file.
    pub offset: i64,
    /// Presentation timestamp.
    pub timestamp: i64,
}

/// Index entry describing the frame index section.
#[derive(Debug, Clone, Copy)]
pub struct BufferIndex {
    /// Magic number for validation (`INDEX_MAGIC_NUMBER`).
    pub magic_number: u32,
    /// Number of offset entries.
    pub num_offsets: i32,
    /// File offset where index data begins.
    pub index_data_offset: i64,
}

/// Index entry for audio data.
#[derive(Debug, Clone, Copy)]
pub struct AudioIndex {
    /// Number of audio chunk offsets.
    pub num_offsets: i64,
    /// Start timestamp in milliseconds.
    pub start_timestamp_ms: i64,
}

/// Per-chunk audio metadata.
#[derive(Debug, Clone, Copy)]
pub struct AudioMetadata {
    /// Timestamp of this audio chunk in nanoseconds.
    pub timestamp_ns: i64,
}

/// Magic bytes identifying an MCRAW container (`"MOTION "`).
pub const CONTAINER_ID: [u8; 7] = *b"MOTION ";
/// Current container version supported by this decoder.
pub const CONTAINER_VERSION: u8 = 3;
/// Magic number used to validate frame index data.
pub const INDEX_MAGIC_NUMBER: u32 = 0x8A905612;
