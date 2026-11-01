# SPDX-License-Identifier: AGPL-3.0-or-later
"""ASCII filename-helper corpus and an independent segment/regular-expression model."""
import re


def canonical(raw: bytes) -> bytes:
    prefix = raw.split(b"\0", 1)[0]
    segments = re.split(rb"[\x01-\x1f]", prefix)
    return b"".join(re.sub(rb"/+", b"/", part.replace(b"\\", b"/").lstrip(b"/").lower())
                    for part in segments)


def cases() -> list[tuple[bytes, int]]:
    paths = [b"", b"MAIN", b"Maps\\MP\\_Utility", b"/\\///Maps\\\\MP//Utility/",
             b"a\n/b", b"a/\n/b", b"a\rb\\c", b"  A B  ", b"../A/./B", b"A.GSC",
             b"a\0PRIVATE_UNUSED_TAIL", b"\0tail", b"\\\\", b"////", b"a\\\\b////c"]
    paths += [b"A" + bytes([byte]) + b"/B" for byte in range(1, 128)]
    paths += [b"A" * count for count in (63, 64, 1023, 1024, 8192)]
    result = []
    for raw in paths:
        size = len(canonical(raw))
        for capacity in sorted({1, max(1, size), size + 1, size + 2}):
            result.append((raw, capacity))
    return result


def verify(rows: bytes, corpus: list[tuple[bytes, int]]) -> None:
    actual = rows.decode("ascii").splitlines()
    if len(actual) != len(corpus):
        raise ValueError("Filename helper case count differs")
    for row, (raw, capacity) in zip(actual, corpus, strict=True):
        parts = row.split()
        expected = canonical(raw)
        rejected = len(expected) >= capacity
        fields = ["P", raw.hex() or "~", str(capacity), "E" if rejected else "S",
                  (expected[:capacity] if rejected else expected).hex() or "~"]
        if parts != fields:
            raise ValueError("Native filename helper differs from independent segment model")
