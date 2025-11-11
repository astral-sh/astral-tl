#[inline(never)]
pub fn is_ident(c: u8) -> bool {
    c.is_ascii_digit()
        || c.is_ascii_uppercase()
        || c.is_ascii_lowercase()
        || c == b'-'
        || c == b'_'
        || c == b':'
        || c == b'+'
        || c == b'/'
}

#[inline(always)]
pub fn to_lower(byte: u8) -> u8 {
    let is_upper = byte.is_ascii_uppercase() as u8;
    let lower = is_upper * 0x20;
    byte + lower
}
