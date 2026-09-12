# Formatting and metadata

## Choose a pagination profile

Profiles control wrapping and page breaks:

- `jumpcut` is the default. It keeps em-dashes together and avoids starting a continuation page with a parenthetical.
- `industry` aims for page breaks closer to Final Draft’s and is less customizable.

```sh
jumpcut script.fountain script.pdf --render-profile industry
```

You can also choose a profile in the script’s `Fmt` metadata (see below), but command-line options take precedence.

## Change the whole document with `fmt`

Fountain files can begin with yaml metadata. If you use a special "Fmt" key in this metadata, you can pass through options to customize how JumpCut outputs your screenplay.

For example, to use A4 paper and bold scene headings:

```text
Title: Night Shift
Author: Alex Morgan
Fmt: a4 bold-scene-headings
```

Capitalization doesn't matter. Each option should be separated by a space.

Other options include underlined headings, double-spaced dialogue, and multicamera TV formatting. See the [formatting reference](formatting-reference.md) for the full list, including margin settings.

## Adjust or Cheat One Paragraph

Add `.lift` or `.widen` inside a Fountain note to adjust a single paragraph:

```text
Jack smells the liquor. [[ .lift .widen-3 ]]
```

This removes one line of space before the paragraph and widens it by three eighths of an inch. `.lift-N` removes N lines; `.widen-N` adds N eighths of an inch. Without a number, each uses 1.

When converting an FDX file with these kinds of paragraph adjustments, JumpCut adds modifier notes to the Fountain document so the adjustments survive conversion. See the [reference](formatting-reference.md#paragraph-adjustments) for details.

## Reuse metadata across scripts

Keep shared metadata in a separate Fountain file and pass it with `--metadata`:

```sh
jumpcut script.fountain script.pdf --metadata common.fountain
```

With no filename, `--metadata` looks for `metadata.fountain` beside the input file, or in the current directory when reading from stdin.

## Title pages and cast lists

JumpCut tries to preserve title pages when converting FDX to Fountain. Extra pages, such as cast lists, become Fountain text separated by page breaks. It adds `Frontmatter-page-count` so those pages don’t count toward screenplay page numbering.

You can also [set this count yourself](formatting-reference.md#front-matter). To omit the title page from HTML or PDF, use `--no-title-page`.
