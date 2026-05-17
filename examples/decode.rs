use std::env;
use std::fs;
use std::io::Write;

use motioncam_decoder::Decoder;

fn write_audio(audio_chunks: &[motioncam_decoder::AudioChunk], output_path: &str) {
    let mut file = fs::File::create(output_path).expect("failed to create audio file");
    for chunk in audio_chunks {
        let bytes: Vec<u8> = chunk
            .samples
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();
        file.write_all(&bytes).expect("failed to write audio");
    }
    println!("  wrote {}", output_path);
}

fn write_raw_frame(data: &[u16], width: usize, height: usize, output_path: &str) {
    let bytes: Vec<u8> = data.iter().flat_map(|s| s.to_le_bytes()).collect();
    fs::write(output_path, &bytes).expect("failed to write frame");
    println!("  wrote {} ({}x{})", output_path, width, height);
}

fn fmt_metadata(meta: &serde_json::Value) -> String {
    let w = meta["width"].as_u64().unwrap_or(0);
    let h = meta["height"].as_u64().unwrap_or(0);
    let comp = meta["compressionType"].as_i64().unwrap_or(-1);
    format!("{}x{} comp={}", w, h, comp)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: decode <input.mcraw> [-n <num_frames>]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let mut end_frame: Option<usize> = None;

    if args.len() > 3 && args[2] == "-n" {
        end_frame = Some(args[3].parse().expect("invalid frame count"));
    }

    let decoder = Decoder::from_path(input_path).expect("failed to open decoder");

    let timestamps: Vec<i64> = decoder.frame_timestamps().collect();

    println!("Found {} frames", timestamps.len());

    let end = end_frame.unwrap_or(timestamps.len()).min(timestamps.len());

    //
    // Audio
    //
    {
        let audio_chunks = decoder.load_audio().expect("failed to load audio");
        if !audio_chunks.is_empty() {
            println!(
                "Audio: {} chunks, {} Hz, {} channels",
                audio_chunks.len(),
                decoder.audio_sample_rate_hz(),
                decoder.num_audio_channels()
            );
            write_audio(&audio_chunks, "audio.raw");
        } else {
            println!("Audio: none");
        }
    }

    //
    // Frames
    //
    for i in 0..end {
        let (pixels, meta) = decoder
            .load_frame(timestamps[i])
            .unwrap_or_else(|e| panic!("failed to decode frame {}: {}", i, e));

        let width = meta["width"].as_u64().unwrap_or(0) as usize;
        let height = meta["height"].as_u64().unwrap_or(0) as usize;

        let filename = format!("frame_{:06}.raw", i);
        println!("Frame {}: {} (ts={})", i, fmt_metadata(&meta), timestamps[i]);
        write_raw_frame(&pixels, width, height, &filename);
    }
}
