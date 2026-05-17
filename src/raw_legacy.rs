use crate::error::Result;

const BLOCK_SIZE: usize = 16;
const ENCODING_BLOCK: usize = BLOCK_SIZE * 2;
const HEADER_LENGTH: usize = 2;
const ENCODING_BLOCK_LENGTH: [usize; 17] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 32, 32, 32, 32, 32, 32];

#[inline(always)]
fn decode_header(inp: &[u8]) -> (u8, u16) {
    let b0 = inp[0];
    let b1 = inp[1];
    ((b0 >> 4) & 0x0F, ((b0 & 0x0F) as u16) << 8 | b1 as u16)
}

#[inline(always)]
fn decode_block_legacy(out: &mut [u16], bits: u8, inp: &[u8]) -> usize {
    match bits {
        0 => out[..BLOCK_SIZE].fill(0),
        1 => {
            for i in 0..BLOCK_SIZE / 8 {
                let b = inp[i] as u16;
                out[i * 8] = (b >> 7) & 1;
                out[i * 8 + 1] = (b >> 6) & 1;
                out[i * 8 + 2] = (b >> 5) & 1;
                out[i * 8 + 3] = (b >> 4) & 1;
                out[i * 8 + 4] = (b >> 3) & 1;
                out[i * 8 + 5] = (b >> 2) & 1;
                out[i * 8 + 6] = (b >> 1) & 1;
                out[i * 8 + 7] = b & 1;
            }
        }
        2 => {
            for i in 0..BLOCK_SIZE / 4 {
                let b = inp[i] as u16;
                out[i * 4] = (b >> 6) & 3;
                out[i * 4 + 1] = (b >> 4) & 3;
                out[i * 4 + 2] = (b >> 2) & 3;
                out[i * 4 + 3] = b & 3;
            }
        }
        3 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                out[o] = (b0 >> 5) & 7;
                out[o + 1] = (b0 >> 2) & 7;
                out[o + 2] = ((b0 & 3) << 1) | ((b1 >> 7) & 1);
                out[o + 3] = (b1 >> 4) & 7;
                out[o + 4] = (b1 >> 1) & 7;
                out[o + 5] = ((b1 & 1) << 2) | ((inp[i + 2] as u16 >> 6) & 3);
                out[o + 6] = (inp[i + 2] as u16 >> 3) & 7;
                out[o + 7] = inp[i + 2] as u16 & 7;
                i += 3;
                o += 8;
            }
        }
        4 => {
            for i in 0..BLOCK_SIZE / 2 {
                let b = inp[i] as u16;
                out[i * 2] = (b >> 4) & 0xF;
                out[i * 2 + 1] = b & 0xF;
            }
        }
        5 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                let b2 = inp[i + 2] as u16;
                let b3 = inp[i + 3] as u16;
                let b4 = inp[i + 4] as u16;
                out[o] = (b0 >> 3) & 0x1F;
                out[o + 1] = ((b0 & 7) << 2) | ((b1 >> 6) & 3);
                out[o + 2] = (b1 >> 1) & 0x1F;
                out[o + 3] = ((b1 & 1) << 4) | ((b2 >> 4) & 0xF);
                out[o + 4] = ((b2 & 0xF) << 1) | ((b3 >> 7) & 1);
                out[o + 5] = (b3 >> 2) & 0x1F;
                out[o + 6] = ((b3 & 3) << 3) | ((b4 >> 5) & 7);
                out[o + 7] = b4 & 0x1F;
                i += 5;
                o += 8;
            }
        }
        6 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                let b2 = inp[i + 2] as u16;
                let b3 = inp[i + 3] as u16;
                out[o] = (b0 >> 2) & 0x3F;
                out[o + 1] = ((b0 & 3) << 4) | ((b1 >> 4) & 0xF);
                out[o + 2] = ((b1 & 0xF) << 2) | ((b2 >> 6) & 3);
                out[o + 3] = b2 & 0x3F;
                out[o + 4] = (b3 >> 2) & 0x3F;
                out[o + 5] = ((b3 & 3) << 4) | ((inp[i + 4] as u16 >> 4) & 0xF);
                out[o + 6] = ((inp[i + 4] as u16 & 0xF) << 2) | ((inp[i + 5] as u16 >> 6) & 3);
                out[o + 7] = inp[i + 5] as u16 & 0x3F;
                i += 6;
                o += 8;
            }
        }
        7 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                out[o] = (b0 >> 1) & 0x7F;
                out[o + 1] = ((b0 & 1) << 6) | ((b1 >> 2) & 0x3F);
                out[o + 2] = ((b0 & 3) << 5) | ((b1 >> 3) & 0x1F);
                out[o + 3] = ((b0 & 7) << 4) | ((b1 >> 4) & 0xF);
                out[o + 4] = ((b0 & 0xF) << 3) | ((b1 >> 5) & 7);
                out[o + 5] = ((b0 & 0x1F) << 2) | ((b1 >> 6) & 3);
                out[o + 6] = ((b0 & 0x3F) << 1) | ((b1 >> 7) & 1);
                out[o + 7] = b1 & 0x7F;
                i += 2;
                o += 8;
            }
        }
        8 => {
            for i in 0..BLOCK_SIZE {
                out[i] = inp[i] as u16;
            }
        }
        9 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                out[o] = (b0 << 1) | ((b1 >> 7) & 1);
                out[o + 1] = ((b0 & 0x7F) << 2) | ((inp[i + 2] as u16 >> 6) & 3);
                out[o + 2] = ((inp[i + 2] as u16 & 0x3F) << 3) | ((inp[i + 3] as u16 >> 5) & 7);
                out[o + 3] = ((inp[i + 3] as u16 & 0x1F) << 4) | ((inp[i + 4] as u16 >> 4) & 0xF);
                out[o + 4] = ((inp[i + 4] as u16 & 0xF) << 5) | ((inp[i + 5] as u16 >> 3) & 0x1F);
                out[o + 5] = ((inp[i + 5] as u16 & 7) << 6) | ((inp[i + 6] as u16 >> 2) & 0x3F);
                out[o + 6] = ((inp[i + 6] as u16 & 3) << 7) | ((inp[i + 7] as u16 >> 1) & 0x7F);
                out[o + 7] = ((inp[i + 7] as u16 & 1) << 8) | inp[i + 8] as u16;
                i += 9;
                o += 8;
            }
        }
        10 => {
            let mut o = 0;
            let mut i = 0;
            while o < BLOCK_SIZE {
                let b0 = inp[i] as u16;
                let b1 = inp[i + 1] as u16;
                let b2 = inp[i + 2] as u16;
                let b3 = inp[i + 3] as u16;
                let b4 = inp[i + 4] as u16;
                out[o] = (b0 << 2) | ((b1 >> 6) & 3);
                out[o + 1] = ((b1 & 0x3F) << 4) | ((b2 >> 4) & 0xF);
                out[o + 2] = ((b2 & 0xF) << 6) | ((b3 >> 2) & 0x3F);
                out[o + 3] = ((b3 & 3) << 8) | b4;
                i += 5;
                o += 4;
            }
        }
        _ => {
            for i in 0..BLOCK_SIZE {
                out[i] = ((inp[i * 2] as u16) << 8) | inp[i * 2 + 1] as u16;
            }
        }
    }
    ENCODING_BLOCK_LENGTH[bits as usize]
}

/// Decode a raw frame using compression type 6 (legacy).
///
/// Decodes the compressed `input` into `output` (row-major `u16` Bayer data).
/// Returns the number of `u16` samples written.
pub fn decode_legacy(output: &mut [u16], width: usize, height: usize, input: &[u8]) -> Result<usize> {
    let padded_width = ENCODING_BLOCK * width.div_ceil(ENCODING_BLOCK);
    let mut row = vec![0u16; padded_width];
    let mut offset = 0;
    let mut out_idx = 0;
    let mut p = [0u16; ENCODING_BLOCK];
    for _ in 0..height {
        for x in (0..padded_width).step_by(ENCODING_BLOCK) {
            if offset + HEADER_LENGTH >= input.len() {
                break;
            }
            let (bits0, ref0) = decode_header(&input[offset..]);
            offset += HEADER_LENGTH;
            let (p0, p1) = p.split_at_mut(BLOCK_SIZE);
            offset += decode_block_legacy(p0, bits0.min(16), &input[offset..]);
            if offset + HEADER_LENGTH >= input.len() {
                break;
            }
            let (bits1, ref1) = decode_header(&input[offset..]);
            offset += HEADER_LENGTH;
            offset += decode_block_legacy(p1, bits1.min(16), &input[offset..]);
            for i in (0..ENCODING_BLOCK).step_by(2) {
                row[x + i] = p[i / 2] + ref0;
                row[x + i + 1] = p[BLOCK_SIZE + i / 2] + ref1;
            }
        }
        let c = width.min(padded_width);
        output[out_idx..out_idx + c].copy_from_slice(&row[..c]);
        out_idx += width;
    }
    Ok(out_idx)
}
