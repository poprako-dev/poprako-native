use std::io::{Read, Seek, SeekFrom};

use crate::complex::archive::json::METADATA_LIMIT;
use crate::result::{AppError, AppResult};

fn number<const N: usize>(bytes: &[u8], offset: usize) -> AppResult<[u8; N]> {
    bytes
        .get(offset..offset + N)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(AppError::InvalidInput)
}

/// # Errors
/// Bounds central directory allocation before the ZIP library parses it.
pub fn preflight<R: Read + Seek>(reader: &mut R) -> AppResult<()> {
    let length = reader
        .seek(SeekFrom::End(0))
        .map_err(|_| AppError::Storage)?;

    let tail_length = usize::try_from(length.min(65_557)).map_err(|_| AppError::InvalidInput)?;

    let tail_offset = length - tail_length as u64;

    reader
        .seek(SeekFrom::Start(tail_offset))
        .map_err(|_| AppError::Storage)?;

    let mut tail = vec![0_u8; tail_length];

    reader
        .read_exact(&mut tail)
        .map_err(|_| AppError::InvalidInput)?;

    let offset = tail
        .windows(4)
        .rposition(|bytes| bytes == b"PK\x05\x06")
        .ok_or(AppError::InvalidInput)?;

    let end = &tail[offset..];

    if end.len() < 22
        || u16::from_le_bytes(number(end, 4)?) != 0
        || u16::from_le_bytes(number(end, 6)?) != 0
        || end.len() != 22 + usize::from(u16::from_le_bytes(number(end, 20)?))
    {
        return Err(AppError::InvalidInput);
    }

    let mut count = u64::from(u16::from_le_bytes(number(end, 10)?));

    let mut size = u64::from(u32::from_le_bytes(number(end, 12)?));

    let mut start = u64::from(u32::from_le_bytes(number(end, 16)?));

    if count == u64::from(u16::MAX) || size == u64::from(u32::MAX) || start == u64::from(u32::MAX) {
        let locator_offset = (tail_offset + offset as u64)
            .checked_sub(20)
            .ok_or(AppError::InvalidInput)?;

        let mut locator = [0_u8; 20];

        reader
            .seek(SeekFrom::Start(locator_offset))
            .map_err(|_| AppError::Storage)?;

        reader
            .read_exact(&mut locator)
            .map_err(|_| AppError::InvalidInput)?;

        if &locator[..4] != b"PK\x06\x07"
            || u32::from_le_bytes(number(&locator, 4)?) != 0
            || u32::from_le_bytes(number(&locator, 16)?) != 1
        {
            return Err(AppError::InvalidInput);
        }

        let mut record = [0_u8; 56];

        reader
            .seek(SeekFrom::Start(u64::from_le_bytes(number(&locator, 8)?)))
            .map_err(|_| AppError::Storage)?;

        reader
            .read_exact(&mut record)
            .map_err(|_| AppError::InvalidInput)?;

        if &record[..4] != b"PK\x06\x06"
            || u32::from_le_bytes(number(&record, 16)?) != 0
            || u32::from_le_bytes(number(&record, 20)?) != 0
        {
            return Err(AppError::InvalidInput);
        }

        count = u64::from_le_bytes(number(&record, 32)?);

        size = u64::from_le_bytes(number(&record, 40)?);

        start = u64::from_le_bytes(number(&record, 48)?);
    }

    let allocation = count
        .checked_mul(512)
        .and_then(|count| count.checked_add(size))
        .ok_or(AppError::InvalidInput)?;

    if allocation > METADATA_LIMIT as u64 || start.checked_add(size).is_none_or(|end| end > length)
    {
        return Err(AppError::InvalidInput);
    }

    reader
        .seek(SeekFrom::Start(0))
        .map_err(|_| AppError::Storage)?;

    Ok(())
}
