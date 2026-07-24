static IDENT_CHARS: [bool; 256] = {
    let mut chars = [false; 256];
    let mut i = 0;
    while i < 256 {
        let c = i as u8;
        // NOTE: '/' is intentionally NOT an identifier byte. Per the HTML tokenizer, '/' is never
        // part of a tag/attribute name — it triggers the self-closing-start-tag state. Treating it as
        // an ident byte made `read_ident` fold the solidus into the name for no-space void tags
        // (`<br/>` -> name "br/"), which is not in VOID_TAGS and so was pushed as a phantom element
        // that never closes, mis-nesting all following siblings. (Attribute VALUES like href="/x" are
        // read via read_to, not read_ident, so they are unaffected.)
        if c.is_ascii_digit()
            || c.is_ascii_uppercase()
            || c.is_ascii_lowercase()
            || c == b'-'
            || c == b'_'
            || c == b':'
            || c == b'+'
        {
            chars[i] = true;
        }
        i += 1;
    }
    chars
};

pub fn is_ident(c: u8) -> bool {
    IDENT_CHARS[c as usize]
}

#[inline(always)]
pub fn to_lower(byte: u8) -> u8 {
    let is_upper = byte.is_ascii_uppercase() as u8;
    let lower = is_upper * 0x20;
    byte + lower
}
