# Formatting and Metadata

This document explains JumpCut's CLI formatting controls, `fmt` metadata, and metadata-file prepending behavior in terms of what they change and when you would use them.

## CLI Render Profile Override

Render profiles are the quickest way to change the overall feel of a screenplay output without hand-tuning every margin or continuation rule. A profile bundles together pagination and style settings that are meant to work as a coherent set.

The CLI can override the metadata-driven render profile for `pdf`, `text`, or `html` output:

```sh
jumpcut -f pdf --render-profile industry script.fountain script.pdf
jumpcut -f text --render-profile balanced --paginate script.fountain
```

The two built-in profiles are:

- `industry`: the default behavior. This aims for the kind of pagination, page-break decisions, and `(MORE)` / `(CONT'D)` behavior used by major industry tools (like Final Draft).
- `balanced`: a more opinionated profile. This keeps JumpCut in the same general screenplay format, but prefers cleaner dash wrapping and less intrusive continuation behavior when that produces nicer-looking pages.

If both Fountain metadata and `--render-profile` are present, the CLI option wins. The override only replaces profile-level `fmt` tokens such as `balanced`, `clean-dashes`, and `no-dual-contds`; unrelated knobs like `allow-lowercase-title` or `dl-*` / `dr-*` are left alone.

You can also use the CLI to suppress continued markers when you want cleaner reading copies, drafts for informal review, or outputs that should not advertise every page split:

```sh
jumpcut -f text --paginate --no-continueds script.fountain
jumpcut -f pdf --render-profile balanced --no-continueds script.fountain script.pdf
```

And you can suppress title-page output for HTML and PDF without deleting title-page metadata from the source file:

```sh
jumpcut -f html --no-title-page script.fountain
jumpcut -f pdf --no-title-page script.fountain script.pdf
```

## Formatting Metadata (`fmt`)

JumpCut uses the `fmt` metadata key to control shared layout and rendering behavior. In practice, `fmt` is where you say things like:

- "Use A4 instead of Letter."
- "Make scene headings bold and underlined."
- "Nudge dialogue margins."
- "Prefer a more balanced pagination style."

These options can affect pagination itself and, depending on the option, can also influence FDX, HTML, text, and PDF output.

To use these options, add a `fmt` key to your screenplay's metadata at the top of the document, followed by a space-separated list of options.

It helps to think of `fmt` options in three groups:

- profile/template options such as `multicam`, `a4`, and `balanced` choose a starting point
- geometry/style overrides such as `double-spaced-dialogue`, `dl-*`, and `dr-*` fine-tune that starting point
- behavior flags such as `allow-lowercase-title`, `clean-dashes`, and `no-dual-contds` tweak how the renderer behaves without redefining the whole layout

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

#### Profiles and Starting Points

- `multicam`: Starts from a multicamera-TV style layout. Use this when the script is fundamentally a multicam document, not just a single-cam screenplay with a few custom tweaks.
- `a4`: Switches the page from US Letter to A4 and updates the default lines-per-page to match. Use this when the output is meant for A4 paper rather than US screenplay defaults.
- `balanced`: Switches to the more opinionated pagination/render profile. Use this when you want pages that read more smoothly and are less rigidly tied to Final Draft quirks.

#### Styling and Layout Tweaks

- `bold-scene-headings`: Makes scene headings bold. Alias: `bsh`.
- `underline-scene-headings`: Underlines scene headings. Alias: `ush`.
- `all-caps-action`: Converts action text to uppercase. Alias: `acat`.
- `single-space-before-scene-headings`: Reduces the extra blank space before scene headings. Alias: `ssbsh`.
- `double-spaced-dialogue`: Double-spaces dialogue lines. Alias: `dsd`.
- `no-auto-act-breaks`: Keeps `NEW ACT` blocks from forcing a new page.
- `no-act-underlines`: Removes the default underline styling from cold openings and act markers.
- `courier-final-draft`: Uses "Courier Final Draft" instead of the default "Courier Prime". Alias: `cfd`.

#### Geometry Overrides

These are the low-level knobs for directly tweaking element and page margins.

- `dl-X.XX`: Sets the dialogue left indent in inches.
- `dr-X.XX`: Sets the dialogue right indent in inches.
- `tm-X.XX`: Sets the page top margin in inches.
- `bm-X.XX`: Sets the page bottom margin in inches.
- `hm-X.XX`: Sets the page header margin in inches.
- `fm-X.XX`: Sets the page footer margin in inches.
- `lpp-X.XX`: Overrides the default body-page line count used by pagination and layout.

#### Behavior Flags

- `allow-lowercase-title`: Preserves the original title casing for a plain title page. Useful when you want authored title case instead of automatic uppercase.
- `clean-dashes`: Keeps interruption dashes and trailing `--` together instead of following Final Draft-style dash splitting. Useful when you prefer cleaner-looking wraps over strict Final Draft parity.
- `no-dual-contds`: Prevents dual-dialogue blocks from triggering continuation counting rules. Useful when dual dialogue would otherwise create too many `(CONT'D)`-style markers.

Scene-heading style flags can be combined:

```text
Fmt: bold-scene-headings underline-scene-headings
```

### Combined Example

To start from the multicam template, add bold and underlined scene headings, and then override the dialogue margins explicitly:

```text
Fmt: multicam bold-scene-headings underline-scene-headings all-caps-action dl-2.0 dr-5.5
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

These page-metric overrides affect the shared layout profile used by pagination and by renderers that honor shared page geometry. They are most useful when you are matching a house style, a printer target, or an external pagination reference.

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

JumpCut can prepend content from a separate Fountain file as metadata to your main screenplay. This is useful when you want to keep shared metadata such as `title`, `author`, `copyright`, or `fmt` in one reusable place instead of copying it into every script file.

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

## FDX Title Pages In Fountain Output

JumpCut can convert `.fdx` documents into Fountain, and it attempts to preserve title pages and "frontmatter" like opening quote pages or cast lists:

- JumpCut parses FDX title pages to extract metadata (`Title`, `Credit`, `Author`, `Source`, `Draft`, `Draft date`, and similar keys when available)
- Extra title-section pages (cast list, etc) are turned into ordinary Fountain body content, separated with forced page breaks (`===`)
- To keep page numbering accurate, JumpCut writes a metadata key:

```text
Frontmatter-page-count: N
```

`frontmatter-page-count` means:

- the number of extra front matter pages after the main title page
- not the total title-section page count

This keeps front matter pages from ballooning the screenplay page count.

Example:

```text
Title: GUY TEXT
Credit: by
Author: Aaron Brownstein & Simon Ganz
Frontmatter-page-count: 1

THE GUYS

JEFF

NATHAN

ROSS

ETC.

===

INT. OFFICE - DAY
```

This metadata key is automatically added for you if you are importing a final draft document with multiple title pages. But you can also add it yourself if you are writing fountain documents and are trying to add a cast page or vanity quote page and don't want it to count in the page numbers.
