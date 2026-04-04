#!/usr/bin/env python3
"""Measure matching PDF text boxes from the command line.

Examples:
  python3 tools/pdf_measure.py sample.pdf --text "written by"
  python3 tools/pdf_measure.py a.pdf b.pdf --text "2."
  python3 tools/pdf_measure.py ref.pdf --text "Sample Script" --contains --json
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import xml.etree.ElementTree as ET
from dataclasses import asdict, dataclass
from pathlib import Path


POINTS_PER_INCH = 72.0


@dataclass
class MatchRecord:
    pdf: str
    page: int
    text: str
    x_min_pt: float
    y_min_pt: float
    x_max_pt: float
    y_max_pt: float
    width_pt: float
    height_pt: float
    x_min_in: float
    y_min_in: float
    width_in: float
    height_in: float
    from_left_margin_pt: float
    from_top_margin_pt: float
    from_left_margin_in: float
    from_top_margin_in: float
    x_min_nearest_sixteenth_in: str
    x_min_sixteenth_delta_in: float
    y_min_nearest_sixteenth_in: str
    y_min_sixteenth_delta_in: float
    width_nearest_sixteenth_in: str
    width_sixteenth_delta_in: float
    height_nearest_sixteenth_in: str
    height_sixteenth_delta_in: float
    from_left_margin_nearest_sixteenth_in: str
    from_left_margin_sixteenth_delta_in: float
    from_top_margin_nearest_sixteenth_in: str
    from_top_margin_sixteenth_delta_in: float


def parse_args() -> argparse.Namespace:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(
        description="Measure matching text boxes in one or more PDFs."
    )
    parser.add_argument("pdfs", nargs="+", help="PDFs to inspect.")
    parser.add_argument("--text", required=True, help="Target text to search for.")
    parser.add_argument(
        "--contains",
        action="store_true",
        help="Match lines containing the target text instead of requiring an exact match.",
    )
    parser.add_argument(
        "--case-insensitive",
        action="store_true",
        help="Match text case-insensitively.",
    )
    parser.add_argument(
        "--left-margin-in",
        type=float,
        default=1.5,
        help="Reference left text margin in inches. Default: %(default)s",
    )
    parser.add_argument(
        "--top-margin-in",
        type=float,
        default=1.0,
        help="Reference top text margin in inches. Default: %(default)s",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit machine-readable JSON instead of human-readable text.",
    )
    parser.add_argument(
        "--repo",
        default=str(repo_root),
        help=argparse.SUPPRESS,
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    all_matches: list[MatchRecord] = []

    for pdf in args.pdfs:
        pdf_path = Path(pdf).resolve()
        if not pdf_path.is_file():
            raise SystemExit(f"PDF not found: {pdf_path}")
        all_matches.extend(
            find_matches(
                pdf_path,
                target_text=args.text,
                contains=args.contains,
                case_insensitive=args.case_insensitive,
                left_margin_in=args.left_margin_in,
                top_margin_in=args.top_margin_in,
            )
        )

    if args.json:
        payload = {
            "target_text": args.text,
            "contains": args.contains,
            "case_insensitive": args.case_insensitive,
            "left_margin_in": args.left_margin_in,
            "top_margin_in": args.top_margin_in,
            "matches": [asdict(match) for match in all_matches],
        }
        print(json.dumps(payload, indent=2))
    else:
        if not all_matches:
            print("No matches found.", file=sys.stderr)
            return 1
        for match in all_matches:
            print(f"{match.pdf} :: page {match.page}")
            print(f"  text: {match.text}")
            print(
                f"  bbox pt: left={match.x_min_pt:.3f} top={match.y_min_pt:.3f} "
                f"right={match.x_max_pt:.3f} bottom={match.y_max_pt:.3f} "
                f"width={match.width_pt:.3f} height={match.height_pt:.3f}"
            )
            print(
                f"  bbox in: left={match.x_min_in:.4f} top={match.y_min_in:.4f} "
                f"width={match.width_in:.4f} height={match.height_in:.4f}"
            )
            print(
                f"  nearest 1/16 in: left={match.x_min_nearest_sixteenth_in} "
                f"(delta {match.x_min_sixteenth_delta_in:+.4f}in), "
                f"top={match.y_min_nearest_sixteenth_in} "
                f"(delta {match.y_min_sixteenth_delta_in:+.4f}in), "
                f"width={match.width_nearest_sixteenth_in} "
                f"(delta {match.width_sixteenth_delta_in:+.4f}in), "
                f"height={match.height_nearest_sixteenth_in} "
                f"(delta {match.height_sixteenth_delta_in:+.4f}in)"
            )
            print(
                f"  from margins: left={match.from_left_margin_pt:.3f}pt "
                f"({match.from_left_margin_in:.4f}in), "
                f"top={match.from_top_margin_pt:.3f}pt "
                f"({match.from_top_margin_in:.4f}in)"
            )
            print(
                f"  margin nearest 1/16 in: left={match.from_left_margin_nearest_sixteenth_in} "
                f"(delta {match.from_left_margin_sixteenth_delta_in:+.4f}in), "
                f"top={match.from_top_margin_nearest_sixteenth_in} "
                f"(delta {match.from_top_margin_sixteenth_delta_in:+.4f}in)"
            )
            print()

    return 0


def find_matches(
    pdf_path: Path,
    *,
    target_text: str,
    contains: bool,
    case_insensitive: bool,
    left_margin_in: float,
    top_margin_in: float,
) -> list[MatchRecord]:
    xml_text = subprocess.run(
        ["pdftotext", "-bbox-layout", "-enc", "UTF-8", str(pdf_path), "-"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    root = ET.fromstring(xml_text)

    needle = target_text.casefold() if case_insensitive else target_text
    matches: list[MatchRecord] = []

    for page_index, page in enumerate(root.findall(".//{*}page"), start=1):
        for line in page.findall("./{*}flow/{*}block/{*}line"):
            words = line.findall("./{*}word")
            if not words:
                continue
            text = " ".join("".join(word.itertext()) for word in words)
            haystack = text.casefold() if case_insensitive else text
            matched = needle in haystack if contains else needle == haystack
            if not matched:
                continue

            x_min = float(words[0].attrib["xMin"])
            y_min = float(words[0].attrib["yMin"])
            x_max = float(words[-1].attrib["xMax"])
            y_max = max(float(word.attrib["yMax"]) for word in words)
            width = x_max - x_min
            height = y_max - y_min
            left_margin_pt = left_margin_in * POINTS_PER_INCH
            top_margin_pt = top_margin_in * POINTS_PER_INCH

            matches.append(
                MatchRecord(
                    pdf=str(pdf_path),
                    page=page_index,
                    text=text,
                    x_min_pt=x_min,
                    y_min_pt=y_min,
                    x_max_pt=x_max,
                    y_max_pt=y_max,
                    width_pt=width,
                    height_pt=height,
                    x_min_in=x_min / POINTS_PER_INCH,
                    y_min_in=y_min / POINTS_PER_INCH,
                    width_in=width / POINTS_PER_INCH,
                    height_in=height / POINTS_PER_INCH,
                    from_left_margin_pt=x_min - left_margin_pt,
                    from_top_margin_pt=y_min - top_margin_pt,
                    from_left_margin_in=(x_min - left_margin_pt) / POINTS_PER_INCH,
                    from_top_margin_in=(y_min - top_margin_pt) / POINTS_PER_INCH,
                    x_min_nearest_sixteenth_in=nearest_sixteenth_label(
                        x_min / POINTS_PER_INCH
                    ),
                    x_min_sixteenth_delta_in=sixteenth_delta(
                        x_min / POINTS_PER_INCH
                    ),
                    y_min_nearest_sixteenth_in=nearest_sixteenth_label(
                        y_min / POINTS_PER_INCH
                    ),
                    y_min_sixteenth_delta_in=sixteenth_delta(
                        y_min / POINTS_PER_INCH
                    ),
                    width_nearest_sixteenth_in=nearest_sixteenth_label(
                        width / POINTS_PER_INCH
                    ),
                    width_sixteenth_delta_in=sixteenth_delta(
                        width / POINTS_PER_INCH
                    ),
                    height_nearest_sixteenth_in=nearest_sixteenth_label(
                        height / POINTS_PER_INCH
                    ),
                    height_sixteenth_delta_in=sixteenth_delta(
                        height / POINTS_PER_INCH
                    ),
                    from_left_margin_nearest_sixteenth_in=nearest_sixteenth_label(
                        (x_min - left_margin_pt) / POINTS_PER_INCH
                    ),
                    from_left_margin_sixteenth_delta_in=sixteenth_delta(
                        (x_min - left_margin_pt) / POINTS_PER_INCH
                    ),
                    from_top_margin_nearest_sixteenth_in=nearest_sixteenth_label(
                        (y_min - top_margin_pt) / POINTS_PER_INCH
                    ),
                    from_top_margin_sixteenth_delta_in=sixteenth_delta(
                        (y_min - top_margin_pt) / POINTS_PER_INCH
                    ),
                )
            )

    return matches


def nearest_sixteenth_label(value_in: float) -> str:
    rounded = round(value_in * 16)
    return sixteenth_label(rounded)


def sixteenth_delta(value_in: float) -> float:
    rounded = round(value_in * 16) / 16
    return value_in - rounded


def sixteenth_label(sixteenths: int) -> str:
    sign = "-" if sixteenths < 0 else ""
    sixteenths = abs(sixteenths)
    whole = sixteenths // 16
    remainder = sixteenths % 16
    if remainder == 0:
        return f"{sign}{whole}\""
    reduced_num, reduced_den = reduce_fraction(remainder, 16)
    if whole == 0:
        return f'{sign}{reduced_num}/{reduced_den}"'
    return f'{sign}{whole} {reduced_num}/{reduced_den}"'


def reduce_fraction(numerator: int, denominator: int) -> tuple[int, int]:
    a, b = numerator, denominator
    while b:
        a, b = b, a % b
    return numerator // a, denominator // a


if __name__ == "__main__":
    raise SystemExit(main())
