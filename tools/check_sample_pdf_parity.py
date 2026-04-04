#!/usr/bin/env python3

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path("/ductor/workspace/jumpcut")
DEFAULT_SAMPLE_FOUNTAIN = Path("/ductor/workspace/output_to_user/jumpcut-pdf-progress-sample.fountain")
DEFAULT_OUTPUT_PDF = Path("/ductor/workspace/output_to_user/jumpcut-pdf-progress-sample.pdf")
DEFAULT_REFERENCE_PDF = Path(
    "/ductor/workspace/telegram_files/2026-04-04/jumpcut-pdf-progress-reference_2.pdf"
)
DEFAULT_REPORT_NAME = "full-sample-vs-updated-reference-2"


def run(cmd: list[str], cwd: Path) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def load_report(report_path: Path) -> dict:
    return json.loads(report_path.read_text())


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Regenerate the sample PDF and check that placement stays within a small drift budget."
    )
    parser.add_argument("--sample-fountain", type=Path, default=DEFAULT_SAMPLE_FOUNTAIN)
    parser.add_argument("--output-pdf", type=Path, default=DEFAULT_OUTPUT_PDF)
    parser.add_argument("--reference-pdf", type=Path, default=DEFAULT_REFERENCE_PDF)
    parser.add_argument("--report-name", default=DEFAULT_REPORT_NAME)
    parser.add_argument("--max-mean-x", type=float, default=0.10)
    parser.add_argument("--max-mean-y", type=float, default=0.10)
    parser.add_argument("--max-abs-x", type=float, default=0.75)
    parser.add_argument("--max-abs-y", type=float, default=0.10)
    args = parser.parse_args()

    run(
        [
            "cargo",
            "run",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
            "--quiet",
            "--",
            str(args.sample_fountain),
            str(args.output_pdf),
            "--format",
            "pdf",
        ],
        cwd=ROOT.parent,
    )

    run(
        [
            "python3",
            str(ROOT / "scripts/pdf_text_placement_diagnostics.py"),
            str(args.output_pdf),
            str(args.reference_pdf),
            "--name",
            args.report_name,
        ],
        cwd=ROOT,
    )

    report_path = ROOT / "target/pdf-placement-diagnostics" / args.report_name / "report.json"
    report = load_report(report_path)
    summary = report["summary"]

    failures: list[str] = []
    if report["text_mismatch"]:
        failures.append("text mismatch detected")
    if abs(summary["mean_word_x_delta"]) > args.max_mean_x:
        failures.append(
            f"mean word x delta {summary['mean_word_x_delta']:.4f}pt exceeds {args.max_mean_x:.4f}pt"
        )
    if abs(summary["mean_word_y_delta"]) > args.max_mean_y:
        failures.append(
            f"mean word y delta {summary['mean_word_y_delta']:.4f}pt exceeds {args.max_mean_y:.4f}pt"
        )
    if summary["max_abs_word_x_delta"] > args.max_abs_x:
        failures.append(
            f"max abs word x delta {summary['max_abs_word_x_delta']:.4f}pt exceeds {args.max_abs_x:.4f}pt"
        )
    if summary["max_abs_word_y_delta"] > args.max_abs_y:
        failures.append(
            f"max abs word y delta {summary['max_abs_word_y_delta']:.4f}pt exceeds {args.max_abs_y:.4f}pt"
        )

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
        return 1

    print("status: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
