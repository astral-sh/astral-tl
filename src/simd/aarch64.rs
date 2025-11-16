use std::arch::aarch64::*;

/// NEON-optimized search for the first non-identifier byte.
///
/// An identifier byte is one of:
/// - '0'-'9' (digits)
/// - 'a'-'z' (lowercase letters)
/// - 'A'-'Z' (uppercase letters)
/// - '-' (hyphen)
/// - '_' (underscore)
/// - '/' (slash)
/// - ':' (colon)
/// - '+' (plus)
#[target_feature(enable = "neon")]
pub unsafe fn search_non_ident_neon(haystack: &[u8]) -> Option<usize> {
    // If the haystack is too small, short-circuit to the fallback implementation.
    let len = haystack.len();
    if len < 16 {
        return super::fallback::search_non_ident(haystack);
    }

    let ptr = haystack.as_ptr();
    let mut offset = 0;

    // Process the input in 16-byte chunks.
    while offset + 16 <= len {
        let chunk = vld1q_u8(ptr.add(offset));

        let is_ident = is_ident_chunk(chunk);
        let min = vminvq_u8(is_ident);

        // If `minv` returns `0xFF`, every byte is an identifier character.
        if min != 0xFF {
            // Find the first `0x00` byte in the `is_ident` mask.
            let mut bytes = [0u8; 16];
            vst1q_u8(bytes.as_mut_ptr(), is_ident);

            for (i, &byte) in bytes.iter().enumerate() {
                if byte == 0 {
                    return Some(offset + i);
                }
            }
        }

        offset += 16;
    }

    // Handle any remaining bytes with the fallback implementation.
    if offset < len {
        super::fallback::search_non_ident(&haystack[offset..]).map(|x| offset + x)
    } else {
        None
    }
}

/// Returns a mask where each byte is `0xFF` if it's an identifier character or `0x00` otherwise.
#[inline(always)]
unsafe fn is_ident_chunk(chunk: uint8x16_t) -> uint8x16_t {
    // C'0'-'9' (0x30-0x39)
    let ge_0 = vcgeq_u8(chunk, vdupq_n_u8(b'0'));
    let le_9 = vcleq_u8(chunk, vdupq_n_u8(b'9'));
    let is_digit = vandq_u8(ge_0, le_9);

    // C'a'-'z' (0x61-0x7A)
    let ge_a_lower = vcgeq_u8(chunk, vdupq_n_u8(b'a'));
    let le_z_lower = vcleq_u8(chunk, vdupq_n_u8(b'z'));
    let is_lowercase = vandq_u8(ge_a_lower, le_z_lower);

    // C'A'-'Z' (0x41-0x5A)
    let ge_a_upper = vcgeq_u8(chunk, vdupq_n_u8(b'A'));
    let le_z_upper = vcleq_u8(chunk, vdupq_n_u8(b'Z'));
    let is_uppercase = vandq_u8(ge_a_upper, le_z_upper);

    // C'-' (0x2D)
    let is_hyphen = vceqq_u8(chunk, vdupq_n_u8(b'-'));

    // C'_' (0x5F)
    let is_underscore = vceqq_u8(chunk, vdupq_n_u8(b'_'));

    // C'/' (0x2F)
    let is_slash = vceqq_u8(chunk, vdupq_n_u8(b'/'));

    // C':' (0x3A)
    let is_colon = vceqq_u8(chunk, vdupq_n_u8(b':'));

    // C'+' (0x2B)
    let is_plus = vceqq_u8(chunk, vdupq_n_u8(b'+'));

    // Combine all identifier character masks.
    let is_alpha = vorrq_u8(is_lowercase, is_uppercase);
    let is_alnum = vorrq_u8(is_digit, is_alpha);
    let is_special1 = vorrq_u8(is_hyphen, is_underscore);
    let is_special2 = vorrq_u8(is_slash, is_colon);
    let is_special3 = vorrq_u8(is_special2, is_plus);
    let is_special = vorrq_u8(is_special1, is_special3);
    vorrq_u8(is_alnum, is_special)
}
