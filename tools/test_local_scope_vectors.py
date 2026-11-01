# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reject broken native scope trace admission without invoking the native driver."""
import unittest

from generate_local_scope_vectors import OUTPUT, validate


class LocalScopeTraceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(line for line in OUTPUT.read_bytes().splitlines()
                                 if not line.startswith(b"#")) + b"\n"

    def test_complete_trace_is_admitted(self) -> None:
        validate(self.trace)

    def test_empty_missing_duplicate_and_unrecognized_cases_fail(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"C 18 1")], self.trace + self.trace,
                      self.trace.replace(b"C 0 0", b"C 99 0"), self.trace + b"\n"):
            with self.subTest(trace=trace[:24]), self.assertRaises(ValueError):
                validate(trace)

    def test_missing_parent_and_repeated_parent_fail(self) -> None:
        row = b"R 0 2 2 2 3 0 n:1 n:3\n"
        for trace in (self.trace.replace(row, b"", 1), self.trace.replace(row, row + row, 1)):
            with self.subTest(trace=trace[:80]), self.assertRaises(ValueError):
                validate(trace)

    def test_unknown_rows_and_allocations_fail(self) -> None:
        for old, new in ((b"A 4096", b"A 8192"), (b"B 0 2 2", b"Z 0 2 2"),
                         (b"C 0 0", b"C 0"), (b"A 4096", b"A 4096 extra")):
            with self.subTest(new=new), self.assertRaises(ValueError):
                validate(self.trace.replace(old, new, 1))

    def test_width_duplicates_and_inherited_bits_fail(self) -> None:
        for old, new in ((b"2 2 2 3 0 n:1", b"2 2 65 3 0 n:1"),
                         (b"n:1 n:3", b"n:1 n:1"), (b"n:1 n:3", b"n:1 x:3"),
                         (b"2 2 2 3 0 n:1", b"2 2 2 2 0 n:1")):
            with self.subTest(new=new), self.assertRaises(ValueError):
                validate(self.trace.replace(old, new, 1))

    def test_parent_consequences_are_checked_independently(self) -> None:
        trace = self.trace.replace(b"R 0 2 2 2 3 0 n:1 n:3\n",
                                  b"R 0 2 2 3 3 0 n:1 n:3 n:2\n", 1)
        with self.assertRaises(ValueError):
            validate(trace)


if __name__ == "__main__":
    unittest.main()
