#!/usr/bin/env python3
"""Require a successful CI workflow on the exact commit being packaged."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from typing import Any
from urllib.parse import urlencode

REQUIRED_JOBS = ("windows-latest", "ubuntu-22.04")
PENDING_STATUSES = frozenset(
    {"queued", "in_progress", "waiting", "requested", "pending", "waiting_for_review"}
)
WORKFLOW_FILE = "ci.yml"


def fail(message: str, code: int = 1) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(code)


def gh_api(path: str) -> Any:
    helper = os.environ.get("RNGKIT_GH_API")
    if helper:
        command = [helper, path]
    else:
        command = ["gh", "api", path]
    proc = subprocess.run(command, check=False, capture_output=True, text=True)
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip() or f"exit {proc.returncode}"
        fail(f"GitHub API request failed for {path}: {detail}", 2)
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        fail(f"GitHub API returned invalid JSON for {path}: {exc}", 2)
    raise AssertionError


def select_run(runs: list[dict[str, Any]], sha: str) -> dict[str, Any] | None:
    matched = [run for run in runs if run.get("head_sha") == sha]
    foreign = [run for run in runs if run.get("head_sha") != sha]
    if foreign and not matched:
        other = foreign[0].get("head_sha") or "unknown"
        fail(
            f"CI lookup for commit {sha} returned runs for a different commit "
            f"({other}); refusing to package another SHA"
        )
    if not matched:
        return None
    matched.sort(key=lambda run: str(run.get("created_at") or ""), reverse=True)
    return matched[0]


def describe_run(run: dict[str, Any]) -> str:
    run_id = run.get("id")
    url = run.get("html_url") or f"run {run_id}"
    return f"run {run_id} ({url})"


def require_jobs(run: dict[str, Any], repo: str, required: tuple[str, ...]) -> None:
    run_id = run.get("id")
    payload = gh_api(f"repos/{repo}/actions/runs/{run_id}/jobs")
    jobs = payload.get("jobs") if isinstance(payload, dict) else None
    if not isinstance(jobs, list):
        fail(f"CI {describe_run(run)} returned no jobs list")
    by_name = {job.get("name"): job for job in jobs if isinstance(job, dict)}
    missing = [name for name in required if name not in by_name]
    if missing:
        fail(
            f"CI {describe_run(run)} is missing required jobs: {', '.join(missing)}"
        )
    failed: list[str] = []
    pending: list[str] = []
    for name in required:
        job = by_name[name]
        status = job.get("status")
        conclusion = job.get("conclusion")
        if status in PENDING_STATUSES or status != "completed":
            pending.append(f"{name}={status}")
            continue
        if conclusion != "success":
            failed.append(f"{name}={conclusion}")
    if pending:
        fail(
            f"CI pending for commit {run.get('head_sha')}: {describe_run(run)} "
            f"jobs still running ({', '.join(pending)}). Wait for CI to finish, "
            "then re-run Release."
        )
    if failed:
        fail(
            f"CI failed for commit {run.get('head_sha')}: {describe_run(run)} "
            f"required jobs did not succeed ({', '.join(failed)})"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sha", required=True, help="commit SHA being packaged")
    parser.add_argument("--repo", required=True, help="owner/name")
    parser.add_argument("--workflow", default=WORKFLOW_FILE)
    parser.add_argument(
        "--require-job",
        action="append",
        dest="require_jobs",
        default=[],
        help="required CI job name (repeatable)",
    )
    args = parser.parse_args()
    sha = args.sha.strip().lower()
    if not re_full_sha(sha):
        fail(f"invalid commit SHA {args.sha!r}")
    required = tuple(args.require_jobs) if args.require_jobs else REQUIRED_JOBS
    query = urlencode({"head_sha": sha, "per_page": "100"})
    payload = gh_api(f"repos/{args.repo}/actions/workflows/{args.workflow}/runs?{query}")
    runs = payload.get("workflow_runs") if isinstance(payload, dict) else None
    if not isinstance(runs, list):
        fail(f"CI lookup for commit {sha} returned no workflow_runs list", 2)
    normalized = []
    for run in runs:
        if not isinstance(run, dict):
            continue
        copy = dict(run)
        copy["head_sha"] = str(copy.get("head_sha") or "").lower()
        normalized.append(copy)
    run = select_run(normalized, sha)
    if run is None:
        fail(
            f"CI absent for commit {sha}: no {args.workflow} run was found. "
            "Push the commit to main or open a pull request, wait for Windows "
            "and Ubuntu CI to finish, then re-run Release."
        )
    if run.get("head_sha") != sha:
        fail(
            f"CI {describe_run(run)} is for commit {run.get('head_sha')}, not {sha}; "
            "refusing to package another SHA"
        )
    status = run.get("status")
    conclusion = run.get("conclusion")
    if status in PENDING_STATUSES or status != "completed":
        fail(
            f"CI pending for commit {sha}: {describe_run(run)} is {status}. "
            "Wait for Windows and Ubuntu CI to finish, then re-run Release."
        )
    if conclusion != "success":
        fail(
            f"CI failed for commit {sha}: {describe_run(run)} conclusion={conclusion}"
        )
    require_jobs(run, args.repo, required)
    print(
        f"CI passed for commit {sha}: {describe_run(run)} "
        f"jobs {', '.join(required)}"
    )


def re_full_sha(value: str) -> bool:
    if len(value) not in {40, 64}:
        return False
    return all(ch in "0123456789abcdef" for ch in value)


if __name__ == "__main__":
    main()
