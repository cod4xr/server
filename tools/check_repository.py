#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Enforce source sizes, metadata inheritance, and dependency boundaries."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

import script_test_targets

ROOT = Path(__file__).resolve().parents[1]
SOURCE_SUFFIXES = {
    ".rs", ".py", ".c", ".cpp", ".h", ".js", ".ts", ".tsx", ".ps1", ".sh"
}
EXCLUDED = {
    ".git", "target", "reference", ".venv", "__pycache__", "node_modules"
}
MAX_LINES = 400


def main() -> None:
    failures = []
    largest = (0, "")
    checked_files = 0
    for directory, subdirectories, filenames in os.walk(ROOT):
        subdirectories[:] = [
            name for name in subdirectories if name not in EXCLUDED
        ]
        for filename in filenames:
            path = Path(directory) / filename
            if path.suffix not in SOURCE_SUFFIXES:
                continue
            count = len(path.read_text(encoding="utf-8").splitlines())
            relative = path.relative_to(ROOT).as_posix()
            largest = max(largest, (count, relative))
            checked_files += 1
            if count > MAX_LINES:
                failures.append(
                    f"{relative}: {count} lines exceeds {MAX_LINES}"
                )

    rules = json.loads(
        (ROOT / "dependency-rules.json").read_text(encoding="utf-8")
    )
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        cwd=ROOT, text=True
    ))
    members = set(metadata["workspace_members"])
    names = set()
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue
        name = package["name"]
        names.add(name)
        if name not in rules:
            failures.append(f"{name}: missing dependency policy")
            continue
        manifest = tomllib.loads(
            Path(package["manifest_path"]).read_text(encoding="utf-8")
        )
        if manifest.get("lints", {}).get("workspace") is not True:
            failures.append(f"{name}: must inherit workspace lints")
        inherited_fields = (
            "version", "edition", "rust-version", "license", "publish"
        )
        for field in inherited_fields:
            if manifest["package"].get(field) != {"workspace": True}:
                failures.append(f"{name}: must inherit package.{field}")
        for dependency in package["dependencies"]:
            if dependency["name"] not in rules[name]:
                failures.append(
                    f"{name}: unapproved dependency {dependency['name']}"
                )
    if names != set(rules):
        failures.append(
            "Dependency policy must describe exactly the workspace crates"
        )
    failures.extend(script_test_targets.validate(ROOT))
    if failures:
        print("\n".join(failures), file=sys.stderr)
        raise SystemExit(1)
    print(
        f"Checked {checked_files} source files and {len(names)} crate policies"
    )
    print(
        f"Largest source: {largest[1]} "
        f"({largest[0]} lines; ceiling {MAX_LINES})"
    )


if __name__ == "__main__":
    main()
