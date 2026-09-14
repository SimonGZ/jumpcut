# Formatting reference

 Put these options on a `Fmt:` line in the yaml frontmatter of a fountain file, separated by spaces. Names are case-insensitive.

Example:

```text
Title: Night Shift
Author: Alex Morgan
Fmt: industry bold-scene-headings single-space-before-scene-headings no-auto-act-breaks
```


## Profiles and paper sizes

- `multicam`: Use a multicamera TV layout.
- `a4`: Use A4 paper instead of US Letter, with a matching line count.
- `jumpcut`: Use the default opinionated pagination profile.
- `industry`: Use Final Draft-style pagination.

## Styles

- `bold-scene-headings`: Makes scene headings bold. Alias: `bsh`.
- `underline-scene-headings`: Underlines scene headings. Alias: `ush`.
- `all-caps-action`: Converts action text to uppercase. Alias: `acat`.
- `single-space-before-scene-headings`: Reduces the extra blank space before scene headings. Alias: `ssbsh`.
- `double-spaced-dialogue`: Double-spaces dialogue lines. Alias: `dsd`.
- `no-auto-act-breaks`: Keeps `NEW ACT` blocks from forcing a new page.
- `no-act-underlines`: Removes the default underline styling from cold openings and act markers.
- `courier-final-draft`: Uses "Courier Final Draft" instead of the default "Courier Prime". Alias: `cfd`.

## Element Margins / Settings

You can customize the margins for different element types. Measurements below are in inches.

| Element | Left edge | Right edge |
| --- | --- | --- |
| Action | `al-X.XX` | `ar-X.XX` |
| Scene heading | `shl-X.XX` | `shr-X.XX` |
| Character | `cl-X.XX` | `cr-X.XX` |
| Dialogue | `dl-X.XX` | `dr-X.XX` |
| Parenthetical | `pl-X.XX` | `pr-X.XX` |
| Transition | `tl-X.XX` | `tr-X.XX` |
| Lyrics | `ll-X.XX` | `lr-X.XX` |
| Cold opening | `col-X.XX` | `cor-X.XX` |
| New act | `nal-X.XX` | `nar-X.XX` |
| End of act | `eal-X.XX` | `ear-X.XX` |

Both edges are absolute positions measured from the **left edge of the page**; the right value is not a distance from the right edge. Each tag changes only one edge. Omitted edges keep their profile defaults. Tags are case-insensitive, and the last valid occurrence of a tag wins. Use nonnegative finite values with the right edge greater than the left edge. Decimal precision is not limited to two places (for example, `lr-7.375`).

```text
Fmt: industry al-1.75 ar-7.25 cl-3.75 dl-2.50 dr-6.00
```

Centered elements are centered between their own edges. Dialogue tags apply to ordinary dialogue; dual dialogue is its own special case and can't be customized at this time.

Imported FDX element margins and page margins are written to these `Fmt` tags when exporting Fountain, preserving their values when reopened in Jumpcut. NOTE: No other Fountain converters honor these tags. They are currently JumpCut specific.

JumpCut will try to honor custom element margins in PDF and all HTML modes, including fractional values such as `2.125`. Ordinary HTML output will adapt to narrower viewports (like mobile phones), but exact-wrap and paginated HTML will not automatically adapt to smaller screens. 

## Page Margins

You can also customize page margins.

- `tm-X.XX`: Sets the page top margin in inches.
- `bm-X.XX`: Sets the page bottom margin in inches.
- `hm-X.XX`: Sets the page header margin in inches.
- `fm-X.XX`: Sets the page footer margin in inches.
- `lpp-X.XX`: Overrides the default body-page line count (it changes the number of lines the pagination engine tries to fit on a page).

## Other options

- `allow-lowercase-title`: Keep the title’s original capitalization on a plain title page.
- `clean-dashes`: Keep interruption dashes and trailing `--` together. Enabled by default in the `jumpcut` profile.
- `no-dual-contds`: Exclude dual dialogue when counting automatic character continuations.

## Cheats / Paragraph adjustments

If you're trying to save space and you have a single element that you want to manually tweak (without tweaking all other elements of the same type), JumpCut lets you change the margins and spacing of individual elements by sticking special markup in fountain notes.

Use `.lift` and `.widen` inside `[[ ... ]]` notes:

| Modifier | Effect |
| --- | --- |
| `.lift` or `.lift-1` | Remove one line of space before the paragraph. |
| `.lift-N` | Remove N lines of space before the paragraph. |
| `.widen` or `.widen-1` | Widen the paragraph by 0.125 inches. |
| `.widen-N` | Widen the paragraph by N × 0.125 inches. |

Multiple modifiers add together, whether they share a note or appear in separate notes. Other text in the note is preserved. Unrecognized modifiers remain ordinary notes.

If you import from a Final Draft document that has individual paragraph cheats like these, JumpCut will try to preserve them and translate them into Fountain notes.

## Front matter / Vanity Quote Pages

Sometimes a screenplay begins with a quote page or cast page that shouldn't count against the script's page count. You can use the `Frontmatter-page-count` metadata option to exclude these pages as shown below:

```text
Title: Night Shift
Author: Alex Morgan
Frontmatter-page-count: 1

CAST

ALEX — the night manager

===

INT. HOTEL - NIGHT
```

JumpCut will attempt to add this metadata itself if you import from a Final Draft file with extra title pages.
