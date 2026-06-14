use std::collections::HashMap;
use std::path::Path;
use memmap2::Mmap;
use serde_json::Value;
use rayon::prelude::*;
use crate::container::*;
use crate::error::{McRawError, Result, read_le_bytes};
use crate::raw;
use crate::raw_legacy;

/// A chunk of decoded audio data.
pub struct AudioChunk {
    /// Timestamp of this chunk in nanoseconds, or `-1` if unavailable.
    pub timestamp_ns: i64,
    /// 16-bit signed PCM samples.
    pub samples: Vec<i16>,
}

/// Decoder for MCRAW container files.
///
/// Opens a `.mcraw` file via memory-mapped I/O and provides access to
/// container metadata, raw frame data, and audio chunks.
pub struct Decoder {
    mmap: Mmap,
    metadata: Value,
    frame_offsets: Vec<BufferOffset>,
    frame_map: HashMap<i64, BufferOffset>,
    audio_offsets: Vec<BufferOffset>,
}

impl Decoder {
    /// Open and decode an MCRAW file at the given path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Self::from_mmap(mmap)
    }

    fn from_mmap(mmap: Mmap) -> Result<Self> {
        let data = &mmap[..];
        if data.len() < 8 {
            return Err(McRawError::InvalidHeader);
        }
        if data[0..7] != CONTAINER_ID {
            return Err(McRawError::InvalidHeader);
        }
        if data[7] != CONTAINER_VERSION {
            return Err(McRawError::UnsupportedVersion(data[7]));
        }

        let mut cursor = 8;
        let meta_item = parse_item(&data[cursor..])?;
        cursor += 8;
        if meta_item.item_type != ItemType::Metadata {
            return Err(McRawError::InvalidItemType(meta_item.item_type as u32));
        }
        let meta_json = data
            .get(cursor..cursor + meta_item.size as usize)
            .ok_or(McRawError::TruncatedData)?;
        let metadata: Value = serde_json::from_slice(meta_json)?;

        let idx_size = std::mem::size_of::<BufferIndex>() + std::mem::size_of::<Item>();
        if data.len() < idx_size {
            return Err(McRawError::TruncatedData);
        }
        let end_cursor = data.len() - idx_size;
        let idx_item = parse_item(&data[end_cursor..])?;
        if idx_item.item_type != ItemType::BufferIndex {
            return Err(McRawError::InvalidItemType(idx_item.item_type as u32));
        }
        let buf_idx = parse_buffer_index(&data[end_cursor + 8..])?;
        if buf_idx.magic_number != INDEX_MAGIC_NUMBER {
            return Err(McRawError::CorruptedIndex);
        }

        let mut frame_offsets = vec![BufferOffset { offset: 0, timestamp: 0 }; buf_idx.num_offsets as usize];
        let mut off_cursor = buf_idx.index_data_offset as usize;
        for bo in &mut frame_offsets {
            *bo = parse_buffer_offset(data.get(off_cursor..).ok_or(McRawError::TruncatedData)?)?;
            off_cursor += 16;
        }

        frame_offsets.sort_by_key(|b| b.timestamp);
        let frame_map: HashMap<_, _> = frame_offsets.iter().map(|b| (b.timestamp, *b)).collect();

        let mut audio_offsets = Vec::new();
        if let Some(last) = frame_offsets.last() {
            let mut cur = last.offset as usize;
            while cur + 8 <= data.len() {
                let Ok(item) = parse_item(&data[cur..]) else { break; };
                cur += 8;
                match item.item_type {
                    ItemType::Buffer | ItemType::Metadata | ItemType::AudioData | ItemType::AudioDataMetadata => {
                        cur += item.size as usize;
                    }
                    ItemType::AudioIndex => {
                        let aidx = parse_audio_index(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
                        cur += 16;
                        audio_offsets.reserve(aidx.num_offsets as usize);
                        for _ in 0..aidx.num_offsets {
                            audio_offsets.push(parse_buffer_offset(data.get(cur..).ok_or(McRawError::TruncatedData)?)?);
                            cur += 16;
                        }
                    }
                    _ => break,
                }
            }
        }

        Ok(Self { mmap, metadata, frame_offsets, frame_map, audio_offsets })
    }

    /// Returns a reference to the container-level metadata (JSON).
    pub fn container_metadata(&self) -> &Value {
        &self.metadata
    }

    /// Returns an iterator over all frame timestamps in the container.
    pub fn frame_timestamps(&self) -> impl Iterator<Item = i64> + '_ {
        self.frame_offsets.iter().map(|b| b.timestamp)
    }

    /// Audio sample rate in Hz, read from container metadata.
    pub fn audio_sample_rate_hz(&self) -> i64 {
        self.metadata["extraData"]["audioSampleRate"].as_i64().unwrap_or(0)
    }

    /// Number of audio channels, read from container metadata.
    pub fn num_audio_channels(&self) -> i64 {
        self.metadata["extraData"]["audioChannels"].as_i64().unwrap_or(0)
    }

    /// Load metadata for a specific frame without decoding pixel data.
    pub fn load_frame_metadata(&self, timestamp: i64) -> Result<Value> {
        let offset = self.frame_map.get(&timestamp).ok_or(McRawError::FrameNotFound(timestamp))?.offset as usize;
        let data = &self.mmap[..];
        let mut cur = offset;
        let buf_item = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
        cur += 8 + buf_item.size as usize;
        let meta_item = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
        cur += 8;
        let json_bytes = data.get(cur..cur + meta_item.size as usize).ok_or(McRawError::TruncatedData)?;
        Ok(serde_json::from_slice(json_bytes)?)
    }

    /// Load and decode a single frame by timestamp.
    ///
    /// Returns `(pixels, metadata)` where `pixels` is a `Vec<u16>` in
    /// row-major order (Bayer raw data).
    pub fn load_frame(&self, timestamp: i64) -> Result<(Vec<u16>, Value)> {
        let offset = self.frame_map.get(&timestamp).ok_or(McRawError::FrameNotFound(timestamp))?.offset as usize;
        let data = &self.mmap[..];
        let mut cur = offset;
        let buf_item = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
        cur += 8;
        let compressed = data.get(cur..cur + buf_item.size as usize).ok_or(McRawError::TruncatedData)?;
        cur += buf_item.size as usize;
        let meta_item = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
        cur += 8;
        let json_bytes = data.get(cur..cur + meta_item.size as usize).ok_or(McRawError::TruncatedData)?;
        let metadata: Value = serde_json::from_slice(json_bytes)?;
        let width = metadata["width"].as_u64().ok_or_else(|| McRawError::DecompressionFailed("missing width".into()))? as usize;
        let height = metadata["height"].as_u64().ok_or_else(|| McRawError::DecompressionFailed("missing height".into()))? as usize;
        let comp_type = metadata["compressionType"].as_i64().unwrap_or(7);
        let mut output = vec![0u16; width * height];
        match comp_type {
            7 => { raw::decode(&mut output, width, height, compressed)?; }
            6 => { raw_legacy::decode_legacy(&mut output, width, height, compressed)?; }
            other => return Err(McRawError::InvalidCompressionType(other as i32)),
        }
        Ok((output, metadata))
    }

    /// Decode all frames in parallel using rayon.
    ///
    /// Only available when the `parallel` feature is enabled (default).
    pub fn load_all_frames_parallel(&self) -> Result<Vec<(i64, Vec<u16>, Value)>> {
        self.frame_offsets
            .par_iter()
            .map(|bo| {
                let (pixels, meta) = self.load_frame(bo.timestamp)?;
                Ok((bo.timestamp, pixels, meta))
            })
            .collect()
    }

    /// Load all audio chunks from the container.
    pub fn load_audio(&self) -> Result<Vec<AudioChunk>> {
        let data = &self.mmap[..];
        let mut chunks = Vec::with_capacity(self.audio_offsets.len());
        for off in &self.audio_offsets {
            let mut cur = off.offset as usize;
            if cur + 8 > data.len() {
                continue;
            }
            let item = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?)?;
            cur += 8;
            if item.item_type != ItemType::AudioData {
                continue;
            }
            let samples_bytes = data.get(cur..cur + item.size as usize).ok_or(McRawError::TruncatedData)?;
            cur += item.size as usize;
            let samples: Vec<i16> = samples_bytes
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            let mut timestamp_ns = -1i64;
            if cur + 8 <= data.len() {
                if let Ok(meta_item) = parse_item(data.get(cur..).ok_or(McRawError::TruncatedData)?) {
                    if meta_item.item_type == ItemType::AudioDataMetadata && cur + 16 <= data.len() {
                        timestamp_ns = i64::from_le_bytes(read_le_bytes(data, cur + 8)?);
                    }
                }
            }
            chunks.push(AudioChunk { timestamp_ns, samples });
        }
        Ok(chunks)
    }
}

#[inline]
fn parse_item(data: &[u8]) -> Result<Item> {
    if data.len() < 8 {
        return Err(McRawError::TruncatedData);
    }
    Ok(Item {
        item_type: ItemType::try_from(u32::from_le_bytes(read_le_bytes(data, 0)?))?,
        size: u32::from_le_bytes(read_le_bytes(data, 4)?),
    })
}

#[inline]
fn parse_buffer_offset(data: &[u8]) -> Result<BufferOffset> {
    Ok(BufferOffset {
        offset: i64::from_le_bytes(read_le_bytes(data, 0)?),
        timestamp: i64::from_le_bytes(read_le_bytes(data, 8)?),
    })
}

#[inline]
fn parse_buffer_index(data: &[u8]) -> Result<BufferIndex> {
    Ok(BufferIndex {
        magic_number: u32::from_le_bytes(read_le_bytes(data, 0)?),
        num_offsets: i32::from_le_bytes(read_le_bytes(data, 4)?),
        index_data_offset: i64::from_le_bytes(read_le_bytes(data, 8)?),
    })
}

#[inline]
fn parse_audio_index(data: &[u8]) -> Result<AudioIndex> {
    Ok(AudioIndex {
        num_offsets: i64::from_le_bytes(read_le_bytes(data, 0)?),
        start_timestamp_ms: i64::from_le_bytes(read_le_bytes(data, 8)?),
    })
}
