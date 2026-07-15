# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`astral-tl` (crate name: `tl`) is a zero-copy, pure-Rust HTML parser library. It exposes a DOM-like API backed by an arena of `Node` objects indexed by `NodeHandle`. The library crate name is `tl` (set in `Cargo.toml` `[lib] name = "tl"`), so all external usage is `use tl::...`.

## Commands

```bash
# Build
cargo build

# Run tests (all unit tests live in src/tests.rs)
cargo test --lib

# Run a single test by name
cargo test --lib <test_name>

# Run benchmarks (criterion, HTML reports in target/criterion/)
cargo bench

# Run with SIMD internals exposed (needed by fuzz/bench crates)
cargo build --features __INTERNALS_DO_NOT_USE

# Fuzzing (requires nightly + cargo-fuzz)
cargo +nightly fuzz run parse
cargo +nightly fuzz run queryselector
cargo +nightly fuzz run find
cargo +nightly fuzz run parse_mut
```

## Architecture

### Data flow

```
tl::parse(input: &str, options) → VDom<'a>
         │
         └─ Parser::new(input, options)
              └─ Parser::parse()          # drives the stream
                   ├─ parse_single()     # one tag or raw text per call
                   ├─ parse_tag()        # reads name, attributes, pushes to stack
                   ├─ parse_attributes() # builds Attributes struct
                   └─ read_end()         # pops stack, updates raw span, fills id/class maps
```

`VDom` is a thin wrapper around `Parser`; all nodes live in `Parser::tags` (a `Vec<Node>`). `NodeHandle` is just a `u32` index into that vec. Callers must always pass the originating `Parser`/`VDom` to resolve a handle — handles are not self-contained.

### Key types

| Type | File | Role |
|---|---|---|
| `VDom<'a>` | `src/vdom.rs` | Public entry point; wraps `Parser`, exposes query API |
| `VDomGuard` | `src/vdom.rs` | Owned version — leaks input string, frees it on drop |
| `Parser<'a>` | `src/parser/base.rs` | Arena + stream + stack for recursive tag parsing |
| `Node<'a>` | `src/parser/tag.rs` | Enum: `Tag(HTMLTag)`, `Raw(Bytes)`, `Comment(Bytes)` |
| `HTMLTag<'a>` | `src/parser/tag.rs` | Single element: name, attributes, children handles, raw span |
| `Attributes<'a>` | `src/parser/tag.rs` | `id` and `class` stored directly; others in `InlineHashMap` |
| `NodeHandle` | `src/parser/handle.rs` | `u32` index; resolved via `handle.get(parser)` |
| `Bytes<'a>` | `src/bytes.rs` | Borrowed-or-owned byte slice kept at 16 bytes on 64-bit |
| `ParserOptions` | `src/parser/options.rs` | Bitflags: `track_ids()`, `track_classes()` |
| `Selector<'a>` | `src/queryselector/selector.rs` | Recursive enum for CSS selectors |

### Inline data structures (`src/inline/`)

`InlineVec<T, N>` and `InlineHashMap<K, V, N>` are stack-allocated up to `N` elements; they spill to heap beyond that. They exist specifically to avoid heap allocations for the typical case of few children/attributes.

- `HTMLTag` children: `InlineVec<NodeHandle, INLINED_SUBNODES>` (small child lists stay on stack)
- `Attributes` raw map: `InlineHashMap` (most tags have only a handful of attributes)

### SIMD (`src/simd/`)

Byte search functions dispatch at compile time: `x86_64` (SSE2), `aarch64` (NEON), or `fallback`. The `simd` module is private by default; exposed only with `--features __INTERNALS_DO_NOT_USE` (used by fuzz and bench crates). Do not use this feature in production code.

### `ParserOptions` tracking flags

`get_element_by_id()` is O(n) by default; calling `.track_ids()` on `ParserOptions` makes it O(1) by building a `HashMap` during parsing. Same for `.track_classes()`. Both options are opt-in to avoid the overhead for callers that don't need them.

### Query selector

CSS selectors are parsed into a `Selector` enum tree in `src/queryselector/`. The iterator (`QuerySelectorIterator`) walks the flat `Parser::tags` arena and calls `Selector::matches()` on each node. Supported combinators: tag, id (`#`), class (`.`), `*`, `,` (or), ` ` (descendant), `>` (parent), attribute `[attr]`, `[attr=val]`, `[attr~=val]`, `[attr^=val]`, `[attr$=val]`, `[attr*=val]`.

### Lifetime discipline

`VDom<'a>` borrows the input `&str` with lifetime `'a`. All `Bytes`, `HTMLTag` names, attribute values, and raw spans are zero-copy slices into that original string — no allocations for the parse result itself. When an owned version is needed (e.g., to store across threads), use `tl::parse_owned()` which returns `VDomGuard` — this leaks the string internally and frees it on drop. `VDomGuard` is `Send + Sync`.

### Void tags

`src/parser/constants.rs` lists the 15 HTML void tags (`br`, `img`, `input`, etc.) that must never push onto the parent stack. When adding or removing void tags, update this constant.
