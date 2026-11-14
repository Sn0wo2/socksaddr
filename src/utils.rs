use crate::ParseError;

pub(super) fn take<const N: usize>(buf: &mut &[u8]) -> Result<[u8; N], ParseError> {
    let (bytes, remaining) = buf.split_first_chunk().ok_or(ParseError::Truncated)?;
    *buf = remaining;
    Ok(*bytes)
}
