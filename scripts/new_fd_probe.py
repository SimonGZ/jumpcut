#!/usr/bin/env python3
import argparse
import json
import shutil
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Create a new Final Draft probe fixture from a Fountain file."
    )
    parser.add_argument("probe_id", help="folder name / stable probe id")
    parser.add_argument("source_fountain", help="path to the source Fountain file to copy")
    parser.add_argument(
        "--kind",
        choices=["dialogue", "flow"],
        default="dialogue",
        help="target block kind for the probe JSON template",
    )
    parser.add_argument(
        "--speaker",
        default=None,
        help="optional speaker to prefill for dialogue probes",
    )
    parser.add_argument(
        "--description",
        default=None,
        help="optional human-readable description",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    source_path = Path(args.source_fountain).resolve()
    if not source_path.exists():
        raise SystemExit(f"source fountain does not exist: {source_path}")

    probe_dir = repo_root / "tests" / "fixtures" / "fd-probes" / args.probe_id
    probe_dir.mkdir(parents=True, exist_ok=False)

    shutil.copyfile(source_path, probe_dir / "source.fountain")

    expected = {
        "probe_id": args.probe_id,
        "description": args.description
        or f"Draft probe for {args.probe_id.replace('-', ' ')}.",
        "status": "draft",
        "lines_per_page": 54,
        "target": {
            "kind": args.kind,
            "contains_text": "REPLACE_ME_WITH_UNIQUE_SOURCE_SNIPPET",
        },
        "expected": {
            "kind": "split",
            "top_page": 1,
            "bottom_page": 2,
            "top_fragment_ends_with": "REPLACE_ME_WITH_TOP_FRAGMENT_END",
            "bottom_fragment_starts_with": "REPLACE_ME_WITH_BOTTOM_FRAGMENT_START",
        },
        "final_draft_notes": [
            "Replace the placeholder text after checking the probe in Final Draft.",
            "Record what Final Draft did, not the guessed rule behind it.",
        ],
    }

    if args.speaker is not None:
        expected["target"]["speaker"] = args.speaker

    with (probe_dir / "expected.json").open("w", encoding="utf-8") as handle:
        json.dump(expected, handle, indent=2)
        handle.write("\n")

    print(probe_dir)


if __name__ == "__main__":
    main()
