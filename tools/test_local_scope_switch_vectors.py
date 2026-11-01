# SPDX-License-Identifier: AGPL-3.0-or-later
"""Switch scope/transfer expectations reject damaged cleanup or joins."""
import unittest

from generate_local_scope_switch_vectors import OUTPUT
from local_scope_switch_model import validate


class LocalScopeSwitchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed_case(self, scenario: int, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(f"S {scenario}\n".encode("ascii"))
        self.assertIn(old, self.trace[start:].split(b"\nS ", 1)[0])
        return self.trace[:start] + self.trace[start:].replace(old, new, 1)

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_case_inventory(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"S 8\n")], self.trace + self.trace,
                      self.trace.replace(b"S 0\n", b"S 99\n", 1)):
            with self.subTest(trace=trace[:16]), self.assertRaises(ValueError):
                validate(trace)

    def test_missing_and_disjoint_defaults_do_not_join_private_names(self) -> None:
        for scenario in (0, 1):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"R 0 2 2 2 3", b"R 0 3 2 3 7"))

    def test_fallthrough_performs_cleanup_before_default_body(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(3, b"23:1 22:4", b"22:4 23:1"))

    def test_common_and_prior_outer_cells_survive(self) -> None:
        for scenario in (2, 5, 6):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"98", b"23:1 98"))

    def test_nested_continue_remains_a_loop_transfer(self) -> None:
        for old, new in ((b"B 1 4", b"B 2 4"), (b"Q 2", b"Q 1"),
                         (b"94 124", b"124 94"), (b"4 11 0", b"4 15 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(8, old, new))

    def test_nested_partial_break_removes_a_private_leaf_before_its_jump(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(10, b"23:1 98", b"98"))

    def test_aborting_nested_arms_do_not_publish_their_initialized_bitmap(self) -> None:
        for scenario in (9, 10):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, b"R 0 2 2 3 3", b"R 0 3 2 3 7"))

    def test_unreachable_native_cleanup_underflow_stays_explicit(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(9, b"23:255", b"23:1"))


if __name__ == "__main__":
    unittest.main()
