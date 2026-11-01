# SPDX-License-Identifier: AGPL-3.0-or-later
"""Developer planning evidence must keep private names and loop transfer joins separate."""
import unittest

from generate_local_scope_developer_analysis_vectors import OUTPUT
from local_scope_developer_analysis_model import validate


class LocalScopeDeveloperAnalysisTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed_case(self, scenario: int, later: int, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(f"D {scenario} {later}\n".encode("ascii"))
        self.assertIn(old, self.trace[start:].split(b"\nD ", 1)[0])
        return self.trace[:start] + self.trace[start:].replace(old, new, 1)

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_case_inventory_and_rows_are_bounded(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"D 9 1\n")], self.trace + self.trace,
                      self.trace.replace(b"D 0 0", b"D 10 0", 1), self.trace + b"A 8192\n"):
            with self.subTest(trace=trace[:16]), self.assertRaises(ValueError):
                validate(trace)

    def test_private_and_common_developer_names_do_not_export_to_the_function_parent(self) -> None:
        for scenario in (0, 1, 2, 3, 5):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, 0, b"R 0 2 2 2 3", b"R 0 2 2 3 3"))

    def test_later_direct_declaration_does_not_retroactively_change_developer_public_prefix(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(0, 1, b"B 0 2 2 3 3", b"B 0 2 3 3 3"))

    def test_common_and_disjoint_branch_children_remain_distinct(self) -> None:
        for scenario, old, new in ((2, b"B 0 2 3 3 3", b"B 0 2 2 3 3"),
                                   (3, b"n:1 n:3 n:4", b"n:1 n:3 n:2")):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, 0, old, new))

    def test_return_and_nested_planning_do_not_invent_parent_aborts(self) -> None:
        for scenario in (4, 5):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, 0, b"R 0 2", b"R 3 2"))

    def test_dynamic_break_does_not_export_names_but_continue_unions_do(self) -> None:
        for scenario, old, new in ((6, b"R 0 2 2 2 3", b"R 0 2 2 3 3"),
                                   (7, b"R 0 2 2 3 3", b"R 0 2 2 2 3"),
                                   (8, b"R 0 2 2 4 3", b"R 0 2 2 5 3"),
                                   (9, b"R 0 2 2 5 3", b"R 0 2 2 4 3")):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed_case(scenario, 0, old, new))

    def test_for_initializer_body_and_update_keep_their_distinct_name_order(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(9, 0, b"n:5 n:2 n:4", b"n:5 n:4 n:2"))


if __name__ == "__main__":
    unittest.main()
