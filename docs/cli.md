# Command-line guide

JumpCut reads a Fountain or FDX screenplay and converts it to another format. The simplest form is:

```sh
jumpcut input.fountain output.pdf
```

JumpCut will infer the format from the file extension. But this can be written more explicitly:

```sh
jumpcut input.fountain --format pdf --output output.pdf
```

If you want the same filename you can specify just the format and use `--write`:

```sh
jumpcut input.fountain --format pdf --write
# This produces input.pdf in the same folder as input.fountain
```

## Common examples

```sh
# Paginated plain text
jumpcut script.fountain --format text --paginate --output script.txt

# HTML with screenplay line wraps
jumpcut script.fountain --format html --exact-wraps --output script.html

# Read from stdin
cat script.fountain | jumpcut - --format json
```

Supported output formats are `fountain`, `fdx`, `html`, `json`, `text`, and `pdf`.

## Options

| Option | What it does |
| --- | --- |
| `-f, --format FORMAT` | Choose the output format. |
| `-o, --output FILE` | Write to a specific file. |
| `-w, --write` | Derive the output filename from the input and format. |
| `--paginate` | Add pagination to text output. |
| `--exact-wraps` | Preserve screenplay line wraps in HTML output. |
| `--line-numbers` | Add line numbers to text output. |
| `--render-profile PROFILE` | Use `jumpcut` or `industry` pagination behavior. |
| `--no-continueds` | Omit `(MORE)` and `(CONT'D)` markers. |
| `--no-title-page` | Omit the title page from HTML and PDF. |
| `--metadata [FILE]` | Prepend metadata from a Fountain file. With no file, look for `metadata.fountain`. |
| `--embed-courier-prime` | Put the screenplay font directly into HTML. |
| `--font-dir DIR` | Use a custom four-face TrueType font family for PDF or embedded-font HTML. |
| `-h, --help` | Show help. |
| `-V, --version` | Show the installed version. |

## Fonts and PDF details

PDF output uses the bundled Courier Prime family by default. To use another family, provide all four faces in a directory:

```text
my-fonts/
├── regular.ttf
├── bold.ttf
├── italic.ttf
└── bold-italic.ttf
```

Then pass the directory when creating a PDF:

```sh
jumpcut script.fountain --format pdf --font-dir ./my-fonts --output script.pdf
```

For HTML, add `--embed-courier-prime` as well. See [Embedded HTML fonts](html-embedded-fonts.md) for the browser and WASM APIs.

PDF files include bookmarks for scene headings. The title page is included by default; use `--no-title-page` when you want only the screenplay pages.

## Metadata and formatting

For reusable metadata, profiles, page geometry, and one-off layout adjustments, see [Formatting and metadata](formatting-and-metadata.md).
