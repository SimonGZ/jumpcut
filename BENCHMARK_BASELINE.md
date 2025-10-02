# WASM Rewrite Baseline Benchmarks

Command: `cargo bench`
Date: 2025-10-01T22:29:45Z

| Benchmark | Min | Median | Max |
| --- | --- | --- | --- |
| Parse/Parse/108 | 505.21 µs | 506.45 µs | 507.85 µs |
| Parse/Parse/Big Fish | 1.5295 ms | 1.5326 ms | 1.5360 ms |

Notes:
- Criterion reported no statistically significant change relative to prior run.
- Outliers: Parse/Parse/108 had 3/100 high outliers (2 mild, 1 severe); Parse/Parse/Big Fish had 6/100 high outliers (5 mild, 1 severe).

## no-regex-parser Feature Benchmarks

Command: `cargo bench --features no-regex-parser`
Date: 2025-10-01T22:45:12Z

| Benchmark | Min | Median | Max |
| --- | --- | --- | --- |
| Parse/Parse/108 | 1.2919 ms | 1.2947 ms | 1.2977 ms |
| Parse/Parse/Big Fish | 4.2905 ms | 4.3016 ms | 4.3133 ms |

Notes:
- Regression vs baseline (+153% small sample, +182% large sample).
- All runs executed with `no-regex-parser` enabled; investigate marker scanner hot paths.

## no-regex-parser Benchmarks (stack-based pass)

Command: `cargo bench --features no-regex-parser`
Date: 2025-10-01T22:55:49Z

| Benchmark | Min | Median | Max |
| --- | --- | --- | --- |
| Parse/Parse/108 | 651.22 µs | 652.70 µs | 654.39 µs |
| Parse/Parse/Big Fish | 2.1398 ms | 2.1492 ms | 2.1591 ms |

Notes:
- Linear delimiter stack cut runtime roughly in half versus the first prototype, but still trails the regex baseline by ~29%.
- Criterion emitted mild high outliers for both benches; consider longer warmups when iterating further.
- The regex-free parser is now the default build; the `no-regex-parser` feature remains as a no-op for compatibility.

## StyleMask Bitset Migration Benchmarks

Command: `cargo bench --bench jumpcut_benchmarks`
Date: 2025-10-02T06:31:12Z

| Benchmark | Min | Median | Max |
| --- | --- | --- | --- |
| Parse/Parse/108 | 564.62 µs | 567.08 µs | 570.20 µs |
| Parse/Parse/Big Fish | 1.8753 ms | 1.8818 ms | 1.8889 ms |
| Markup/Markup/dense | 670.66 ns | 671.88 ns | 673.27 ns |
| Markup/Markup/multi_line | 1.0109 µs | 1.0126 µs | 1.0146 µs |

Notes:
- Switching `TextRun` styles to the `StyleMask` bitset trimmed markup hot paths by ~54–55% and nudged full-script parses ~1–2% faster.
- Criterion flagged statistically significant gains for the Markup group; Parse improvements are within expected noise but consistently positive.
- Runs used the default feature set; no configuration changes beyond the mask refactor.
