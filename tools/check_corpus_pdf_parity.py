#!/usr/bin/env python3

from __future__ import annotations

import json
import math
import statistics
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGET_DIR = ROOT / "target" / "pdf-parity"
REPORTS_DIR = ROOT / "target" / "pdf-placement-diagnostics"


@dataclass(frozen=True)
class PdfParityCase:
    name: str
    fountain: Path
    reference_pdf: Path
    report_name: str
    max_mean_x: float
    max_mean_y: float
    max_abs_x: float
    max_abs_y: float


@dataclass(frozen=True)
class WordBox:
    page_number: int
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


CASES = [
    PdfParityCase(
        name="big-fish",
        fountain=ROOT / "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/big-fish/extracted/reference.pdf",
        report_name="verify-big-fish-pdf-parity",
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=3.50,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="little-women",
        fountain=ROOT / "tests/fixtures/corpus/public/little-women/source/source.fountain",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/little-women/extracted/reference.pdf",
        report_name="verify-little-women-pdf-parity",
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=2.25,
        max_abs_y=0.10,
    ),
]


def run(cmd: list[str], cwd: Path) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def output_pdf_for(case: PdfParityCase) -> Path:
    return TARGET_DIR / f"{case.name}.pdf"


def extract_word_boxes(pdf_path: Path) -> list[WordBox]:
    with tempfile.NamedTemporaryFile(suffix=".xhtml", delete=False) as handle:
        tmp_path = Path(handle.name)
    try:
        run(["pdftotext", "-bbox-layout", str(pdf_path), str(tmp_path)], cwd=ROOT)
        tree = ET.parse(tmp_path)
    finally:
        tmp_path.unlink(missing_ok=True)

    boxes: list[WordBox] = []
    for page_number, page in enumerate(tree.findall(".//{*}page"), start=1):
        for word in page.findall(".//{*}word"):
            text = "".join(word.itertext())
            if not text:
                continue
            boxes.append(
                WordBox(
                    page_number=page_number,
                    text=text,
                    x_min=float(word.attrib["xMin"]),
                    y_min=float(word.attrib["yMin"]),
                    x_max=float(word.attrib["xMax"]),
                    y_max=float(word.attrib["yMax"]),
                )
            )
    return boxes


def compare_word_boxes(actual: list[WordBox], reference: list[WordBox]) -> tuple[dict, list[str]]:
    text_mismatch = len(actual) != len(reference) or any(
        a.page_number != b.page_number or a.text != b.text
        for a, b in zip(actual, reference)
    )

    if text_mismatch:
        report = {
            "text_mismatch": True,
            "summary": {
                "word_count": min(len(actual), len(reference)),
                "mean_word_x_delta": math.nan,
                "mean_word_y_delta": math.nan,
                "mean_word_width_delta": math.nan,
                "max_abs_word_x_delta": math.nan,
                "max_abs_word_y_delta": math.nan,
            },
            "word_deltas": [],
        }
        return report, ["text mismatch detected"]

    deltas = []
    x_deltas = []
    y_deltas = []
    width_deltas = []
    height_deltas = []

    for actual_box, reference_box in zip(actual, reference):
        x_delta = actual_box.x_min - reference_box.x_min
        y_delta = actual_box.y_min - reference_box.y_min
        width_delta = actual_box.width - reference_box.width
        height_delta = actual_box.height - reference_box.height

        x_deltas.append(x_delta)
        y_deltas.append(y_delta)
        width_deltas.append(width_delta)
        height_deltas.append(height_delta)
        deltas.append(
            {
                "page_number": actual_box.page_number,
                "text": actual_box.text,
                "x_delta": x_delta,
                "y_delta": y_delta,
                "width_delta": width_delta,
                "height_delta": height_delta,
                "actual_x_min": actual_box.x_min,
                "reference_x_min": reference_box.x_min,
                "actual_y_min": actual_box.y_min,
                "reference_y_min": reference_box.y_min,
            }
        )

    report = {
        "text_mismatch": False,
        "summary": {
            "word_count": len(deltas),
            "mean_word_x_delta": statistics.fmean(x_deltas),
            "mean_word_y_delta": statistics.fmean(y_deltas),
            "mean_word_width_delta": statistics.fmean(width_deltas),
            "max_abs_word_x_delta": max(abs(delta) for delta in x_deltas),
            "max_abs_word_y_delta": max(abs(delta) for delta in y_deltas),
        },
        "word_deltas": deltas,
    }
    return report, []


def write_report(case: PdfParityCase, report: dict) -> Path:
    out_dir = REPORTS_DIR / case.report_name
    out_dir.mkdir(parents=True, exist_ok=True)
    report_path = out_dir / "report.json"
    review_path = out_dir / "REVIEW.md"

    report_path.write_text(json.dumps(report, indent=2) + "\n")

    summary = report["summary"]
    review_path.write_text(
        "\n".join(
            [
                f"# {case.name} PDF parity",
                "",
                f"- text mismatch: {report['text_mismatch']}",
                f"- word count: {summary['word_count']}",
                f"- mean word x delta: {summary['mean_word_x_delta']:.4f}pt"
                if not math.isnan(summary["mean_word_x_delta"])
                else "- mean word x delta: n/a",
                f"- mean word y delta: {summary['mean_word_y_delta']:.4f}pt"
                if not math.isnan(summary["mean_word_y_delta"])
                else "- mean word y delta: n/a",
                f"- max abs word x delta: {summary['max_abs_word_x_delta']:.4f}pt"
                if not math.isnan(summary["max_abs_word_x_delta"])
                else "- max abs word x delta: n/a",
                f"- max abs word y delta: {summary['max_abs_word_y_delta']:.4f}pt"
                if not math.isnan(summary["max_abs_word_y_delta"])
                else "- max abs word y delta: n/a",
            ]
        )
        + "\n"
    )

    return report_path


def check_case(case: PdfParityCase) -> list[str]:
    output_pdf = output_pdf_for(case)
    output_pdf.parent.mkdir(parents=True, exist_ok=True)

    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
            "--quiet",
            "--bin",
            "jumpcut",
            "--",
            "-f",
            "pdf",
            str(case.fountain),
            str(output_pdf),
        ],
        cwd=ROOT,
    )

    report, failures = compare_word_boxes(
        extract_word_boxes(output_pdf),
        extract_word_boxes(case.reference_pdf),
    )
    report_path = write_report(case, report)
    summary = report["summary"]

    if not report["text_mismatch"]:
        if abs(summary["mean_word_x_delta"]) > case.max_mean_x:
            failures.append(
                f"mean word x delta {summary['mean_word_x_delta']:.4f}pt exceeds {case.max_mean_x:.4f}pt"
            )
        if abs(summary["mean_word_y_delta"]) > case.max_mean_y:
            failures.append(
                f"mean word y delta {summary['mean_word_y_delta']:.4f}pt exceeds {case.max_mean_y:.4f}pt"
            )
        if summary["max_abs_word_x_delta"] > case.max_abs_x:
            failures.append(
                f"max abs word x delta {summary['max_abs_word_x_delta']:.4f}pt exceeds {case.max_abs_x:.4f}pt"
            )
        if summary["max_abs_word_y_delta"] > case.max_abs_y:
            failures.append(
                f"max abs word y delta {summary['max_abs_word_y_delta']:.4f}pt exceeds {case.max_abs_y:.4f}pt"
            )

    print(f"case: {case.name}")
    print(f"report: {report_path}")
    print(f"text mismatch: {report['text_mismatch']}")
    print(f"mean word x delta: {summary['mean_word_x_delta']:.4f}pt")
    print(f"mean word y delta: {summary['mean_word_y_delta']:.4f}pt")
    print(f"max abs word x delta: {summary['max_abs_word_x_delta']:.4f}pt")
    print(f"max abs word y delta: {summary['max_abs_word_y_delta']:.4f}pt")
    if failures:
        print("status: FAIL")
        for failure in failures:
            print(f"- {failure}")
    else:
        print("status: PASS")
    print()

    return failures


def main() -> int:
    any_failures = False
    for case in CASES:
        failures = check_case(case)
        if failures:
            any_failures = True
    return 1 if any_failures else 0


if __name__ == "__main__":
    sys.exit(main())
