#!/usr/bin/env python3
"""Render deterministic NPA command diagnostics for GitHub step summaries."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Iterable


USAGE_OR_INTERNAL_KINDS = {"Usage", "Internal"}
SUMMARY_COLUMNS = (
    "file",
    "command",
    "status",
    "exit_code",
    "kind",
    "reason_code",
    "module",
    "path",
    "expected_hash",
    "actual_hash",
)


def main(argv: list[str]) -> int:
    paths = list(iter_json_paths(argv[1:] or ["ci-output"]))
    if not paths:
        print("No JSON diagnostics found under `ci-output`.")
        return 0

    rows: list[dict[str, str]] = []
    for path in paths:
        rows.extend(rows_for_file(path))

    print_markdown_table(rows)
    return 0


def iter_json_paths(arguments: Iterable[str]) -> Iterable[Path]:
    for argument in arguments:
        path = Path(argument)
        if path.is_dir():
            yield from sorted(child for child in path.iterdir() if child.suffix == ".json")
        elif path.is_file() and path.suffix == ".json":
            yield path


def rows_for_file(path: Path) -> list[dict[str, str]]:
    display_path = safe_path(path.as_posix())
    try:
        with path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError):
        return [
            base_row(
                display_path,
                command="-",
                status="failed",
                exit_code="2",
                kind="InvalidJson",
                reason_code="json_parse_failed",
            )
        ]

    if not isinstance(data, dict):
        return [
            base_row(
                display_path,
                command="-",
                status="failed",
                exit_code="2",
                kind="InvalidJson",
                reason_code="json_object_required",
            )
        ]

    command = clean_cell(data.get("command", "-"))
    status = clean_cell(data.get("status", "unknown"))
    diagnostics = data.get("diagnostics", [])
    if not isinstance(diagnostics, list):
        diagnostics = []
    exit_code = str(infer_exit_code(status, diagnostics))

    if not diagnostics:
        return [
            base_row(
                display_path,
                command=command,
                status=status,
                exit_code=exit_code,
                kind="-",
                reason_code="-",
            )
        ]

    rows = []
    for diagnostic in diagnostics:
        if not isinstance(diagnostic, dict):
            rows.append(
                base_row(
                    display_path,
                    command=command,
                    status=status,
                    exit_code=exit_code,
                    kind="InvalidDiagnostic",
                    reason_code="diagnostic_object_required",
                )
            )
            continue
        row = base_row(
            display_path,
            command=command,
            status=status,
            exit_code=exit_code,
            kind=clean_cell(diagnostic.get("kind", "-")),
            reason_code=clean_cell(diagnostic.get("reason_code", "-")),
        )
        row["module"] = clean_cell(diagnostic.get("module", "-"))
        row["path"] = safe_path(clean_cell(diagnostic.get("path", "-")))
        row["expected_hash"] = clean_cell(diagnostic.get("expected_hash", "-"))
        row["actual_hash"] = clean_cell(diagnostic.get("actual_hash", "-"))
        rows.append(row)
    return rows


def infer_exit_code(status: str, diagnostics: list[Any]) -> int:
    if status == "passed":
        return 0
    for diagnostic in diagnostics:
        if isinstance(diagnostic, dict) and diagnostic.get("kind") in USAGE_OR_INTERNAL_KINDS:
            return 2
    return 1


def base_row(
    path: str,
    *,
    command: str,
    status: str,
    exit_code: str,
    kind: str,
    reason_code: str,
) -> dict[str, str]:
    return {
        "file": path,
        "command": command,
        "status": status,
        "exit_code": exit_code,
        "kind": kind,
        "reason_code": reason_code,
        "module": "-",
        "path": "-",
        "expected_hash": "-",
        "actual_hash": "-",
    }


def safe_path(value: str) -> str:
    value = clean_cell(value).replace("\\", "/")
    if value in {"", "-"}:
        return "-"
    path = Path(value)
    if path.is_absolute():
        try:
            return path.resolve().relative_to(Path.cwd().resolve()).as_posix()
        except (OSError, ValueError):
            return path.name or "."
    parts = []
    for part in value.split("/"):
        if part in {"", ".", ".."}:
            continue
        parts.append(part)
    if not parts:
        return "."
    return "/".join(parts)


def clean_cell(value: Any) -> str:
    if value is None:
        return "-"
    text = str(value).replace("\r", " ").replace("\n", " ").replace("\t", " ")
    text = " ".join(text.split())
    return text or "-"


def markdown_cell(value: str) -> str:
    return clean_cell(value).replace("\\", "\\\\").replace("|", "\\|")


def print_markdown_table(rows: list[dict[str, str]]) -> None:
    print("| " + " | ".join(SUMMARY_COLUMNS) + " |")
    print("| " + " | ".join("---" for _ in SUMMARY_COLUMNS) + " |")
    for row in rows:
        print("| " + " | ".join(markdown_cell(row[column]) for column in SUMMARY_COLUMNS) + " |")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
