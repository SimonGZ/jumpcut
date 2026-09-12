# Writing Fountain for JumpCut

Fountain is a plain-text screenplay format. Write scene headings, action, character names, and dialogue like this:

```text
Title: Night Shift
Author: Alex Morgan

INT. HOTEL - NIGHT

Alex watches the revolving door.

ALEX
(quietly)
We’re closed.
```

JumpCut recognizes scene headings, action, dialogue, parentheticals, transitions, title pages, scene numbers, dual dialogue, lyrics, notes, sections, and synopses. See the [Fountain syntax guide](https://fountain.io/syntax/) for the full syntax.

## Useful marks

| Mark | Meaning |
| --- | --- |
| `.Heading` | Force a scene heading. |
| `!Action` | Force an action paragraph. |
| `@Name` | Force a character cue, including names with lowercase letters. |
| `NAME ^` | Put this character’s dialogue beside the preceding dialogue. |
| `>FADE OUT.` | Force a transition. |
| `> Text <` | Center text. |
| `~Text` | Lyrics. |
| `*text*`, `**text**`, `***text***` | Italic, bold, or both. |
| `_text_` | Underline. |
| `[[note]]` | A note. |
| `/* comment */` | A comment. |
| `===` | A page break. |

## JumpCut additions

Use `Fmt` metadata to change document formatting, and `.lift` or `.widen` notes to adjust individual paragraphs. These additions are described in the [formatting guide](formatting-and-metadata.md).
