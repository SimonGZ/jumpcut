# Formatting and Metadata

This document covers JumpCut's CLI formatting controls, `fmt` metadata, and metadata-file prepending behavior.

## CLI Render Profile Override

The CLI can override the metadata-driven render profile for `pdf`, `text`, or `html` output:

```sh
jumpcut -f pdf --render-profile final-draft script.fountain script.pdf
jumpcut -f text --render-profile balanced --paginate script.fountain
```

If both Fountain metadata and `--render-profile` are present, the CLI option wins. The override only replaces profile-related `fmt` tokens such as `balanced`, `clean-dashes`, and `no-dual-contds`; unrelated `fmt` knobs like `allow-lowercase-title` or `dl-*` / `dr-*` are preserved.

You can also suppress continued markers from the CLI:

```sh
jumpcut -f text --paginate --no-continueds script.fountain
jumpcut -f pdf --render-profile balanced --no-continueds script.fountain script.pdf
```

You can also suppress title-page output for HTML and PDF without removing title-page metadata from the source:

```sh
jumpcut -f html --no-title-page script.fountain
jumpcut -f pdf --no-title-page script.fountain script.pdf
```

## Formatting Metadata (`fmt`)

JumpCut uses the `fmt` metadata key to control shared layout and rendering behavior. These options affect pagination and can also influence FDX, HTML, text, and PDF output, depending on the option.

To use these options, add a `fmt` key to your screenplay's metadata at the top of the document, followed by a space-separated list of options.

JumpCut treats `fmt` options in two layers:

- template options such as `multicam` establish a base layout/style profile
- explicit geometry knobs such as `single-space-before-scene-headings`, `double-spaced-dialogue`, `dl-*`, and `dr-*` then override that base, regardless of where they appear in the `fmt` string
- render/style options such as `allow-lowercase-title` and `clean-dashes` adjust output behavior without changing the shared page geometry

The `fmt` parser is whitespace-based and case-insensitive for the supported option names, so these are equivalent:

```text
Fmt: a4 balanced
fmt: A4 BALANCED
```

Example:

```text
Title: My Awesome Screenplay
Author: John Doe
Fmt: bold-scene-headings underline-scene-headings all-caps-action double-spaced-dialogue dl-1.5 dr-7.0
```

### Available `fmt` Options

- `multicam`: Applies JumpCut's shared multicam layout profile as a starting point for pagination and rendering.
- `a4`: Switches the shared page geometry from US Letter to A4 and updates the default lines-per-page accordingly.
- `balanced`: Enables cleaner dash wrapping and disables dual-dialogue continuation counting as a combined profile toggle.
- `bold-scene-headings`: Makes scene headings bold. Alias: `bsh`.
- `underline-scene-headings`: Underlines scene headings. Alias: `ush`.
- `all-caps-action`: Converts action text to uppercase. Alias: `acat`.
- `single-space-before-scene-headings`: Reduces the space before scene headings from 24 points to 12 points. Alias: `ssbsh`.
- `double-spaced-dialogue`: Changes dialogue spacing from single to double. Alias: `dsd`.
- `no-auto-act-breaks`: Keeps `NEW ACT` blocks from forcing a new page.
- `no-act-underlines`: Removes the default underline styling from cold openings and act markers.
- `courier-final-draft`: Uses "Courier Final Draft" instead of the default "Courier Prime". Alias: `cfd`.
- `dl-X.XX`: Sets the dialogue left indent in inches.
- `dr-X.XX`: Sets the dialogue right indent in inches.
- `tm-X.XX`: Sets the page top margin in inches.
- `bm-X.XX`: Sets the page bottom margin in inches.
- `hm-X.XX`: Sets the page header margin in inches.
- `fm-X.XX`: Sets the page footer margin in inches.
- `lpp-X.XX`: Overrides the default body-page line count used by pagination and renderer layout.
- `allow-lowercase-title`: Preserves the original title casing for an unstylized title page.
- `clean-dashes`: Keeps interruption dashes and trailing `--` together instead of following Final Draft-compatible dash splitting.
- `no-dual-contds`: Prevents dual-dialogue blocks from triggering continuation counting rules.

The short forms remain accepted, but the long forms are now the preferred documented names.

Scene-heading style flags can be combined:

```text
Fmt: bold-scene-headings underline-scene-headings
```

### Combined Example

To start from the multicam template, keep its double-spaced dialogue, and then override the dialogue margins explicitly:

```text
Fmt: multicam bold-scene-headings underline-scene-headings all-caps-action double-spaced-dialogue dl-2.0 dr-5.5
```

### Page Geometry Examples

Switch to A4 page geometry:

```text
Fmt: a4
```

Switch to A4 and explicitly tune page metrics:

```text
Fmt: a4 tm-1.0 bm-1.0 hm-0.5 fm-0.5 lpp-58
```

Keep Letter size but override the usable page metrics:

```text
Fmt: tm-1.1 bm-1.0 hm-0.4 fm-0.5 lpp-55
```

These page-metric overrides affect the shared layout profile used by pagination and by renderers that honor shared page geometry.

### Title and Dash Examples

Preserve the original title casing for a plain title page:

```text
Fmt: allow-lowercase-title
```

Keep interruption dashes together instead of following Final Draft's line-wrap behavior:

```text
Fmt: clean-dashes
```

Disable only dual-dialogue continuation counting without switching the dash-wrap policy:

```text
Fmt: no-dual-contds
```

## Prepending Metadata

JumpCut allows you to prepend content from a separate Fountain file as metadata to your main screenplay. This is useful for managing common metadata such as `title`, `author`, `copyright`, or `fmt` across multiple files.

Use `--metadata` or `-m` to specify a metadata file:

```sh
jumpcut <screenplay-file> --metadata <metadata-file>
jumpcut <screenplay-file> -m <metadata-file>
```

If you provide `--metadata` without a file path, JumpCut looks for a file named `metadata.fountain`:

- if the input is a file, JumpCut looks in the same directory as that input file
- if the input is stdin (`-`), JumpCut looks in the current working directory

Examples:

```sh
# Looks for 'metadata.fountain' next to 'my_screenplay.fountain'
jumpcut my_screenplay.fountain -m -f fdx > my_screenplay.fdx

# Looks for 'metadata.fountain' in the current working directory
cat my_screenplay.fountain | jumpcut - -m -f html > my_screenplay.html

# Uses an explicit metadata file
jumpcut -m ~/my_templates/common_header.fountain my_screenplay.fountain -f json > my_screenplay.json
```
