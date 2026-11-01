# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected emission traces must satisfy independent counts and scope state."""
import unittest

from generate_local_scope_emission_vectors import OUTPUT
from local_scope_emission_model import validate


class LocalScopeEmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def test_complete_native_trace(self) -> None:
        validate(self.trace)

    def test_missing_unknown_and_duplicate_cases_reject(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"T 1 5 2")], self.trace + self.trace,
                      self.trace.replace(b"I 0", b"I 9", 1)):
            with self.subTest(trace=trace[:10]), self.assertRaises(ValueError):
                validate(trace)

    def test_later_declaration_does_not_retain_private_local(self) -> None:
        start = self.trace.index(b"I 1\n")
        trace = self.trace[:start] + self.trace[start:].replace(b"23:1", b"23:0", 1)
        with self.assertRaises(ValueError):
            validate(trace)

    def test_prior_local_and_common_join_do_not_gain_cleanup(self) -> None:
        for marker in (b"I 2\n", b"E 0\n"):
            start = self.trace.index(marker)
            trace = self.trace[:start] + self.trace[start:].replace(b" p:", b" 23:1 p:", 1)
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                validate(trace)

    def test_continue_and_break_use_distinct_public_counts(self) -> None:
        for marker, old, new in ((b"T 0 4 1\n", b"23:3", b"23:2"),
                                 (b"T 1 4 1\n", b"23:2", b"23:3")):
            start = self.trace.index(marker)
            trace = self.trace[:start] + self.trace[start:].replace(old, new, 1)
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                validate(trace)

    def test_initialization_names_and_reset_created_count_are_checked(self) -> None:
        for old, new in ((b"F 0 2 2 3 7", b"F 0 3 2 3 7"),
                         (b"3 7 0 n:1", b"3 3 0 n:1"), (b"n:1 n:3 n:2", b"n:1 n:2 n:3")):
            with self.subTest(new=new), self.assertRaises(ValueError):
                validate(self.trace.replace(old, new, 1))


if __name__ == "__main__":
    unittest.main()
