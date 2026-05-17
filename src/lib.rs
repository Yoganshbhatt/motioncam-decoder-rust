//! decoder for MotionCam MCRAW files.
//!
//! This crate provides a pure-Rust decoder for the MCRAW container format
//! used by [MotionCam Pro](https://www.motioncamapp.com/). It supports
//! both the current and legacy raw compression formats, audio extraction,
//! and parallel frame decoding.

pub mod container;
pub mod decoder;
pub mod error;
pub mod raw;
pub mod raw_legacy;
mod simd_portable;

pub use container::*;
pub use decoder::{AudioChunk, Decoder};
pub use error::{McRawError, Result};
pub use raw::decode as raw_decode;
pub use raw_legacy::decode_legacy as raw_decode_legacy;
