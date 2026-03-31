#!/usr/bin/env python3
"""Run JumpCut verification commands.

This script is a portable version of the jumpcut_verify tool, designed to live
inside the repository's tools/ directory.

Examples:
  python3 tools/verify.py
  python3 tools/verify.py --mode test
  python3 tools/verify.py --mode diagnostics --json
  python3 tools/verify.py --background
  python3 tools/verify.py --latest
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path


@dataclass
class StepResult:
    name: str
    command: list[str]
    exit_code: int
    duration_seconds: float


def parse_args() -> argparse.Namespace:
    # Default to the repo root (one level up from the tools/ directory)
    repo_root = Path(__file__).resolve().parents[1]

    parser = argparse.ArgumentParser(
        description="Run JumpCut verification directly."
    )
    parser.add_argument(
        "--repo",
        default=str(repo_root),
        help="Path to the jumpcut repo (default: %(default)s)",
    )
    parser.add_argument(
        "--mode",
        choices=("full", "test", "diagnostics"),
        default="full",
        help="Which verification steps to run (default: %(default)s)",
    )
    parser.add_argument(
        "--background",
        action="store_true",
        help="Launch the verification run as a detached background process.",
    )
    parser.add_argument(
        "--status",
        help="Show status for a specific state JSON file.",
    )
    parser.add_argument(
        "--latest",
        action="store_true",
        help="Show status for the most recent verification run in the repo.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Print the final summary as JSON.",
    )
    # Internal arguments for background mode
    parser.add_argument("--log-path", help=argparse.SUPPRESS)
    parser.add_argument("--summary-path", help=argparse.SUPPRESS)
    parser.add_argument("--state-path", help=argparse.SUPPRESS)
    return parser.parse_args()


def build_steps(mode: str) -> list[tuple[str, list[str]]]:
    steps: list[tuple[str, list[str]]] = []
    if mode in {"full", "test"}:
        steps.append(("cargo-test", ["cargo", "test"]))
    if mode in {"full", "diagnostics"}:
        steps.append(
            (
                "pagination-diagnostics",
                ["cargo", "run", "--bin", "pagination-diagnostics", "--", "all"],
            )
        )
    return steps


def check_diagnostics(repo: Path, log_handle) -> int:
    """Scan diagnostic outputs for reported issues/regressions and copy fixtures for easier review."""
    debug_dir = repo / "target" / "pagination-debug"
    if not debug_dir.is_dir():
        return 0

    header = "\n=== Checking diagnostic reports for issues ===\n"
    sys.stdout.write(header)
    log_handle.write(header)

    issues_found = 0

    # Check comparison-report.json files (Page Break Parity)
    for report_path in sorted(debug_dir.glob("**/comparison-report.json")):
        try:
            data = json.loads(report_path.read_text(encoding="utf-8"))
            
            # Copy the canonical fixture into the same folder for easy review
            fixture_rel_path = data.get("fixture_path")
            if fixture_rel_path:
                fixture_src = (repo / fixture_rel_path).resolve()
                if fixture_src.is_file():
                    fixture_dst = (report_path.parent / "canonical.page-breaks.json").resolve()
                    # Only copy if source and destination are different
                    if fixture_src != fixture_dst:
                        shutil.copy2(fixture_src, fixture_dst)
                        msg = f"info: copied canonical fixture to {fixture_dst.relative_to(repo.resolve())}\n"
                        log_handle.write(msg)

            total_issues = data.get("total_issues", 0)
            if total_issues > 0:
                rel_path = report_path.relative_to(repo)
                msg = f"FAIL: {rel_path} reports {total_issues} issues\n"
                sys.stdout.write(msg)
                log_handle.write(msg)
                issues_found += 1
        except Exception as e:
            msg = f"ERROR: failed to parse/process {report_path}: {e}\n"
            sys.stdout.write(msg)
            log_handle.write(msg)
            issues_found += 1

    # Check parity.json files (Line Break Parity)
    for parity_path in sorted(debug_dir.glob("**/parity.json")):
        try:
            data = json.loads(parity_path.read_text(encoding="utf-8"))
            disagreements = data.get("disagreement_count", 0)
            if disagreements > 0:
                rel_path = parity_path.relative_to(repo)
                msg = f"FAIL: {rel_path} reports {disagreements} line-break disagreements\n"
                sys.stdout.write(msg)
                log_handle.write(msg)
                issues_found += 1
        except Exception as e:
            msg = f"ERROR: failed to parse {parity_path}: {e}\n"
            sys.stdout.write(msg)
            log_handle.write(msg)
            issues_found += 1

    if issues_found == 0:
        msg = "SUCCESS: No issues found in diagnostic reports.\n"
        sys.stdout.write(msg)
        log_handle.write(msg)
        return 0
    else:
        msg = f"FAIL: Found issues in {issues_found} diagnostic reports.\n"
        sys.stdout.write(msg)
        log_handle.write(msg)
        return 1


def make_log_paths(repo: Path) -> tuple[Path, Path, Path]:
    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    log_dir = repo / "target" / "verification-runs"
    log_dir.mkdir(parents=True, exist_ok=True)
    return (
        log_dir / f"{stamp}.log",
        log_dir / f"{stamp}.json",
        log_dir / f"{stamp}.state.json",
    )


def run_step(name: str, command: list[str], repo: Path, log_handle) -> StepResult:
    started = time.monotonic()
    header = f"\n=== {name}: {' '.join(command)} ===\n"
    sys.stdout.write(header)
    sys.stdout.flush()
    log_handle.write(header)
    log_handle.flush()

    process = subprocess.Popen(
        command,
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=os.environ.copy(),
    )

    assert process.stdout is not None
    for line in process.stdout:
        sys.stdout.write(line)
        log_handle.write(line)
    exit_code = process.wait()
    duration = time.monotonic() - started

    footer = f"=== {name} finished with exit code {exit_code} in {duration:.1f}s ===\n"
    sys.stdout.write(footer)
    sys.stdout.flush()
    log_handle.write(footer)
    log_handle.flush()

    return StepResult(
        name=name,
        command=command,
        exit_code=exit_code,
        duration_seconds=duration,
    )


def write_json(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def is_pid_running(pid: int) -> bool:
    if pid <= 0:
        return False
    try:
        os.kill(pid, 0)
    except (ProcessLookupError, OSError):
        return False
    return True


def format_status(repo: Path, state_path: Path, as_json: bool) -> int:
    if not state_path.is_file():
        print(f"error: status file not found: {state_path}", file=sys.stderr)
        return 2

    state = json.loads(state_path.read_text(encoding="utf-8"))
    status = state.get("status", "unknown")
    pid = int(state.get("pid", 0) or 0)
    if status == "running" and not is_pid_running(pid):
        status = "stale"
        state["status"] = status

    if as_json:
        print(json.dumps(state, indent=2))
    else:
        print(f"status: {status}")
        print(f"repo: {state.get('repo', repo)}")
        print(f"mode: {state.get('mode', '?')}")
        if pid:
            print(f"pid: {pid}")
        print(f"log: {state.get('log_path', '')}")
        summary_path = state.get("summary_path", "")
        if summary_path:
            print(f"summary: {summary_path}")
        if status in {"done", "failed"}:
            print(f"success: {state.get('success', False)}")
            duration = state.get("duration_seconds")
            if duration is not None:
                print(f"duration_seconds: {duration:.3f}")

    return 0 if status in {"running", "done"} else 1


def latest_state_path(repo: Path) -> Path | None:
    run_dir = repo / "target" / "verification-runs"
    if not run_dir.is_dir():
        return None
    candidates = sorted(run_dir.glob("*.state.json"))
    return candidates[-1] if candidates else None


def spawn_background(args: argparse.Namespace, repo: Path) -> int:
    log_path, summary_path, state_path = make_log_paths(repo)
    python = sys.executable
    cmd = [
        python,
        str(Path(__file__).resolve()),
        "--repo",
        str(repo),
        "--mode",
        args.mode,
        "--log-path",
        str(log_path),
        "--summary-path",
        str(summary_path),
        "--state-path",
        str(state_path),
    ]

    initial_state = {
        "status": "running",
        "repo": str(repo),
        "mode": args.mode,
        "pid": 0,
        "started_at": datetime.now(UTC).isoformat(),
        "log_path": str(log_path),
        "summary_path": str(summary_path),
        "state_path": str(state_path),
    }
    write_json(state_path, initial_state)

    with log_path.open("a", encoding="utf-8") as log_handle:
        process = subprocess.Popen(
            cmd,
            cwd=repo,
            stdout=log_handle,
            stderr=subprocess.STDOUT,
            text=True,
            start_new_session=True,
            env=os.environ.copy(),
        )

    initial_state["pid"] = process.pid
    write_json(state_path, initial_state)

    if args.json:
        print(json.dumps(initial_state, indent=2))
    else:
        print("started background verification")
        print(f"pid: {process.pid}")
        print(f"log: {log_path}")
        print(f"summary: {summary_path}")
        print(f"state: {state_path}")

    return 0


def main() -> int:
    args = parse_args()
    repo = Path(args.repo).resolve()
    if not repo.is_dir():
        print(f"error: repo not found: {repo}", file=sys.stderr)
        return 2

    if args.status:
        return format_status(repo, Path(args.status).resolve(), args.json)

    if args.latest:
        state_path = latest_state_path(repo)
        if state_path is None:
            print("error: no verification runs found", file=sys.stderr)
            return 2
        return format_status(repo, state_path, args.json)

    if args.background:
        return spawn_background(args, repo)

    default_log_path, default_json_path, default_state_path = make_log_paths(repo)
    log_path = Path(args.log_path).resolve() if args.log_path else default_log_path
    json_path = Path(args.summary_path).resolve() if args.summary_path else default_json_path
    state_path = Path(args.state_path).resolve() if args.state_path else default_state_path
    steps = build_steps(args.mode)
    results: list[StepResult] = []
    overall_started = time.monotonic()
    started_at = datetime.now(UTC).isoformat()
    
    if state_path.is_file():
        try:
            existing_state = json.loads(state_path.read_text(encoding="utf-8"))
            started_at = str(existing_state.get("started_at") or started_at)
        except Exception:
            pass

    state_payload = {
        "status": "running",
        "repo": str(repo),
        "mode": args.mode,
        "pid": os.getpid(),
        "started_at": started_at,
        "log_path": str(log_path),
        "summary_path": str(json_path),
        "state_path": str(state_path),
    }
    write_json(state_path, state_payload)

    with log_path.open("w", encoding="utf-8") as log_handle:
        # Run standard steps (Tests, Generation)
        for name, command in steps:
            result = run_step(name, command, repo, log_handle)
            results.append(result)
        
        # Run Diagnostic check step if we are in a mode that generates them
        if args.mode in {"full", "diagnostics"}:
            diag_exit_code = check_diagnostics(repo, log_handle)
            results.append(StepResult(
                name="check-diagnostics",
                command=["internal-check"],
                exit_code=diag_exit_code,
                duration_seconds=0.0
            ))

    total_duration = time.monotonic() - overall_started
    # Success is defined as ALL steps having an exit code of 0
    success = all(result.exit_code == 0 for result in results)
    
    summary = {
        "repo": str(repo),
        "mode": args.mode,
        "success": success,
        "duration_seconds": total_duration,
        "started_at": started_at,
        "finished_at": datetime.now(UTC).isoformat(),
        "log_path": str(log_path),
        "steps": [asdict(result) for result in results],
    }

    write_json(json_path, summary)
    summary["summary_path"] = str(json_path)

    state_payload.update(
        {
            "status": "done" if success else "failed",
            "success": success,
            "duration_seconds": total_duration,
            "finished_at": summary["finished_at"],
        }
    )
    write_json(state_path, state_payload)

    if args.json:
        print(json.dumps(summary, indent=2))
    else:
        status = "PASS" if success else "FAIL"
        print(f"\n{status}: {args.mode} verification")
        print(f"log: {log_path}")
        print(f"summary: {json_path}")
        print(f"state: {state_path}")

    return 0 if success else 1


if __name__ == "__main__":
    raise SystemExit(main())
