use crate::util;

/// Fallback functions, used for search_non_ident on unsupported architectures
pub mod fallback;

// Architecture-specific SIMD implementations
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

/// Checks if the given byte is a "closing" byte (/ or >)
#[inline]
pub fn is_closing(needle: u8) -> bool {
    (needle == b'/') | (needle == b'>')
}

/// Searches for the first non-identifier in `haystack`
#[inline]
pub fn search_non_ident(haystack: &[u8]) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: We check for SSE2 availability at runtime on x86_64
        // SSE2 is available on all x86_64 CPUs by definition
        unsafe { x86_64::search_non_ident_sse2(haystack) }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is available on all aarch64 CPUs by definition
        unsafe { aarch64::search_non_ident_neon(haystack) }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        fallback::search_non_ident(haystack)
    }
}

/// Searches for the first occurrence of any of 3 bytes in `haystack`
#[inline]
pub fn find3(haystack: &[u8], needle: [u8; 3]) -> Option<usize> {
    memchr::memchr3(needle[0], needle[1], needle[2], haystack)
}

/// Searches for the first occurence of `needle` in `haystack`
#[inline]
pub fn find(haystack: &[u8], needle: u8) -> Option<usize> {
    memchr::memchr(needle, haystack)
}

/// Checks if the ASCII characters in `haystack` match `needle` (case insensitive)
pub fn matches_case_insensitive<const N: usize>(haystack: &[u8], needle: [u8; N]) -> bool {
    if haystack.len() != N {
        return false;
    }

    // LLVM seems to already generate pretty good SIMD even without explicit use

    let mut mask = true;
    for i in 0..N {
        mask &= util::to_lower(haystack[i]) == needle[i];
    }
    mask
}
