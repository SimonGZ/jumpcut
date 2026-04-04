#!/usr/bin/env python3
"""Compare text placement between two PDFs using pdftotext -bbox-layout.

The tool emits:
- `report.json`: machine-readable structured comparison report
- `word-deltas.csv`: per-word placement deltas
- `REVIEW.md`: human-readable summary

It assumes the two PDFs contain the same text in the same order on the pages
being compared. When that assumption fails, the report records the mismatch
rather than trying to force a geometric comparison.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import statistics
import subprocess
import sys
import xml.etree.ElementTree as ET
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


@dataclass
class Word:
    page_number: int
    block_index: int
    line_index: int
    word_index: int
    text: str
    x_min: float
    y_min: float
    x_max: float
    y_max: float

    @property
    def width(self) -> float:
        return self.x_max - self.x_min

    @property
    def height(self) -> float:
        return self.y_max - self.y_min


@dataclass
class WordDelta:
    page_number: int
    line_index: int
    word_index: int
    text: str
    x_delta: float
    y_delta: float
    width_delta: float
    height_delta: float
    actual_x_min: float
    reference_x_min: float
    actual_y_min: float
    reference_y_min: float


def parse_args() -> argparse.Namespace:
    repo_root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(
        description="Compare text placement between two PDFs."
    )
    parser.add_argument("actual_pdf", help="Path to the generated PDF under test.")
    parser.add_argument("reference_pdf", help="Path to the reference PDF.")
    parser.add_argument(
        "--name",
        default=None,
        help="Stable diagnostic name. Defaults to '<actual>-vs-<reference>'.",
    )
    parser.add_argument(
        "--skip-leading-pages",
        type=int,
        default=0,
        help="Skip this many leading pages in both PDFs before comparing.",
    )
    parser.add_argument(
        "--out-dir",
        default=None,
        help="Output directory. Defaults to target/pdf-placement-diagnostics/<name>.",
    )
    parser.add_argument(
        "--max-outliers",
        type=int,
        default=12,
        help="How many largest absolute deltas to include in REVIEW.md.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    actual_pdf = Path(args.actual_pdf).resolve()
    reference_pdf = Path(args.reference_pdf).resolve()

    if not actual_pdf.is_file():
        raise SystemExit(f"actual PDF not found: {actual_pdf}")
    if not reference_pdf.is_file():
        raise SystemExit(f"reference PDF not found: {reference_pdf}")

    name = args.name or f"{actual_pdf.stem}-vs-{reference_pdf.stem}"
    out_dir = (
        Path(args.out_dir).resolve()
        if args.out_dir
        else (Path(__file__).resolve().parents[1] / "target" / "pdf-placement-diagnostics" / name)
    )
    out_dir.mkdir(parents=True, exist_ok=True)

    actual_pages = extract_pdf_words(actual_pdf)
    reference_pages = extract_pdf_words(reference_pdf)
    report = compare_pages(
        actual_pages,
        reference_pages,
        skip_leading_pages=args.skip_leading_pages,
        actual_pdf=actual_pdf,
        reference_pdf=reference_pdf,
    )

    report_path = out_dir / "report.json"
    review_path = out_dir / "REVIEW.md"
    csv_path = out_dir / "word-deltas.csv"

    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    write_word_deltas_csv(csv_path, report["word_deltas"])
    review_path.write_text(
        render_review(report, max_outliers=args.max_outliers),
        encoding="utf-8",
    )

    print(f"wrote {report_path}")
    print(f"wrote {csv_path}")
    print(f"wrote {review_path}")
    return 0


def extract_pdf_words(pdf_path: Path) -> list[list[Word]]:
    command = ["pdftotext", "-bbox-layout", "-enc", "UTF-8", str(pdf_path), "-"]
    result = subprocess.run(command, capture_output=True, text=True, check=True)
    xml_text = result.stdout
    root = ET.fromstring(xml_text)

    pages: list[list[Word]] = []
    for page_index, page in enumerate(root.findall(".//{*}page"), start=1):
        words: list[Word] = []
        line_counter = 0
        for block_index, block in enumerate(page.findall("./{*}flow/{*}block")):
            for line in block.findall("./{*}line"):
                for word_index, word in enumerate(line.findall("./{*}word")):
                    text = "".join(word.itertext())
                    if not text:
                        continue
                    words.append(
                        Word(
                            page_number=page_index,
                            block_index=block_index,
                            line_index=line_counter,
                            word_index=word_index,
                            text=text,
                            x_min=float(word.attrib["xMin"]),
                            y_min=float(word.attrib["yMin"]),
                            x_max=float(word.attrib["xMax"]),
                            y_max=float(word.attrib["yMax"]),
                        )
                    )
                line_counter += 1
        pages.append(words)
    return pages


def compare_pages(
    actual_pages: list[list[Word]],
    reference_pages: list[list[Word]],
    *,
    skip_leading_pages: int,
    actual_pdf: Path,
    reference_pdf: Path,
) -> dict:
    actual_pages = actual_pages[skip_leading_pages:]
    reference_pages = reference_pages[skip_leading_pages:]

    compared_page_count = min(len(actual_pages), len(reference_pages))
    page_reports: list[dict] = []
    all_word_deltas: list[dict] = []
    all_line_start_deltas: list[float] = []
    all_line_vertical_deltas: list[float] = []
    exact_line_start_matches = 0
    total_line_count = 0
    text_mismatch = False

    for page_offset in range(compared_page_count):
        actual_words = actual_pages[page_offset]
        reference_words = reference_pages[page_offset]
        actual_page_number = page_offset + skip_leading_pages + 1
        reference_page_number = page_offset + skip_leading_pages + 1

        page_report = compare_single_page(
            actual_words,
            reference_words,
            actual_page_number=actual_page_number,
            reference_page_number=reference_page_number,
        )
        page_reports.append(page_report)
        all_word_deltas.extend(page_report["word_deltas"])
        all_line_start_deltas.extend(page_report["line_start_x_deltas"])
        all_line_vertical_deltas.extend(page_report["line_start_y_deltas"])
        exact_line_start_matches += page_report["exact_line_start_matches"]
        total_line_count += page_report["compared_line_count"]
        text_mismatch = text_mismatch or page_report["text_mismatch"]

    summary = summarize_report(
        all_word_deltas=all_word_deltas,
        all_line_start_deltas=all_line_start_deltas,
        all_line_vertical_deltas=all_line_vertical_deltas,
        exact_line_start_matches=exact_line_start_matches,
        total_line_count=total_line_count,
    )

    return {
        "tool": {
            "name": "pdf_text_placement_diagnostics",
            "version": 1,
            "method": "pdftotext -bbox-layout",
        },
        "inputs": {
            "actual_pdf": str(actual_pdf),
            "reference_pdf": str(reference_pdf),
            "skip_leading_pages": skip_leading_pages,
        },
        "page_counts": {
            "actual_total_pages": len(actual_pages) + skip_leading_pages,
            "reference_total_pages": len(reference_pages) + skip_leading_pages,
            "compared_pages": compared_page_count,
        },
        "text_mismatch": text_mismatch,
        "summary": summary,
        "pages": page_reports,
        "word_deltas": all_word_deltas,
    }


def compare_single_page(
    actual_words: list[Word],
    reference_words: list[Word],
    *,
    actual_page_number: int,
    reference_page_number: int,
) -> dict:
    actual_lines = group_words_by_line(actual_words)
    reference_lines = group_words_by_line(reference_words)

    text_mismatch = False
    mismatches: list[dict] = []
    word_deltas: list[dict] = []
    line_start_x_deltas: list[float] = []
    line_start_y_deltas: list[float] = []
    exact_line_start_matches = 0

    compared_line_count = min(len(actual_lines), len(reference_lines))
    for index in range(compared_line_count):
        actual_line = actual_lines[index]
        reference_line = reference_lines[index]
        actual_text = " ".join(word.text for word in actual_line)
        reference_text = " ".join(word.text for word in reference_line)
        if actual_text != reference_text:
            text_mismatch = True
            mismatches.append(
                {
                    "kind": "line_text_mismatch",
                    "line_index": index,
                    "actual_text": actual_text,
                    "reference_text": reference_text,
                }
            )
            continue

        x_delta = actual_line[0].x_min - reference_line[0].x_min
        y_delta = actual_line[0].y_min - reference_line[0].y_min
        line_start_x_deltas.append(x_delta)
        line_start_y_deltas.append(y_delta)
        if abs(x_delta) < 1e-6:
            exact_line_start_matches += 1

        for word_index, (actual_word, reference_word) in enumerate(
            zip(actual_line, reference_line, strict=False)
        ):
            if actual_word.text != reference_word.text:
                text_mismatch = True
                mismatches.append(
                    {
                        "kind": "word_text_mismatch",
                        "line_index": index,
                        "word_index": word_index,
                        "actual_text": actual_word.text,
                        "reference_text": reference_word.text,
                    }
                )
                continue

            word_delta = WordDelta(
                page_number=actual_page_number,
                line_index=index,
                word_index=word_index,
                text=actual_word.text,
                x_delta=actual_word.x_min - reference_word.x_min,
                y_delta=actual_word.y_min - reference_word.y_min,
                width_delta=actual_word.width - reference_word.width,
                height_delta=actual_word.height - reference_word.height,
                actual_x_min=actual_word.x_min,
                reference_x_min=reference_word.x_min,
                actual_y_min=actual_word.y_min,
                reference_y_min=reference_word.y_min,
            )
            word_deltas.append(asdict(word_delta))

    if len(actual_lines) != len(reference_lines):
        text_mismatch = True
        mismatches.append(
            {
                "kind": "line_count_mismatch",
                "actual_line_count": len(actual_lines),
                "reference_line_count": len(reference_lines),
            }
        )

    return {
        "actual_page_number": actual_page_number,
        "reference_page_number": reference_page_number,
        "actual_word_count": len(actual_words),
        "reference_word_count": len(reference_words),
        "actual_line_count": len(actual_lines),
        "reference_line_count": len(reference_lines),
        "compared_line_count": compared_line_count,
        "exact_line_start_matches": exact_line_start_matches,
        "text_mismatch": text_mismatch,
        "line_start_x_deltas": line_start_x_deltas,
        "line_start_y_deltas": line_start_y_deltas,
        "page_summary": {
            "mean_word_x_delta": mean_or_none(item["x_delta"] for item in word_deltas),
            "mean_word_y_delta": mean_or_none(item["y_delta"] for item in word_deltas),
            "mean_word_width_delta": mean_or_none(
                item["width_delta"] for item in word_deltas
            ),
            "mean_line_start_x_delta": mean_or_none(line_start_x_deltas),
            "mean_line_start_y_delta": mean_or_none(line_start_y_deltas),
        },
        "mismatches": mismatches,
        "word_deltas": word_deltas,
    }


def group_words_by_line(words: list[Word]) -> list[list[Word]]:
    grouped: dict[int, list[Word]] = {}
    for word in words:
        grouped.setdefault(word.line_index, []).append(word)
    return [grouped[index] for index in sorted(grouped)]


def summarize_report(
    *,
    all_word_deltas: list[dict],
    all_line_start_deltas: list[float],
    all_line_vertical_deltas: list[float],
    exact_line_start_matches: int,
    total_line_count: int,
) -> dict:
    return {
        "word_count": len(all_word_deltas),
        "mean_word_x_delta": mean_or_none(item["x_delta"] for item in all_word_deltas),
        "mean_word_y_delta": mean_or_none(item["y_delta"] for item in all_word_deltas),
        "mean_word_width_delta": mean_or_none(
            item["width_delta"] for item in all_word_deltas
        ),
        "max_abs_word_x_delta": max_abs_or_none(
            item["x_delta"] for item in all_word_deltas
        ),
        "max_abs_word_y_delta": max_abs_or_none(
            item["y_delta"] for item in all_word_deltas
        ),
        "mean_line_start_x_delta": mean_or_none(all_line_start_deltas),
        "mean_line_start_y_delta": mean_or_none(all_line_vertical_deltas),
        "exact_line_start_matches": exact_line_start_matches,
        "total_line_count": total_line_count,
    }


def write_word_deltas_csv(path: Path, word_deltas: list[dict]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "page_number",
                "line_index",
                "word_index",
                "text",
                "x_delta",
                "y_delta",
                "width_delta",
                "height_delta",
                "actual_x_min",
                "reference_x_min",
                "actual_y_min",
                "reference_y_min",
            ],
        )
        writer.writeheader()
        writer.writerows(word_deltas)


def render_review(report: dict, *, max_outliers: int) -> str:
    summary = report["summary"]
    lines = [
        "# PDF Text Placement Review",
        "",
        f"- Actual: `{report['inputs']['actual_pdf']}`",
        f"- Reference: `{report['inputs']['reference_pdf']}`",
        f"- Compared pages: {report['page_counts']['compared_pages']}",
        f"- Skipped leading pages: {report['inputs']['skip_leading_pages']}",
        f"- Text mismatch detected: {'yes' if report['text_mismatch'] else 'no'}",
        "",
        "## Summary",
        "",
        f"- Compared words: {summary['word_count']}",
        f"- Exact line-start matches: {summary['exact_line_start_matches']} / {summary['total_line_count']}",
        f"- Mean word x delta: {format_float(summary['mean_word_x_delta'])} pt",
        f"- Mean word y delta: {format_float(summary['mean_word_y_delta'])} pt",
        f"- Mean word width delta: {format_float(summary['mean_word_width_delta'])} pt",
        f"- Max abs word x delta: {format_float(summary['max_abs_word_x_delta'])} pt",
        f"- Max abs word y delta: {format_float(summary['max_abs_word_y_delta'])} pt",
        "",
        "## Per-Page Summary",
        "",
    ]

    for page in report["pages"]:
        page_summary = page["page_summary"]
        lines.extend(
            [
                f"### Page {page['actual_page_number']}",
                "",
                f"- Actual lines / reference lines: {page['actual_line_count']} / {page['reference_line_count']}",
                f"- Actual words / reference words: {page['actual_word_count']} / {page['reference_word_count']}",
                f"- Exact line-start matches: {page['exact_line_start_matches']} / {page['compared_line_count']}",
                f"- Mean line-start x delta: {format_float(page_summary['mean_line_start_x_delta'])} pt",
                f"- Mean line-start y delta: {format_float(page_summary['mean_line_start_y_delta'])} pt",
                f"- Mean word width delta: {format_float(page_summary['mean_word_width_delta'])} pt",
            ]
        )
        if page["mismatches"]:
            lines.append("- Mismatches:")
            for mismatch in page["mismatches"]:
                lines.append(f"  - {json.dumps(mismatch, ensure_ascii=False)}")
        lines.append("")

    lines.extend(["## Largest Word Outliers", ""])
    outliers = sorted(
        report["word_deltas"],
        key=lambda item: (
            abs(item["x_delta"]) + abs(item["y_delta"]) + abs(item["width_delta"])
        ),
        reverse=True,
    )[:max_outliers]
    if not outliers:
        lines.append("- None")
    else:
        for item in outliers:
            lines.append(
                "- "
                + (
                    f"page {item['page_number']} line {item['line_index']} word {item['word_index']} "
                    f"`{item['text']}`: "
                    f"x {format_float(item['x_delta'])} pt, "
                    f"y {format_float(item['y_delta'])} pt, "
                    f"width {format_float(item['width_delta'])} pt"
                )
            )
    lines.append("")

    return "\n".join(lines) + "\n"


def mean_or_none(values: Iterable[float]) -> float | None:
    values = list(values)
    if not values:
        return None
    return statistics.mean(values)


def max_abs_or_none(values: Iterable[float]) -> float | None:
    values = [abs(value) for value in values]
    if not values:
        return None
    return max(values)


def format_float(value: float | None) -> str:
    if value is None or math.isnan(value):
        return "n/a"
    return f"{value:.4f}"


if __name__ == "__main__":
    sys.exit(main())
