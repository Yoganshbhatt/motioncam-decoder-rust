# motioncam-decoder

Pure-Rust decoder for [MotionCam](https://www.motioncamapp.com/) MCRAW files.

This is a Rust port of the [motioncam-decoder](https://github.com/mirsadm/motioncam-decoder) C++ library.

## Features

- Decodes MCRAW container format (version 3)
- Supports both compression type 7 (new) and compression type 6 (legacy) raw frames
- Audio extraction
- Parallel frame decoding (optional, enabled by default via `rayon`)
- Memory-mapped file I/O for efficient access

## Usage

```rust
use motioncam_decoder::Decoder;

let mut decoder = Decoder::from_path("video.mcraw")?;

// Read container metadata
let container_meta = decoder.container_metadata();

// Get frame timestamps
let timestamps: Vec<i64> = decoder.frame_timestamps().collect();

// Load a single frame
let (pixels, metadata) = decoder.load_frame(timestamps[0])?;

// Load all frames in parallel
let frames = decoder.load_all_frames_parallel()?;

// Load audio
let audio_chunks = decoder.load_audio()?;
```

## Building

```bash
cargo build --release
```

## Documentation

```bash
cargo doc --no-deps --open
```

## License

Apache 2.0
