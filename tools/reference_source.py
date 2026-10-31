#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Read and extract functions only from the pinned, hashed reference source."""

import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]


def declaration(source: str, pattern: str) -> str:
    """Extract one source declaration using an explicit caller-owned pattern."""
    matches = list(re.finditer(pattern, source, re.DOTALL))
    if len(matches) != 1:
        raise ValueError("Expected one matching source declaration")
    return matches[0][0]


class ReferenceSource:
    """A verified, immutable-in-memory snapshot of recorded upstream files."""

    def __init__(self, reference: Path):
        lock = json.loads(
            (ROOT / "upstream.lock.json").read_text(encoding="utf-8")
        )
        self.revision = subprocess.check_output(
            ["git", "-C", str(reference), "rev-parse", "HEAD"], text=True
        ).strip()
        if self.revision != lock["revision"]:
            raise ValueError("Reference revision differs from lock file")
        self.files = {}
        for name, expected in lock["files"].items():
            data = (reference / name).read_bytes().replace(b"\r\n", b"\n")
            if hashlib.sha256(data).hexdigest() != expected:
                raise ValueError(f"Reference hash mismatch: {name}")
            self.files[name] = data.decode("utf-8")

    def function(self, filename: str, name: str) -> str:
        """Extract a function verbatim, tracking C comments and string literals."""
        source = self.files[filename]
        pattern = (
            rf"(?m)^(?:(?:static|inline|__cdecl|__optimize3|__regparm[123])\s+)*"
            rf"(?:void|(?:char|byte)\s*\*|bool|int|unsigned(?:\s+int)?|"
            rf"u?int(?:8|16|32|64)_t|qboolean)\s*"
            rf"(?:(?:__cdecl|QDECL|__regparm[123])\s+)*{re.escape(name)}\s*\([^;{{]*\)\s*\{{"
        )
        matches = list(re.finditer(pattern, source))
        if len(matches) != 1:
            raise ValueError(f"Expected one definition of {name}")
        start = matches[0].start()
        cursor = matches[0].end()
        depth = 1
        quote = None
        while cursor < len(source) and depth:
            char = source[cursor]
            if quote:
                if char == "\\":
                    cursor += 2
                    continue
                if char == quote:
                    quote = None
            elif source.startswith("//", cursor):
                end = source.find("\n", cursor)
                cursor = len(source) if end < 0 else end
                continue
            elif source.startswith("/*", cursor):
                end = source.find("*/", cursor + 2)
                if end < 0:
                    raise ValueError("Unterminated reference comment")
                cursor = end + 2
                continue
            elif char in "\"'":
                quote = char
            elif char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
            cursor += 1
        if depth:
            raise ValueError(f"Unterminated function {name}")
        return source[start:cursor]
