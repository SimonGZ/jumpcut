#!/usr/bin/env python3

import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path("/ductor/workspace/jumpcut")
TARGET_DIR = ROOT / "target" / "pdf-parity"


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
        max_abs_x=1.25,
        max_abs_y=0.10,
    ),
]


def run(cmd: list[str], cwd: Path) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def load_report(report_path: Path) -> dict:
    return json.loads(report_path.read_text())


def output_pdf_for(case: PdfParityCase) -> Path:
    return TARGET_DIR / f"{case.name}.pdf"


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

    run(
        [
            "python3",
            str(ROOT / "scripts/pdf_text_placement_diagnostics.py"),
            str(output_pdf),
            str(case.reference_pdf),
            "--name",
            case.report_name,
        ],
        cwd=ROOT,
    )

    report_path = ROOT / "target/pdf-placement-diagnostics" / case.report_name / "report.json"
    report = load_report(report_path)
    summary = report["summary"]

    failures: list[str] = []
    if report["text_mismatch"]:
        failures.append("text mismatch detected")
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
