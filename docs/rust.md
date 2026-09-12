# Using JumpCut from Rust

The [README](../README.md#use-in-an-application) shows how to add JumpCut and convert a screenplay.

## HTML options

Use `to_html_with_options` when you need control over the HTML output. This example includes Courier Prime in the file:

```rust
let mut screenplay = jumpcut::parse(&content);
let html = screenplay.to_html_with_options(jumpcut::rendering::html::HtmlRenderOptions {
    head: true,
    exact_wraps: false,
    paginated: true,
    embed_courier_prime: true,
    embedded_courier_prime_css: None,
    ..Default::default()
});
```

Without embedded fonts, the HTML uses installed fonts, falling back to Courier. Embedding fonts makes the file larger.

## Migrating from 1.x to 2.0

The default pagination profile is now `jumpcut`. Select `industry` explicitly
when you want Final Draft-compatible pagination.

`Screenplay::to_fdx()` replaces `to_final_draft()`. The old method remains as a
deprecated alias. Applications using the lower-level pagination APIs also need
these updates:

- Configure pagination behavior through `PaginationPolicy::jumpcut()` or
  `PaginationPolicy::industry()`. `PaginationConfig.policy` replaces
  `PaginationConfig.interruption_dash_wrap`.
- Use `ScreenplayLayoutProfile.pagination` for interruption-dash wrapping and
  dialogue-continuation settings. Orphan and widow limits now belong to
  `PaginationPolicy`, rather than `LayoutGeometry`.
- Replace `compose_with_mode` with `compose_with_wrap`, and
  `paginate_with_mode` with `paginate_with_policy`, passing a policy reference
  to the latter.
- Replace `InterruptionDashWrap::FinalDraft` with `InterruptionDashWrap::Industry`
  and `WrapConfig::from_geometry_final_draft` with `from_geometry_industry`.
- Update direct struct initializers for the added layout-override fields. Prefer
  `..Default::default()` where the type supports it.
