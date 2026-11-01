# SPDX-License-Identifier: AGPL-3.0-or-later
"""Loop scope expectations reject condition-wrapper and cleanup regressions."""
import unittest

from generate_local_scope_loop_emission_vectors import OUTPUT
from local_scope_loop_emission_model import validate


class LocalScopeLoopEmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed_case(self, scenario: int, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(f"L {scenario}\n".encode("ascii"))
        self.assertIn(old, self.trace[start:].split(b"\nL ", 1)[0])
        return self.trace[:start] + self.trace[start:].replace(old, new, 1)

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_case_inventory(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"L 9\n")], self.trace + self.trace,
                      self.trace.replace(b"L 0\n", b"L 99\n", 1)):
            with self.subTest(trace=trace[:16]), self.assertRaises(ValueError):
                validate(trace)

    def test_present_for_condition_cannot_be_treated_as_omitted(self) -> None:
        for scenario in (5, 6, 7):
            for old, new in ((b"Q 1", b"Q 0"), (b" 94 ", b" ")):
                trace = self.changed_case(scenario, old, new)
                with self.subTest(scenario=scenario, old=old), self.assertRaises(ValueError):
                    validate(trace)

    def test_dynamic_for_break_cleans_the_private_body_local(self) -> None:
        for old, new in ((b"23:1 ", b""), (b"R 0 4 2 4 7", b"R 0 5 2 5 23")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(7, old, new))

    def test_omitted_condition_has_different_export_and_cleanup(self) -> None:
        for old, new in ((b"Q 0", b"Q 1"), (b"98 99", b"23:1 98 99"),
                         (b"R 0 5 2 5 23", b"R 0 4 2 4 7")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(9, old, new))

    def test_branch_and_nested_loop_states_remain_checked(self) -> None:
        for scenario, old, new in ((4, b"4 11 0", b"4 15 0"),
                                   (8, b"23:1", b"23:0"), (6, b"B 1 5", b"B 0 5")):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, old, new))


if __name__ == "__main__":
    unittest.main()
