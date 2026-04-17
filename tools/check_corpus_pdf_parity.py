#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import math
import shutil
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
DEFAULT_MAX_MEAN_X = 0.10
DEFAULT_MAX_MEAN_Y = 0.10
DEFAULT_MAX_ABS_X = 3.50
DEFAULT_MAX_ABS_Y = 0.10
WORD_Y_BIN = 4.0


@dataclass(frozen=True)
class PdfParityCase:
    name: str
    input_script: Path
    reference_pdf: Path
    report_name: str
    ignore_case: bool
    included_pages: tuple[int, ...]
    ignored_pages: tuple[int, ...]
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
        input_script=ROOT / "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/big-fish/extracted/reference.pdf",
        report_name="verify-big-fish-pdf-parity",
        ignore_case=False,
        included_pages=(),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=3.50,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="little-women",
        input_script=ROOT
        / "tests/fixtures/corpus/public/little-women/source/source.fountain",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/little-women/extracted/reference.pdf",
        report_name="verify-little-women-pdf-parity",
        ignore_case=False,
        included_pages=(),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=2.25,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="little-women-fdx",
        input_script=ROOT
        / "tests/fixtures/corpus/public/little-women/source/source.fdx",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/little-women/source/source.pdf",
        report_name="verify-little-women-fdx-pdf-parity",
        ignore_case=False,
        included_pages=(),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=2.25,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="big-fish-scene-numbers",
        input_script=ROOT
        / "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fountain",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/big-fish-scene-numbers/extracted/reference.pdf",
        report_name="verify-big-fish-scene-numbers-pdf-parity",
        ignore_case=False,
        included_pages=(),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=2.25,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="big-fish-scene-numbers-fdx",
        input_script=ROOT
        / "tests/fixtures/corpus/public/big-fish-scene-numbers/source/source.fdx",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/big-fish-scene-numbers/extracted/reference.pdf",
        report_name="verify-big-fish-scene-numbers-fdx-pdf-parity",
        ignore_case=False,
        included_pages=(),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=2.25,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="extranormal-fdx-body",
        input_script=ROOT / "tests/fixtures/corpus/public/extranormal/source/source.fdx",
        reference_pdf=ROOT / "tests/fixtures/corpus/public/extranormal/extracted/reference.pdf",
        report_name="verify-extranormal-fdx-body-pdf-parity",
        ignore_case=False,
        included_pages=(),
        # Ignore page 1 for now. The known mismatch here is title-page vertical
        # layout, while this probe exists to catch body margin/centering regressions.
        ignored_pages=(1,),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=3.50,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="title-page-cast-page-fdx-title-pages",
        input_script=ROOT / "tests/fixtures/fdx-import/title-page-cast-page.fdx",
        reference_pdf=ROOT / "tests/fixtures/fdx-import/title-page-cast-page-reference.pdf",
        report_name="verify-title-page-cast-page-fdx-title-pages-pdf-parity",
        ignore_case=False,
        # Compare only the imported title-section pages.
        included_pages=(1, 2),
        ignored_pages=(),
        max_mean_x=0.10,
        max_mean_y=0.10,
        max_abs_x=3.50,
        max_abs_y=0.10,
    ),
    PdfParityCase(
        name="title-pages-multi-fdx-title-pages",
        input_script=ROOT / "tests/fixtures/fdx-import/title-pages-multi.fdx",
        reference_pdf=ROOT / "tests/fixtures/fdx-import/title-pages-multi-reference.pdf",
        report_name="verify-title-pages-multi-fdx-title-pages-pdf-parity",
        ignore_case=False,
        # Compare only the imported title-section pages.
        included_pages=(1, 2),
        ignored_pages=(),
        max_mean_x=0.10,
        # The reference title page is authored in Courier Final Draft while JumpCut
        # renders with Courier Prime, which leaves a tiny uniform extraction-box y
        # offset even after the layout geometry is otherwise aligned.
        max_mean_y=0.15,
        max_abs_x=3.50,
        max_abs_y=0.15,
    ),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare generated JumpCut PDFs against reference PDFs."
    )
    parser.add_argument(
        "--no-default-cases",
        action="store_true",
        help="Only run explicitly provided --case entries.",
    )
    parser.add_argument(
        "--case",
        action="append",
        nargs=3,
        metavar=("NAME", "INPUT_SCRIPT", "REFERENCE_PDF"),
        help=(
            "Add an ad hoc parity case. May be provided more than once. "
            "Reports are written under target/pdf-placement-diagnostics/<NAME>."
        ),
    )
    parser.add_argument(
        "--max-mean-x",
        type=float,
        default=DEFAULT_MAX_MEAN_X,
        help="Default threshold for mean word x delta on ad hoc cases.",
    )
    parser.add_argument(
        "--max-mean-y",
        type=float,
        default=DEFAULT_MAX_MEAN_Y,
        help="Default threshold for mean word y delta on ad hoc cases.",
    )
    parser.add_argument(
        "--max-abs-x",
        type=float,
        default=DEFAULT_MAX_ABS_X,
        help="Default threshold for max absolute word x delta on ad hoc cases.",
    )
    parser.add_argument(
        "--max-abs-y",
        type=float,
        default=DEFAULT_MAX_ABS_Y,
        help="Default threshold for max absolute word y delta on ad hoc cases.",
    )
    parser.add_argument(
        "--ignore-case",
        action="store_true",
        help="Treat extracted word text as case-insensitive before geometry comparison.",
    )
    return parser.parse_args()


def run(cmd: list[str], cwd: Path) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def require_tool(name: str, install_hint: str) -> None:
    if shutil.which(name):
        return
    raise SystemExit(
        f"missing required tool: {name}\n"
        f"install hint: {install_hint}"
    )


def output_pdf_for(case: PdfParityCase) -> Path:
    return TARGET_DIR / f"{case.name}.pdf"


def report_name_for(case_name: str) -> str:
    return (
        f"verify-{case_name}-pdf-parity"
        .replace(" ", "-")
        .replace("/", "-")
    )


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

    # Sort by visual reading order (page, y row, x) so comparison is
    # position-based rather than dependent on PDF content-stream order.
    # Words within 2pt of the same y are treated as the same line.
    boxes.sort(key=lambda b: (b.page_number, round(b.y_min / WORD_Y_BIN), b.x_min))
    return boxes


def normalize_word_text(text: str, ignore_case: bool) -> str:
    return text.lower() if ignore_case else text


def filter_ignored_pages(boxes: list[WordBox], ignored_pages: tuple[int, ...]) -> list[WordBox]:
    if not ignored_pages:
        return boxes
    ignored = set(ignored_pages)
    return [box for box in boxes if box.page_number not in ignored]


def filter_included_pages(boxes: list[WordBox], included_pages: tuple[int, ...]) -> list[WordBox]:
    if not included_pages:
        return boxes
    included = set(included_pages)
    return [box for box in boxes if box.page_number in included]


def compare_word_boxes(
    actual: list[WordBox], reference: list[WordBox], ignore_case: bool
) -> tuple[dict, list[str]]:
    first_text_mismatch = None
    if len(actual) != len(reference):
        mismatch_index = min(len(actual), len(reference))
        first_text_mismatch = {
            "index": mismatch_index,
            "reason": "word-count",
            "actual_word_count": len(actual),
            "reference_word_count": len(reference),
        }
        text_mismatch = True
    else:
        text_mismatch = False
        for index, (actual_box, reference_box) in enumerate(zip(actual, reference)):
            if actual_box.page_number != reference_box.page_number:
                first_text_mismatch = {
                    "index": index,
                    "reason": "page-number",
                    "actual": {
                        "page_number": actual_box.page_number,
                        "text": actual_box.text,
                    },
                    "reference": {
                        "page_number": reference_box.page_number,
                        "text": reference_box.text,
                    },
                }
                text_mismatch = True
                break
            if normalize_word_text(actual_box.text, ignore_case) != normalize_word_text(
                reference_box.text, ignore_case
            ):
                first_text_mismatch = {
                    "index": index,
                    "reason": "word-text",
                    "actual": {
                        "page_number": actual_box.page_number,
                        "text": actual_box.text,
                    },
                    "reference": {
                        "page_number": reference_box.page_number,
                        "text": reference_box.text,
                    },
                }
                text_mismatch = True
                break

    if text_mismatch:
        report = {
            "text_mismatch": True,
            "first_text_mismatch": first_text_mismatch,
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
    review_lines = [
        f"# {case.name} PDF parity",
        "",
        f"- text mismatch: {report['text_mismatch']}",
        (
            f"- included pages: {', '.join(map(str, case.included_pages))}"
            if case.included_pages
            else "- included pages: all"
        ),
        (
            f"- ignored pages: {', '.join(map(str, case.ignored_pages))}"
            if case.ignored_pages
            else "- ignored pages: none"
        ),
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

    if report.get("first_text_mismatch"):
        mismatch = report["first_text_mismatch"]
        review_lines.extend(
            [
                "",
                "## First Text Mismatch",
                "",
                f"- index: {mismatch['index']}",
                f"- reason: {mismatch['reason']}",
            ]
        )
        if "actual" in mismatch and "reference" in mismatch:
            review_lines.extend(
                [
                    f"- actual: page {mismatch['actual']['page_number']} `{mismatch['actual']['text']}`",
                    f"- reference: page {mismatch['reference']['page_number']} `{mismatch['reference']['text']}`",
                ]
            )
        else:
            review_lines.extend(
                [
                    f"- actual word count: {mismatch['actual_word_count']}",
                    f"- reference word count: {mismatch['reference_word_count']}",
                ]
            )

    review_path.write_text("\n".join(review_lines) + "\n")

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
            str(case.input_script),
            str(output_pdf),
        ],
        cwd=ROOT,
    )

    report, failures = compare_word_boxes(
        filter_ignored_pages(
            filter_included_pages(extract_word_boxes(output_pdf), case.included_pages),
            case.ignored_pages,
        ),
        filter_ignored_pages(
            filter_included_pages(extract_word_boxes(case.reference_pdf), case.included_pages),
            case.ignored_pages,
        ),
        case.ignore_case,
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


def build_cases(args: argparse.Namespace) -> list[PdfParityCase]:
    cases = [] if args.no_default_cases else list(CASES)

    for name, input_script, reference_pdf in args.case or []:
        case_name = name.strip()
        cases.append(
            PdfParityCase(
                name=case_name,
                input_script=(ROOT / input_script).resolve()
                if not Path(input_script).is_absolute()
                else Path(input_script),
                reference_pdf=(ROOT / reference_pdf).resolve()
                if not Path(reference_pdf).is_absolute()
                else Path(reference_pdf),
                report_name=report_name_for(case_name),
                ignore_case=args.ignore_case,
                included_pages=(),
                ignored_pages=(),
                max_mean_x=args.max_mean_x,
                max_mean_y=args.max_mean_y,
                max_abs_x=args.max_abs_x,
                max_abs_y=args.max_abs_y,
            )
        )

    return cases


def main() -> int:
    args = parse_args()
    require_tool("cargo", "install Rust and ensure cargo is on PATH")
    require_tool(
        "pdftotext",
        "install Poppler utilities (for example: apt install poppler-utils)",
    )

    cases = build_cases(args)
    if not cases:
        raise SystemExit("no PDF parity cases selected")

    any_failures = False
    for case in cases:
        failures = check_case(case)
        if failures:
            any_failures = True
    return 1 if any_failures else 0


if __name__ == "__main__":
    sys.exit(main())
