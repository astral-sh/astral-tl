pub const COMMENT: &[u8; 2] = b"--";
pub const VOID_TAGS: &[&[u8]; 15] = &[
    b"area", b"base", b"br", b"col", b"embed", b"hr", b"img", b"input", b"keygen", b"link",
    b"meta", b"param", b"source", b"track", b"wbr",
];
/// RAWTEXT / RCDATA elements: their content is consumed literally up to the matching close tag,
/// NOT parsed as markup. Without this, inline JS/CSS/JSON containing `<` (e.g. `<script>` bodies on
/// Amazon pages) spawns phantom tags that mis-nest large subtrees. Matched ASCII-case-insensitively.
pub const RAWTEXT_TAGS: &[&[u8]; 4] = &[b"script", b"style", b"textarea", b"title"];
