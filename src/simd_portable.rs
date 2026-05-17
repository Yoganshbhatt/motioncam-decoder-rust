use crate::error::{McRawError, Result};
use wide::u16x8;

#[inline(always)]
pub fn load_u8_to_u16x8(src: &[u8]) -> Result<u16x8> {
    let chunk: &[u8; 8] = src
        .get(..8)
        .ok_or(McRawError::TruncatedData)?
        .try_into()
        .map_err(|_| McRawError::TruncatedData)?;

    Ok(u16x8::from([
        chunk[0] as u16,
        chunk[1] as u16,
        chunk[2] as u16,
        chunk[3] as u16,
        chunk[4] as u16,
        chunk[5] as u16,
        chunk[6] as u16,
        chunk[7] as u16,
    ]))
}

#[inline(always)]
pub fn store_u16x8(dst: &mut [u16], val: u16x8) -> Result<()> {
    let arr: [u16; 8] = val.into();
    dst.get_mut(..8)
        .ok_or(McRawError::TruncatedData)?
        .copy_from_slice(&arr);
    Ok(())
}

#[inline(always)]
pub fn decode_header(input: &[u8]) -> Result<(u8, u16)> {
    let chunk: &[u8; 2] = input
        .get(..2)
        .ok_or(McRawError::TruncatedData)?
        .try_into()
        .map_err(|_| McRawError::TruncatedData)?;
    Ok(((chunk[0] >> 4) & 0x0F, ((chunk[0] & 0x0F) as u16) << 8 | chunk[1] as u16))
}
