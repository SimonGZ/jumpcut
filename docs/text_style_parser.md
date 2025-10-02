# Text Style Parser Overview

## High-Level Flow
- `parse_and_convert_markup` walks every `Element` produced by the Fountain parser.
- `convert_plain_to_styled` replaces any `ElementText::Plain` value with the stack-based styling model when markup tokens are present.
- `insert_markers` performs a single forward scan over the raw text, identifying matching style delimiters (`*` and `_`) and emitting internal sentinel characters that represent the start/end of each style span.
- `create_styled_from_string` consumes the sentinel-enriched string, builds `TextRun` segments, and attaches the active style set to each run.

## Delimiter Scanning
- The scanner iterates through the string using `char_indices`, maintaining a stack (`delimiter_stack`) of potential opening markers.
- Each delimiter stores its byte position, run length (1/2/3), marker character, and whether it can open or close based on surrounding characters (no whitespace, no trailing backslash).
- When a closing delimiter is found, we walk the stack backwards to find the most recent compatible opener whose interior is valid (non-empty, no trailing whitespace/newline).
- Matching pairs are recorded as `MarkerEvent`s. Each event stores the byte position to replace, the number of bytes consumed, and the sentinel to insert.
- Matched delimiters are removed via `swap_remove`, keeping the stack compact without reallocation.
- After the pass, events are sorted by position and applied to rebuild the output string with sentinels while respecting escaped markers (e.g., `\*`).

## Sentinels and Text Runs
- Sentinels (`⏋`, `⎿`, `⏉`, `⏊`) map directly to style sets (bold+italic, bold, italic, underline).
- `create_styled_from_string` walks the sentinel-rich text, toggling styles when it sees a sentinel and pushing `TextRun` entries whenever a run ends.
- Style sets are stored as `HashSet<String>` today for compatibility with existing output formats; they’re derived from the sentinel that triggered the transition.

## Comparison with the Legacy Regex-Based Parser
- **Old approach:** four regular expressions (`***`/`**`/`*`/`_`) replaced matched regions with sentinels via global replacement. Ordering mattered to avoid overlaps, and the expressions relied on Unicode-aware regex features (adding to WASM size).
- **New approach:** deterministic scanner with an explicit delimiter stack, influenced by Markdown parsers (pulldown-cmark). No regex dependencies, so the WASM bundle shrank dramatically (~1.5 MB → 0.67 MB).
- **Reliability:** The stack-based logic makes precedence explicit and handles nesting/overlap through matching rules rather than regex engine side effects.
- **Performance:** Regex removal eliminated DFA setup costs; after recent optimisations (event sorting, swap removal) the stack parser is within ~12 % of the old performance while remaining portable.

## Key Extension Points
- `StyleKind` and sentinel constants make it easy to add new inline styles (e.g., strike-through) by defining a marker/run length mapping and updating `create_styled_from_string`.
- `is_valid_content_slice` centralises the Fountain rules that disallow whitespace-only spans. Adjust here if the grammar changes.
- The delimiter scan currently allocates a `Vec<MarkerEvent>` each call; a future optimisation could reuse a buffer or switch to a small fixed array for tiny strings.
