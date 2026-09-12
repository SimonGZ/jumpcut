# Command-line guide

Give JumpCut an input file and an output filename:

```sh
jumpcut script.fountain script.pdf
jumpcut script.fountain script.fdx
jumpcut script.fdx script.fountain
```

Input can be Fountain or FDX. Output can be `fountain`, `fdx`, `pdf`, `html`, `text`, or `json`. JumpCut chooses the output format from the extension, or you can set it with `--format`.

## Output files and pipes

Use `--write` to keep the input’s name with a new extension:

```sh
jumpcut script.fountain --format pdf --write
```

This creates `script.pdf` beside the input. 

If you don't provide an output filename (or pass the `--write`) option, then Jumpcut writes to stdout and you can pipe the text somewhere else.

If you want to pipe something into Jumpcut, use `-` to read from stdin:

```sh
cat script.fountain | jumpcut - --format html
```

## Layout options

JumpCut has a full pagination engine so that it can produce nice looking PDFs, but you can also pass the `--paginate` option to paginate text and html output.

By default, HTML output lets the browser decide where lines break, but if you want the HTML to exactly match the line breaks of the PDF or Final Draft output, you can pass `--exact-wraps`:

```sh
jumpcut script.fountain script.txt --paginate
jumpcut script.fountain script.html --exact-wraps
```

For HTML and PDF, `--no-title-page` leaves out the title page. Use `--no-continueds` to leave out `(MORE)` and `(CONT'D)` markers.

See the [formatting guide](formatting-and-metadata.md) for paper size, page-break profiles, and metadata. Run `jumpcut --help` for the complete option list.

## Fonts

PDFs use the bundled Courier Prime font and include bookmarks for scene headings.

To include Courier Prime in an HTML file, add `--embed-courier-prime`. The file can then display the font on computers where it isn’t installed:

```sh
jumpcut script.fountain script.html --embed-courier-prime
```

To use a different font family, put four TrueType files in a folder:

```text
my-fonts/
├── regular.ttf
├── bold.ttf
├── italic.ttf
└── bold-italic.ttf
```

Pass that folder with `--font-dir`. For HTML, also add `--embed-courier-prime`:

```sh
jumpcut script.fountain script.pdf --font-dir ./my-fonts
jumpcut script.fountain script.html --font-dir ./my-fonts --embed-courier-prime
```

Embedding fonts makes the HTML file larger.
