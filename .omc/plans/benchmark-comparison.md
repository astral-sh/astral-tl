# Plan: Criterion Comparison Benchmarks — astral-tl fork vs upstream

**Status: approved**

## Requirements Summary

1. Extend `benches/tl.rs` to benchmark both the local fork and the original `tl` crate from crates.io in the same `cargo bench` run.
2. Existing `tl` / `pypi_simple` parse benchmarks get a second variant for the upstream so criterion produces a side-by-side group chart.
3. Add a new `selector` benchmark group that exercises the newly-fixed `>` and descendant combinators. These selectors are intentionally absent from the upstream variant (they return zero matches on upstream — document this clearly).
4. Criterion writes HTML reports automatically to `target/criterion/`; the plan also adds a one-liner shell alias to display a plain-text comparison table via `critcmp`.

## Acceptance Criteria

| # | Criterion | Testable? |
|---|-----------|-----------|
| 1 | `cargo bench` compiles cleanly with no errors | `cargo build --benches` exits 0 |
| 2 | Existing benchmarks (`tl`, `pypi_simple`) produce two variants each: `astral-tl` and `upstream-tl` | criterion HTML report shows two bars per group |
| 3 | New `selector/*` benchmark group exists with at least 4 named selectors | HTML report shows `selector/simple`, `selector/child`, `selector/descendant`, `selector/multi_hop` groups |
| 4 | Each selector benchmark runs without panic | `cargo bench -- selector` exits 0 |
| 5 | `upstream-tl` selector benchmarks document correct vs. zero-match behavior in code comments | verified by reading the bench file |
| 6 | HTML report is generated at `target/criterion/report/index.html` | file exists after `cargo bench` |

## Implementation Steps

### Step 1 — Add upstream `tl` as a renamed dev-dependency (`Cargo.toml`)

File: `Cargo.toml`, `[dev-dependencies]` block

```toml
[dev-dependencies]
criterion = { version = "0.3", features = ["html_reports"] }
tl_upstream = { package = "tl", version = "0.7.11" }
```

Cargo resolves `tl_upstream` as the Rust crate name, avoiding a collision with the local lib name `tl`.

### Step 2 — Rewrite `benches/tl.rs`

File: `benches/tl.rs` (full replacement)

Structure:
- BenchmarkGroup "parse/example_domain": astral-tl vs upstream-tl using INPUT const
- BenchmarkGroup "parse/pypi_simple": astral-tl vs upstream-tl using PYPI_SIMPLE const
- BenchmarkGroup "selector/simple": `div` selector — both impls
- BenchmarkGroup "selector/child": `div > p` — astral-tl finds matches, upstream returns 0 (bug)
- BenchmarkGroup "selector/descendant": `div p` — astral-tl finds matches, upstream returns 0
- BenchmarkGroup "selector/multi_hop": `html > body > div` — astral-tl finds matches, upstream returns 0

New HTML fixture `SELECTOR_HTML`: blog-like page with 3-4 levels of nesting, ~80+ tags.

Key implementation notes:
- Use BenchmarkGroup (criterion 0.3 API) so each group gets its own comparison chart
- For selector benchmarks, parse the DOM OUTSIDE the b.iter() closure (parse cost is separate)
- Only measure `query_selector(...).count()` inside the hot loop
- Add `// NOTE: upstream-tl returns 0 matches for this selector (combinator bug)` on upstream combinator variants
- Call `black_box()` on the count to prevent optimizer elision
- Use `tl_upstream` as the extern crate name for the upstream variant

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `tl_upstream = { package = "tl", version = "0.7.11" }` under `[dev-dependencies]` |
| `benches/tl.rs` | Full rewrite: BenchmarkGroup wrappers on existing benches + SELECTOR_HTML fixture + 4 selector benchmark groups |

No library source files change.

## Verification Steps

1. `cargo build --benches` — must compile clean
2. `cargo bench -- parse` — both `astral-tl` and `upstream-tl` variants appear in terminal output
3. `cargo bench -- selector` — all 4 selector groups appear; no panics
4. Open `target/criterion/report/index.html` — verify side-by-side bars in each group
5. `cargo bench -- selector/child` — confirm astral-tl shows non-zero throughput
