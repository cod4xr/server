# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard constant-versus-dynamic scope consequences and analysis-only boundaries."""
import unittest

from generate_local_scope_constant_vectors import OUTPUT
from local_scope_constant_model import validate


class LocalScopeConstantTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed_case(self, scenario: int, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(f"K {scenario}\n".encode("ascii"))
        self.assertIn(old, self.trace[start:].split(b"\nK ", 1)[0])
        return self.trace[:start] + self.trace[start:].replace(old, new, 1)

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_case_inventory(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"K 7\n")], self.trace + self.trace,
                      self.trace.replace(b"K 0\n", b"K 99\n", 1)):
            with self.subTest(trace=trace[:16]), self.assertRaises(ValueError):
                validate(trace)

    def test_false_cases_remain_analysis_only(self) -> None:
        for scenario in (1, 3):
            for old, new in ((b"O p:0", b"O 94 p:1"), (b"Q 0", b"Q 1")):
                with self.subTest(scenario=scenario, old=old), self.assertRaises(ValueError):
                    validate(self.changed_case(scenario, old, new))

    def test_true_break_names_survive(self) -> None:
        for scenario in (0, 2):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"98", b"23:1 98"))

    def test_mixed_exit_initialization_remains_separate(self) -> None:
        for scenario, old, new in ((4, b"4 11 0", b"4 15 0"), (5, b"5 23 0", b"5 31 0")):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, old, new))

    def test_nonreference_release_callbacks_are_accounted(self) -> None:
        for scenario in range(8):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"D 1", b"D 0"))

    def test_disjoint_break_paths_do_not_export_their_private_names(self) -> None:
        for scenario in (6, 7):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"23:1 ", b""))


if __name__ == "__main__":
    unittest.main()
