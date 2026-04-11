# Final Draft Probe Fixtures

These fixtures are small, hand-authored Fountain documents used to probe
specific Final Draft pagination behaviors.

Each probe lives in its own folder:

- `source.fountain`: the minimal screenplay probe
- `expected.json`: the observed Final Draft result

The intent is to make it easy for a human to:

1. convert a tiny Fountain file
2. open it in Final Draft
3 inspect where Final Draft splits or pushes a block
4. write down the result in `expected.json`
5. turn that into a durable regression test

Use plain observations, not inferred rules. Good fields are:

- page numbers
- whether the block split or was pushed whole
- the exact text the top fragment ends with
- the exact text the bottom fragment starts with
- short notes about `(MORE)` / `(CONT'D)` / visual oddities

## JSON format

`expected.json` currently supports these fields:

- `probe_id`: short stable id
- `description`: human-readable summary
- `status`: `draft` or `active`
- `lines_per_page`: usually `54`
- `target.kind`: `dialogue` or `flow`
- `target.contains_text`: a unique snippet used to find the target block
- `target.speaker`: optional, useful for dialogue disambiguation

`expected.kind = "split"`:

- `top_page`
- `bottom_page`
- `top_fragment_ends_with`
- `bottom_fragment_starts_with`

`expected.kind = "push-whole"`:

- `absent_from_page`
- `whole_on_page`
- `starts_with`

## Workflow

Start new probes as `draft`.
Once Final Draft behavior has been checked and entered, switch to `active`.

You can scaffold a new probe from an existing Fountain file with:

```bash
just fd-probe-new my-probe-name /path/to/source.fountain
```

That creates:

- `tests/fixtures/fd-probes/my-probe-name/source.fountain`
- `tests/fixtures/fd-probes/my-probe-name/expected.json`

Then edit `expected.json` with the observed Final Draft behavior.

`active` probes are enforced by:

- `tests/pagination_fd_probe_test.rs`

Draft probes still parse, but they do not fail the suite.
