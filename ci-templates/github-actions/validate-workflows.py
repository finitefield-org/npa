#!/usr/bin/env python3
"""Validate copyable NPA GitHub Actions workflow templates."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Iterable


WORKFLOW_DIR = Path(__file__).resolve().parent

FORBIDDEN_FLAGS = (
    ("changed-module selector", ("--", "changed")),
    ("all-module selector", ("--", "all")),
    ("external checker mode", ("--checker ", "external")),
    ("registry lookup", ("--", "registry")),
    ("network package resolution", ("--", "network")),
    ("implicit latest package resolution", ("--", "latest")),
)

REQUIRED_PR_COMMANDS = (
    "npa package check --root . --json",
    "npa package build-certs --root . --check --json",
    "npa package check-hashes --root . --json",
    "npa package verify-certs --root . --checker reference --json",
    "npa package axiom-report --root . --check --json",
    "npa package index --root . --check --json",
)

REQUIRED_RELEASE_COMMANDS = REQUIRED_PR_COMMANDS + (
    "npa package verify-certs --root . --checker fast --json",
)

REQUIRED_COMMANDS_BY_WORKFLOW = {
    "npa-package-pr.yml": REQUIRED_PR_COMMANDS,
    "npa-package-release.yml": REQUIRED_RELEASE_COMMANDS,
}


def main(argv: list[str]) -> int:
    workflow_dir = Path(argv[1]) if len(argv) > 1 else WORKFLOW_DIR
    workflow_dir = workflow_dir.resolve()
    errors: list[str] = []

    workflows = sorted(workflow_dir.glob("*.yml"))
    if not workflows:
        errors.append(f"{workflow_dir}: no workflow templates found")

    errors.extend(validate_yaml_syntax(workflows))
    for workflow in workflows:
        text = workflow.read_text(encoding="utf-8")
        errors.extend(validate_forbidden_flags(workflow, text))

    for workflow_name, required_commands in REQUIRED_COMMANDS_BY_WORKFLOW.items():
        workflow = workflow_dir / workflow_name
        if not workflow.is_file():
            errors.append(f"{workflow}: required workflow template is missing")
            continue
        text = workflow.read_text(encoding="utf-8")
        errors.extend(validate_required_commands(workflow, text, required_commands))

    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1

    print("workflow validation ok")
    return 0


def validate_yaml_syntax(workflows: Iterable[Path]) -> list[str]:
    try:
        import yaml  # type: ignore[import-not-found]
    except ImportError:
        return ["PyYAML is required for local YAML syntax validation when actionlint is unavailable"]

    errors = []
    for workflow in workflows:
        try:
            with workflow.open("r", encoding="utf-8") as handle:
                yaml.safe_load(handle)
        except Exception as error:  # noqa: BLE001 - preserve parser message for CI output.
            errors.append(f"{workflow}: YAML syntax validation failed: {error}")
    return errors


def validate_forbidden_flags(workflow: Path, text: str) -> list[str]:
    errors = []
    for label, flag_parts in FORBIDDEN_FLAGS:
        flag = "".join(flag_parts)
        pattern = re.compile(r"(?<!\S)" + re.escape(flag) + r"(?![\w-])")
        if pattern.search(text):
            errors.append(f"{workflow}: forbidden base-template flag {flag!r} ({label})")
    return errors


def validate_required_commands(
    workflow: Path,
    text: str,
    required_commands: Iterable[str],
) -> list[str]:
    errors = []
    for command in required_commands:
        if command not in text:
            errors.append(f"{workflow}: missing required package command: {command}")
    return errors


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
