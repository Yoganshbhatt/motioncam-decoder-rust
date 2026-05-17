use crate::error::{McRawError, Result, read_le_bytes};
use crate::simd_portable::*;
use wide::u16x8;

const ENCODING_BLOCK: usize = 64;
const HEADER_LENGTH: usize = 2;
const METADATA_OFFSET: usize = 16;
const ENCODING_BLOCK_LENGTH: [usize; 17] = [
    0, 8, 16, 24, 32, 40, 48, 64, 64, 80, 80, 128, 128, 128, 128, 128, 128,
];

#[inline(always)]
fn decode1(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x01);
    let p = load_u8_to_u16x8(inp)?;
    store_u16x8(&mut out[0..], p & n)?;
    store_u16x8(&mut out[8..], (p & (n << 1u32)) >> 1u32)?;
    store_u16x8(&mut out[16..], (p & (n << 2u32)) >> 2u32)?;
    store_u16x8(&mut out[24..], (p & (n << 3u32)) >> 3u32)?;
    store_u16x8(&mut out[32..], (p & (n << 4u32)) >> 4u32)?;
    store_u16x8(&mut out[40..], (p & (n << 5u32)) >> 5u32)?;
    store_u16x8(&mut out[48..], (p & (n << 6u32)) >> 6u32)?;
    store_u16x8(&mut out[56..], (p & (n << 7u32)) >> 7u32)?;
    Ok(ENCODING_BLOCK_LENGTH[1])
}

#[inline(always)]
fn decode2_one(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x03);
    let p = load_u8_to_u16x8(inp)?;
    store_u16x8(&mut out[0..], p & n)?;
    store_u16x8(&mut out[8..], (p & (n << 2u32)) >> 2u32)?;
    store_u16x8(&mut out[16..], (p & (n << 4u32)) >> 4u32)?;
    store_u16x8(&mut out[24..], (p & (n << 6u32)) >> 6u32)?;
    Ok(8)
}

#[inline(always)]
fn decode2(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let mut o = decode2_one(&mut out[0..], inp)?;
    o += decode2_one(&mut out[32..], &inp[o..])?;
    Ok(o)
}

#[inline(always)]
fn decode3(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x07);
    let t = u16x8::splat(0x03);
    let r = u16x8::splat(0x01);

    let p0 = load_u8_to_u16x8(inp)?;
    let p1 = load_u8_to_u16x8(&inp[8..])?;
    let p2 = load_u8_to_u16x8(&inp[16..])?;

    let r0 = p0 & n;
    let r1 = (p0 & (n << 3u32)) >> 3u32;
    let _r2 = (p0 & (t << 6u32)) >> 6u32;
    let r3 = p1 & n;
    let r4 = (p1 & (n << 3u32)) >> 3u32;
    let _r5 = (p1 & (t << 6u32)) >> 6u32;
    let r6 = p2 & n;
    let r7 = (p2 & (n << 3u32)) >> 3u32;

    let r2 = _r2 | (((p2 >> 6u32) & r) << 2u32);
    let r5 = _r5 | (((p2 >> 7u32) & r) << 2u32);

    store_u16x8(&mut out[0..], r0)?;
    store_u16x8(&mut out[8..], r1)?;
    store_u16x8(&mut out[16..], r2)?;
    store_u16x8(&mut out[24..], r3)?;
    store_u16x8(&mut out[32..], r4)?;
    store_u16x8(&mut out[40..], r5)?;
    store_u16x8(&mut out[48..], r6)?;
    store_u16x8(&mut out[56..], r7)?;

    Ok(ENCODING_BLOCK_LENGTH[3])
}

#[inline(always)]
fn decode4_one(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x0F);
    let p = load_u8_to_u16x8(inp)?;
    store_u16x8(&mut out[0..], p & n)?;
    store_u16x8(&mut out[8..], (p & (n << 4u32)) >> 4u32)?;
    Ok(8)
}

#[inline(always)]
fn decode4(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let mut o = decode4_one(&mut out[0..], inp)?;
    o += decode4_one(&mut out[16..], &inp[o..])?;
    o += decode4_one(&mut out[32..], &inp[o..])?;
    o += decode4_one(&mut out[48..], &inp[o..])?;
    Ok(o)
}

#[inline(always)]
fn decode5(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x1F);
    let l = u16x8::splat(0x07);
    let u = u16x8::splat(0x03);
    let f = u16x8::splat(0x01);

    let p0 = load_u8_to_u16x8(inp)?;
    let p1 = load_u8_to_u16x8(&inp[8..])?;
    let p2 = load_u8_to_u16x8(&inp[16..])?;
    let p3 = load_u8_to_u16x8(&inp[24..])?;
    let p4 = load_u8_to_u16x8(&inp[32..])?;

    store_u16x8(&mut out[0..], p0 & n)?;
    store_u16x8(&mut out[8..], p1 & n)?;
    store_u16x8(&mut out[16..], p2 & n)?;
    store_u16x8(&mut out[24..], p3 & n)?;
    store_u16x8(&mut out[32..], p4 & n)?;
    store_u16x8(&mut out[40..], (p0 >> 5u32) & l | (((p3 >> 5u32) & u) << 3u32))?;
    store_u16x8(&mut out[48..], (p1 >> 5u32) & l | (((p4 >> 5u32) & u) << 3u32))?;

    let tmp0 = (p2 >> 5u32) & l;
    let tmp1 = tmp0 | ((p3 >> 7u32) & f) << 3u32;
    store_u16x8(&mut out[56..], tmp1 | (((p4 >> 7u32) & f) << 4u32))?;

    Ok(ENCODING_BLOCK_LENGTH[5])
}

#[inline(always)]
fn decode6(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0x3F);
    let l = u16x8::splat(0x03);

    let p0 = load_u8_to_u16x8(inp)?;
    let p1 = load_u8_to_u16x8(&inp[8..])?;
    let p2 = load_u8_to_u16x8(&inp[16..])?;
    let p3 = load_u8_to_u16x8(&inp[24..])?;
    let p4 = load_u8_to_u16x8(&inp[32..])?;
    let p5 = load_u8_to_u16x8(&inp[40..])?;

    store_u16x8(&mut out[0..], p0 & n)?;
    store_u16x8(&mut out[8..], p1 & n)?;
    store_u16x8(&mut out[16..], p2 & n)?;
    store_u16x8(&mut out[24..], p3 & n)?;
    store_u16x8(&mut out[32..], p4 & n)?;
    store_u16x8(&mut out[40..], p5 & n)?;

    store_u16x8(
        &mut out[48..],
        ((p0 >> 6u32) & l) | (((p1 >> 6u32) & l) << 2u32) | (((p2 >> 6u32) & l) << 4u32),
    )?;
    store_u16x8(
        &mut out[56..],
        ((p3 >> 6u32) & l) | (((p4 >> 6u32) & l) << 2u32) | (((p5 >> 6u32) & l) << 4u32),
    )?;

    Ok(ENCODING_BLOCK_LENGTH[6])
}

#[inline(always)]
fn decode8_one(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    store_u16x8(out, load_u8_to_u16x8(inp)?)?;
    Ok(8)
}

#[inline(always)]
fn decode8(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let mut o = 0;
    for i in (0..64).step_by(8) {
        o += decode8_one(&mut out[i..], &inp[o..])?;
    }
    Ok(o)
}

#[inline(always)]
fn decode10(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let n = u16x8::splat(0xFF);
    let l = u16x8::splat(0x03);

    let p0 = load_u8_to_u16x8(inp)?;
    let p1 = load_u8_to_u16x8(&inp[8..])?;
    let p2 = load_u8_to_u16x8(&inp[16..])?;
    let p3 = load_u8_to_u16x8(&inp[24..])?;
    let p4 = load_u8_to_u16x8(&inp[32..])?;
    let p5 = load_u8_to_u16x8(&inp[40..])?;
    let p6 = load_u8_to_u16x8(&inp[48..])?;
    let p7 = load_u8_to_u16x8(&inp[56..])?;
    let p8 = load_u8_to_u16x8(&inp[64..])?;
    let p9 = load_u8_to_u16x8(&inp[72..])?;

    let _r0 = p0 & n;
    let _r1 = p1 & n;
    let _r2 = p2 & n;
    let _r3 = p3 & n;

    store_u16x8(&mut out[0..], _r0 | ((p4 & l) << 8u32))?;
    store_u16x8(&mut out[8..], _r1 | ((p4 & (l << 2u32)) << 6u32))?;
    store_u16x8(&mut out[16..], _r2 | ((p4 & (l << 4u32)) << 4u32))?;
    store_u16x8(&mut out[24..], _r3 | ((p4 & (l << 6u32)) << 2u32))?;

    let _r4 = p5 & n;
    let _r5 = p6 & n;
    let _r6 = p7 & n;
    let _r7 = p8 & n;

    store_u16x8(&mut out[32..], _r4 | ((p9 & l) << 8u32))?;
    store_u16x8(&mut out[40..], _r5 | ((p9 & (l << 2u32)) << 6u32))?;
    store_u16x8(&mut out[48..], _r6 | ((p9 & (l << 4u32)) << 4u32))?;
    store_u16x8(&mut out[56..], _r7 | ((p9 & (l << 6u32)) << 2u32))?;

    Ok(ENCODING_BLOCK_LENGTH[10])
}

#[inline(always)]
fn decode16_one(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let chunk: &[u8; 16] = inp
        .get(..16)
        .ok_or(McRawError::TruncatedData)?
        .try_into()
        .map_err(|_| McRawError::TruncatedData)?;
    let arr = [
        u16::from_le_bytes([chunk[0], chunk[1]]),
        u16::from_le_bytes([chunk[2], chunk[3]]),
        u16::from_le_bytes([chunk[4], chunk[5]]),
        u16::from_le_bytes([chunk[6], chunk[7]]),
        u16::from_le_bytes([chunk[8], chunk[9]]),
        u16::from_le_bytes([chunk[10], chunk[11]]),
        u16::from_le_bytes([chunk[12], chunk[13]]),
        u16::from_le_bytes([chunk[14], chunk[15]]),
    ];
    store_u16x8(out, u16x8::from(arr))?;
    Ok(16)
}

#[inline(always)]
fn decode16(out: &mut [u16], inp: &[u8]) -> Result<usize> {
    let mut o = 0;
    for i in (0..64).step_by(8) {
        o += decode16_one(&mut out[i..], &inp[o..])?;
    }
    Ok(o)
}

#[inline(always)]
fn decode_block(out: &mut [u16], bits: u16, inp: &[u8], offset: usize, len: usize) -> Result<usize> {
    let block_len = ENCODING_BLOCK_LENGTH
        .get(bits as usize)
        .copied()
        .ok_or(McRawError::InvalidCompressionType(bits as i32))?;

    if offset + block_len > len {
        return Ok(len - offset);
    }

    let slice = &inp[offset..];
    match bits {
        0 => {
            out[..ENCODING_BLOCK].fill(0);
            Ok(0)
        }
        1 => decode1(out, slice),
        2 => decode2(out, slice),
        3 => decode3(out, slice),
        4 => decode4(out, slice),
        5 => decode5(out, slice),
        6 => decode6(out, slice),
        7 | 8 => decode8(out, slice),
        9 | 10 => decode10(out, slice),
        _ => decode16(out, slice),
    }
}

fn decode_metadata(inp: &[u8], mut offset: usize, len: usize, out: &mut Vec<u16>) -> Result<usize> {
    if offset + 4 > len {
        return Err(McRawError::TruncatedData);
    }

    let num_blocks = u32::from_le_bytes(read_le_bytes(inp, offset)?) as usize;
    offset += 4;
    out.resize(num_blocks, 0);

    let mut ptr = 0;
    let mut i = 0;
    while i < num_blocks {
        let (bits, reference) = decode_header(&inp[offset..])?;
        offset += HEADER_LENGTH;
        offset += decode_block(&mut out[ptr..], bits as u16, inp, offset, len)?;
        for x in 0..ENCODING_BLOCK.min(num_blocks - i) {
            out[ptr + x] += reference;
        }
        ptr += ENCODING_BLOCK;
        i += ENCODING_BLOCK;
    }
    Ok(offset)
}

/// Decode a raw frame using compression type 7 (current).
///
/// Decodes the compressed `input` into `output` (row-major `u16` Bayer data).
/// Returns the number of `u16` samples written.
pub fn decode(output: &mut [u16], width: usize, _height: usize, input: &[u8]) -> Result<usize> {
    if input.len() < METADATA_OFFSET {
        return Err(McRawError::TruncatedData);
    }

    let enc_w = u32::from_le_bytes(read_le_bytes(input, 0)?) as usize;
    let enc_h = u32::from_le_bytes(read_le_bytes(input, 4)?) as usize;
    let bits_off = u32::from_le_bytes(read_le_bytes(input, 8)?) as usize;
    let refs_off = u32::from_le_bytes(read_le_bytes(input, 12)?) as usize;

    if bits_off > input.len() || refs_off > input.len() || !enc_w.is_multiple_of(ENCODING_BLOCK) || enc_w < width
    {
        return Err(McRawError::DecompressionFailed("invalid raw header".into()));
    }

    let mut bits = Vec::new();
    let mut refs = Vec::new();
    decode_metadata(input, bits_off, input.len(), &mut bits)?;
    decode_metadata(input, refs_off, input.len(), &mut refs)?;

    let mut offset = METADATA_OFFSET;
    let mut meta_idx = 0;
    let mut out_idx = 0;

    let mut row0 = vec![0u16; enc_w];
    let mut row1 = vec![0u16; enc_w];
    let mut row2 = vec![0u16; enc_w];
    let mut row3 = vec![0u16; enc_w];

    for _ in (0..enc_h).step_by(4) {
        for x in (0..enc_w).step_by(ENCODING_BLOCK) {
            let b = [bits[meta_idx], bits[meta_idx + 1], bits[meta_idx + 2], bits[meta_idx + 3]];
            let r = [refs[meta_idx], refs[meta_idx + 1], refs[meta_idx + 2], refs[meta_idx + 3]];

            let mut p0 = [0u16; ENCODING_BLOCK];
            let mut p1 = [0u16; ENCODING_BLOCK];
            let mut p2 = [0u16; ENCODING_BLOCK];
            let mut p3 = [0u16; ENCODING_BLOCK];

            offset += decode_block(&mut p0, b[0], input, offset, input.len())?;
            offset += decode_block(&mut p1, b[1], input, offset, input.len())?;
            offset += decode_block(&mut p2, b[2], input, offset, input.len())?;
            offset += decode_block(&mut p3, b[3], input, offset, input.len())?;

            for i in (0..ENCODING_BLOCK).step_by(2) {
                row0[x + i] = p0[i / 2] + r[0];
                row0[x + i + 1] = p1[i / 2] + r[1];
                row1[x + i] = p2[i / 2] + r[2];
                row1[x + i + 1] = p3[i / 2] + r[3];
                row2[x + i] = p0[ENCODING_BLOCK / 2 + i / 2] + r[0];
                row2[x + i + 1] = p1[ENCODING_BLOCK / 2 + i / 2] + r[1];
                row3[x + i] = p2[ENCODING_BLOCK / 2 + i / 2] + r[2];
                row3[x + i + 1] = p3[ENCODING_BLOCK / 2 + i / 2] + r[3];
            }
            meta_idx += 4;
        }

        let c = width.min(enc_w);
        output[out_idx..out_idx + c].copy_from_slice(&row0[..c]);
        out_idx += width;
        output[out_idx..out_idx + c].copy_from_slice(&row1[..c]);
        out_idx += width;
        output[out_idx..out_idx + c].copy_from_slice(&row2[..c]);
        out_idx += width;
        output[out_idx..out_idx + c].copy_from_slice(&row3[..c]);
        out_idx += width;
    }
    Ok(out_idx)
}
