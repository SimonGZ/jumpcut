# Final Draft 11 pagination reverse engineering

This note records a static-analysis pass over the locally installed Final Draft
11 application. It is intended to guide a compatible JumpCut pagination mode,
not to reproduce Final Draft source code or bypass licensing, activation, or
copy protection.

## Scope and provenance

- Application: Final Draft 11 for macOS
- Version: 11.1.7 (build 96)
- Architecture: x86-64
- Analyzed executable: `Contents/MacOS/Final Draft 11`
- Code-directory hash: `f137ee759018227e4550fc6ef6406b197975e094`
- Team identifier: `7XUZ8R5736`
- Analysis copy: `/tmp/fd11-pagination-research-20260803/Final Draft 11.app`

The installed application was not modified. The application bundle was copied
with `ditto`, and analysis was performed against that temporary copy. No
application binary or other proprietary artifact has been added to this
repository.

## Executive findings

The strongest finding is that Final Draft 11 does not appear to choose page
breaks by globally scoring every legal candidate. Its lower-level text engine
first lays out physical lines and proposes breaks. A screenplay-aware correction
pass then examines a small rolling window of line records around each proposed
break and applies a fixed sequence of special cases.

That is materially different from JumpCut's current model, which generates many
candidates and ranks them by a tuple including sentence preference, page
fullness, balance, and content length. Some of the empirical "scoring" behavior
may instead be an emergent result of Final Draft's ordered local corrections.

Other high-confidence findings:

- `BreakDialogueAndActionAtSentences` is a real, serialized document setting.
  It defaults to enabled when missing and is enabled in the shipped Screenplay
  template.
- When enabled, the sentence correction is applied inside Action,
  Parenthetical, and Dialogue pagination types. It is not restricted to
  Dialogue text.
- The sentence recognizer treats `.`, `!`, `?`, and `;` as terminal punctuation.
- It has explicit case-insensitive abbreviation exceptions for `DR.`, `MR.`,
  `MS.`, `ST.`, `MRS.`, `AVE.`, and `BLVD.`.
- Element pagination behavior is a separate concept from element appearance.
  The FDX `Behavior PaginateAs="..."` attribute controls the pagination role.
  For example, the shipped Lyric style paginates as Dialogue and Shot paginates
  as Scene Heading.
- The screenplay correction pass has separate ordered handlers for Scene
  Heading, Action, Character, Dialogue/Parenthetical, and Transition patterns.
- `PageBreakValid` is not evidence of a legal-break policy flag. It is one of a
  set of `FDParagraphDesc` validity bits, alongside `LeftIndentValid`,
  `SpacingValid`, and similar fields, and means that the descriptor's page-break
  value is present/valid.

## Architecture recovered from RTTI

The executable is stripped, but it retains Itanium C++ RTTI, vtables, Objective-C
class metadata, assertion source paths, and a large number of serialized field
names.

The relevant text hierarchy is:

```text
FDPaigeText
  -> FDMultiPageText
       -> FDPaigeTextPaginated

FDSPText (screenplay text, including an FDMultiPageText base)
FDRTText (rich text, including an FDMultiPageText base)
```

`FDPaigeTextPaginated` does not add a distinct pagination algorithm in its
vtable; apart from destructor entries, its vtable matches `FDMultiPageText`.
Physical layout and page painting live in the Paige/multi-page layer, while
screenplay semantics are supplied by `FDSPText` and its helpers.

Other relevant recovered classes include `FDSPParagraph`,
`FDSPDualDialogueParagraph`, `FDMoreContinued`, and
`FDAutoContinuedsProcessor`. `FDAutoContinuedsProcessor` manages automatic
character `(CONT'D)` state; it is not the primary page-break selector.

## Reconstructed break pipeline

The names below are descriptive names assigned during analysis. The production
binary does not retain function symbols for these routines.

1. The Paige-based engine shapes UTF-16 text into physical line records and
   creates proposed page boundaries.
2. The screenplay pass creates a circular cache of ten nearby line records.
   Each record includes its character offset, pagination type, and line flags.
3. Around a proposed break it evaluates five specialized handlers in this fixed
   order:

   1. Scene Heading
   2. Action
   3. Character
   4. Dialogue/Parenthetical
   5. Transition

4. A handler can return a replacement character offset and, in some cases, a
   nearby line from which the sentence-correction routine should search.
5. If no special handler resolves the break, the sentence-correction routine is
   attempted at the ordinary candidate line.
6. The resulting character offset is validated against actual line boundaries,
   deduplicated, and sent back through the layout engine.

The specialized handlers repeatedly inspect fixed positions 0 through 6 in the
local cache. In particular, positions 4, 5, and 6 straddle the candidate area,
while earlier positions provide look-behind for scene/action/dialogue grouping.
This is a local, ordered rule system rather than an unbounded search with a
global score.

### Pagination type codes

The type mapping was recovered from the executable's type-string initialization
and its string-to-type converter:

| Code | Pagination type |
| ---: | --- |
| 2 | General |
| 3 | Scene Heading |
| 4 | Action |
| 5 | Character |
| 6 | Parenthetical |
| 7 | Dialogue |
| 8 | Transition |
| 9 | Shot |
| 10 | Act |
| 12 | Cast List |
| 13 | New Act |
| 14 | End of Act |

The sentence-correction routine explicitly accepts equal adjacent types only
when the type is 4, 6, or 7: Action, Parenthetical, or Dialogue. A line flag
tested by the routine marks a paragraph boundary; that path is for internal
paragraph correction, not an already valid paragraph boundary.

### What the five local handlers establish

The handlers encode the familiar screenplay keeps directly against neighboring
line records:

- A Scene Heading is kept with following material rather than stranded at the
  bottom of a page.
- Action checks paragraph starts on the lines immediately around the break and
  moves the break to avoid thin fragments.
- Character is kept with dialogue/parenthetical content below it.
- Dialogue and Parenthetical are treated as a group, with multiple lines of
  look-behind through Character and intervening Parenthetical records.
- A Transition at the top side of a proposed page is handled by looking backward
  through the preceding dialogue group or action paragraph.

Static analysis exposes the exact branches and window positions, but line-table
flags still need runtime labels before every branch can be expressed cleanly as
"N orphan lines" or "N widow lines." The important confirmed point is that the
limits arise from fixed neighboring-line patterns; there is no evidence here of
the global candidate-ranking tuple implemented by JumpCut.

## Sentence-boundary behavior

The document setting is stored as a byte in `FDSPText`. The XML reader behaves
as follows:

```text
value = true
if BreakDialogueAndActionAtSentences is present:
    value = parse_yes_no(attribute)
set_break_at_sentences(value)
```

The shipped Screenplay template contains:

```xml
<PageLayout ... BreakDialogueAndActionAtSentences="Yes" ...>
```

When the setting is disabled, the special sentence adjustment exits and the
ordinary physical line break remains available. Sentence-only splitting is
therefore a mode, not an invariant of the Final Draft engine.

When enabled, the routine:

- ignores an already existing paragraph boundary;
- requires the physical lines on both sides to have the same pagination type;
- limits that type to Action, Parenthetical, or Dialogue;
- walks backward through the paragraph text from the proposed wrapped-line
  boundary;
- recognizes ASCII `!`, `.`, `;`, and `?` as terminal punctuation;
- requires the punctuation to be followed by whitespace for its core sentence
  predicate;
- treats a carriage return as an explicit boundary case;
- suppresses false stops after the case-insensitive abbreviations `DR`, `MR`,
  `MS`, `ST`, `MRS`, `AVE`, and `BLVD`; and
- only commits the resulting character position if it corresponds to a valid
  layout-line boundary and has not already been recorded.

This directly differs from JumpCut's current sentence helper, which recognizes
`.`, `!`, and `?`, allows quote/bracket closers, and has no abbreviation
exceptions. It also means a semicolon can affect Final Draft pagination.

## Shipped screenplay defaults

The default template is at
`Contents/Resources/Data/Templates/Scripts/Screenplay.fdxt` in the application
bundle. Relevant defaults include:

| Setting | Default |
| --- | --- |
| Page size | 8.5 × 11 inches |
| Top / bottom margin | 72 pt / 72 pt |
| Header / footer margin | 36 pt / 36 pt |
| Document leading | Normal |
| Sentence correction | Yes |
| Dialogue bottom marker | `(MORE)` |
| Dialogue top marker | `(CONT'D)` |
| Marker at bottom / top of next page | Yes / Yes |
| Automatic character continueds | Yes |
| Scene bottom / top continueds | No / No |

Default screenplay geometry:

| Paginate as | Left | Right | First | Space before |
| --- | ---: | ---: | ---: | ---: |
| Scene Heading | 1.50 | 7.50 | 0.00 | 24 pt |
| Action | 1.50 | 7.50 | 0.00 | 12 pt |
| Character | 3.50 | 7.25 | 0.00 | 12 pt |
| Parenthetical | 3.00 | 5.50 | -0.10 | 0 pt |
| Dialogue | 2.50 | 6.00 | 0.00 | 0 pt |
| Transition | 5.50 | 7.10 | 0.00 | 12 pt |

All measurements in the geometry table are inches except Space before.

## Comparison with JumpCut

### Confirmed gaps

1. JumpCut writes `BreakDialogueAndActionAtSentences="Yes"` when exporting FDX,
   but does not import or store the setting. A parity mode needs this as a real
   boolean, defaulting to `true` when absent from FDX.
2. JumpCut's sentence recognizer omits `;` and does not suppress Final Draft's
   seven hard-coded abbreviations.
3. JumpCut permits mid-part sentence candidates for Dialogue and Lyric, but not
   Parenthetical. Final Draft's internal sentence path explicitly includes
   Parenthetical.
4. JumpCut does not currently import `Behavior PaginateAs`. That loses the
   pagination semantics of custom elements and aliases such as Lyric→Dialogue,
   Shot→Scene Heading, and Cast List→Action.
5. JumpCut's global split-score tuple has no direct analogue in the recovered
   Final Draft path. A faithful compatibility mode should model the proposed
   physical break followed by the ordered local correction pass.

### Hypotheses that should be downgraded

- "Final Draft always splits only at sentence boundaries" is only true when
  the document switch is enabled.
- The three-wrapped-line minimum visible in probes is not implemented as an
  obvious standalone `length >= 3` test in the recovered routine. It is more
  likely an outcome of the local paragraph-start and neighboring-line rules.
- Apparent preferences for fuller or balanced fragments may be consequences of
  starting at the physical break and searching backward, not explicit scoring
  criteria.
- `(MORE)` being budget-neutral remains an empirical observation. The static
  pass has not yet established how continuation-marker insertion feeds back
  into line geometry.

## Recommended implementation direction

Keep the existing paginator as the customizable mode, but introduce a separate
Final Draft 11 compatibility policy with these stages:

1. Produce the ordinary physical break from wrapped line geometry.
2. Materialize a compact line-record window containing offset, `PaginateAs`
   type, paragraph-start flag, and paragraph identity.
3. Apply the five recovered correction families in their observed order.
4. Apply Final Draft's sentence backtracking only when the document option is
   enabled.
5. Add continuation markers after the corrected semantic break, preserving the
   existing empirical `(MORE)` accounting until runtime probes settle it.

The customizable mode can then expose the same stages as knobs: sentence
correction, recognized terminators and abbreviations, scene-heading keep,
action orphan/widow policy, dialogue group policy, transition pullback, and
continuation-marker accounting.

## High-value follow-up probes

The static results make the next dynamic probe set much narrower:

1. Toggle `BreakDialogueAndActionAtSentences` on identical long Action,
   Parenthetical, and Dialogue paragraphs.
2. Put `;` at the candidate boundary and compare against `,` and `.`.
3. Put each of `Dr.`, `Mr.`, `Ms.`, `St.`, `Mrs.`, `Ave.`, and `Blvd.` at the
   candidate boundary, including mixed case.
4. Use a custom element whose visual settings match Action but whose
   `PaginateAs` is Dialogue, then reverse the two.
5. Sweep paragraph starts across cache positions corresponding to two through
   six lines around the page boundary for each of the five special handlers.
6. Repeat dialogue probes with `(MORE)` disabled and with a two-line marker to
   isolate marker geometry from the semantic correction pass.
7. Probe dual dialogue independently; its paragraph class and rendering path
   are distinct and have not yet been reduced to clean rules.

## Reproducing the static analysis

The repository contains a dependency-free Mach-O RTTI/vtable locator and small
Ghidra headless scripts:

- `tools/fd11_rtti_probe.py`
- `tools/ghidra/DecompileAddresses.java`
- `tools/ghidra/ReferencesToAddresses.java`
- `tools/ghidra/SummarizeFunctions.java`

Example RTTI lookup:

```sh
python3 tools/fd11_rtti_probe.py \
  '/path/to/Final Draft 11.app/Contents/MacOS/Final Draft 11' \
  --exact FDMultiPageText FDSPText FDMoreContinued
```

For Ghidra, import a private analysis copy of the executable into a headless
project, run normal auto-analysis once, and then use the scripts with
`-postScript`. Do not commit the application, executable, Ghidra project, or
decompiler output.

### Address notebook for build 96

These unslid virtual addresses are useful anchors for continuing analysis of
the exact executable identified above. They are not stable across builds.

| Address | Assigned description |
| --- | --- |
| `0x10002755e` | Element-name to pagination-type conversion |
| `0x10008fda6` | Read sentence-correction setting |
| `0x10008fdb2` | Write sentence-correction setting |
| `0x1000c19c6` | Pagination correction entry point |
| `0x1000c1c12` | Iterate proposed breaks with rolling line cache |
| `0x1000c2029` | Internal sentence-boundary correction |
| `0x1000c23d2` | Terminal-punctuation and abbreviation predicate |
| `0x1000c351d` | Ordered screenplay-rule dispatcher |
| `0x1000c35bf` | Scene Heading handler |
| `0x1000c36f6` | Action handler |
| `0x1000c3952` | Character handler |
| `0x1000c3aae` | Dialogue/Parenthetical handler |
| `0x1000c3d2f` | Transition handler |
| `0x10029ef30` | Read screenplay page-layout XML fields |
| `0x10029f1d8` | Write screenplay page-layout XML fields |

## Confidence and open areas

| Finding | Confidence |
| --- | --- |
| Serialized sentence option and default | Confirmed |
| Action/Parenthetical/Dialogue type gate | Confirmed |
| `. ! ; ?` terminal set | Confirmed |
| Seven abbreviation exceptions | Confirmed |
| Fixed order of five screenplay handlers | Confirmed |
| Local-window rather than global-scoring architecture | High |
| Exact user-facing orphan/widow counts for every branch | Partial |
| `(MORE)` geometry/accounting | Unresolved statically |
| Dual-dialogue page-break rules | Unresolved |
| Locked-page and revision-page interactions | Unresolved |
