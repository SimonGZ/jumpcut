# Formatting and metadata

JumpCut tries to give you the best possible layout and pagination by default, but there are settings and profiles (collections of settings) that you can tweak to change the behavior.

## Choose a pagination profile

A render profile bundles together pagination and style settings that are meant to work as a coherent set.

```sh
jumpcut -f text --render-profile jumpcut --paginate script.fountain
jumpcut -f pdf --render-profile industry script.fountain script.pdf
```

The two built-in profiles are:

- `jumpcut`: the default, opinionated behavior. It's very similar to the industry-standard pagination you'll see in programs like Final Draft, but with a few key differences. It keeps em-dashes together (Final Draft sometimes splits them), it avoids starting a page with a parenthetical, and it defaults to fewer MORES and CONT'Ds in some cases, particularly in dual dialogue. There are also several settings that can be tweaked to further customize this mode.
- `industry`: this closely matches the behavior you'll see in programs like Final Draft, even when that behavior is a bit ugly. If you want the closest possible match to Final Draft, this is the best choice, but it's less customizable.

If both [Fountain metadata](#change-the-whole-document-with-fmt) and `--render-profile` are present, the CLI option wins. The override replaces profile-level `fmt` tokens, including legacy aliases, while leaving unrelated knobs such as `allow-lowercase-title` and `dl-*` / `dr-*` alone.

## Change the whole document with `fmt`

JumpCut can read a special metadata key (`fmt`) from the top of fountain documents that sets rendering and layout options like:

- "Use A4 instead of Letter."
- "Make scene headings bold and underlined."
- "Nudge dialogue margins."
- "Use JumpCut's opinionated pagination style."

To use these options, add a `fmt` key to your screenplay's metadata at the top of the document, followed by a space-separated list of options.

Example:

```text
Title: My Awesome Screenplay
Author: John Doe
Fmt: bold-scene-headings underline-scene-headings all-caps-action double-spaced-dialogue dl-1.5 dr-7.0
```

The `fmt` parser is whitespace-based and case-insensitive, so these are equivalent:

```text
Fmt: a4 jumpcut
fmt: A4 JUMPCUT
```

### Available `fmt` Options

#### Profiles and Starting Points

- `multicam`: Starts from a multicamera-TV style layout. Use this when the script is fundamentally a multicam document, not just a single-cam screenplay with a few custom tweaks.
- `a4`: Switches the page from US Letter to A4 and updates the default lines-per-page to match. Use this when the output is meant for A4 paper rather than US screenplay defaults.
- `jumpcut`: Selects the default opinionated pagination profile.
- `industry`: Selects Final Draft-style pagination.

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
- `clean-dashes`: Keeps interruption dashes and trailing `--` together. This is already the Jumpcut default and is most useful as an explicit override when combining older metadata.
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

## Adjust one paragraph

`fmt` is the right tool when you want to change the baseline layout for the whole document. Per-element layout overrides are for the opposite case: when one specific paragraph or line needs to sit a little higher or wrap a little wider without redefining the screenplay's overall format.

JumpCut supports these overrides in two ways:

- Fountain modifier notes such as `[[ .lift-1 ]]` or `[[ .widen-2 ]]`. You can write these yourself into a fountain document.
- Imported FDX paragraph-level deviations such as `SpaceBefore` or `RightIndent` values that differ from the element's baseline style. JumpCut will automatically add these to Fountain that is converted from an FDX with existing cheats/overrides.

These overrides affect pagination, text output, PDF output, visual wrapping, and FDX / Fountain serialization instead of being treated as format-specific one-offs.

### Modifier Note Syntax

The layout modifiers are placed in Fountain notes (ie `[[ modifier-goes-here ]]`). JumpCut currently recognizes these modifiers:

- `.lift`: reduce the element's space-before by `1` line
- `.lift-N`: reduce the element's space-before by `N` lines
- `.widen`: increase the element's available width by `0.125"` (one eighth inch)
- `.widen-N`: increase the element's available width by `N * 0.125"`

Examples:

```text

Jack smells the liquor. [[ .lift ]][[ .widen-3 ]] /* Removes one line of space before the paragraph and widens by 3/8th-inch */

```

You can combine multiple modifiers in one note or spread them across multiple notes. JumpCut adds them together before layout.

If a modifier note also contains ordinary note text, JumpCut preserves the remaining text as a normal note:

```text
Jack smells the liquor. [[ .lift-1 comment ]]
```

That note still applies the lift, and the remaining `comment` text is preserved as an ordinary note.

If a token does not match a supported modifier form, JumpCut leaves it alone as a regular note instead of trying to guess what you meant.

### FDX Import and Round-Trip Behavior

On FDX import, JumpCut first derives the baseline layout profile from the document's element settings. If an individual body paragraph uses a different `SpaceBefore` or `RightIndent` from that baseline, JumpCut preserves that difference as a per-element override.

That means imported FDX quirks like a slightly raised action line or a widened dialogue line survive through:

- pagination and wrap calculation
- text and PDF rendering
- Fountain export as synthetic modifier notes
- FDX export as paragraph-level `SpaceBefore` / `RightIndent` overrides

### Title and Dash Examples

Preserve the original title casing for a plain title page:

```text
Fmt: allow-lowercase-title
```

Keep interruption dashes together explicitly:

```text
Fmt: clean-dashes
```

Disable only dual-dialogue continuation counting without switching the dash-wrap policy:

```text
Fmt: no-dual-contds
```

## Reuse metadata across scripts

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

## CLI-specific Settings

You can also use the CLI to suppress continued markers when you want cleaner reading copies, drafts for informal review, or outputs that should not advertise every page split:

```sh
jumpcut -f text --paginate --no-continueds script.fountain
jumpcut -f pdf --render-profile jumpcut --no-continueds script.fountain script.pdf
```

And you can suppress title-page output for HTML and PDF without deleting title-page metadata from the source file:

```sh
jumpcut -f html --no-title-page script.fountain
jumpcut -f pdf --no-title-page script.fountain script.pdf
```

## Keep title pages when converting FDX to Fountain

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

This metadata key is automatically added when importing an FDX document with multiple title pages. You can also add it to a Fountain document for a cast page or vanity quote page that should not count toward screenplay page numbering.
