# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected developer mode/validation/cleanup expectations reject damaged traces."""
import re
import unittest

from generate_local_scope_developer_vectors import OUTPUT
from local_scope_developer_model import validate


class LocalScopeDeveloperTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed_case(self, header: bytes, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(header + b"\n")
        boundary = re.search(rb"\n(?:C|F) ", self.trace[start:])
        end = start + boundary.start() if boundary else len(self.trace)
        case = self.trace[start:end]
        self.assertIn(old, case)
        return self.trace[:start] + case.replace(old, new, 1) + self.trace[end:]

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_complete_unique_bounded_inventory(self) -> None:
        for trace in (b"", self.trace[:self.trace.index(b"C 4 0 0\n")], self.trace + self.trace,
                      self.trace.replace(b"C 0 0 0", b"C 9 0 0", 1)):
            with self.subTest(trace=trace[:16]), self.assertRaises(ValueError):
                validate(trace)

    def test_disabled_body_still_validates_in_discard_mode(self) -> None:
        for old, new in ((b"M 2", b"M"), (b"M 2", b"M 1"), (b"O 22", b"O")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(b"C 1 0 0", old, new))

    def test_disabled_body_rewinds_only_its_output_and_restores_globals(self) -> None:
        for old, new in ((b"W 170", b"W"), (b"W 170", b"W 170 22 2 0"),
                         (b"G 0 123 0 1", b"G 2 999 0 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(b"C 1 0 0", old, new))

    def test_enabled_private_slot_cleanup_does_not_export_the_name(self) -> None:
        for old, new in ((b"B 0 2 2 3 7", b"B 0 3 3 3 7"),
                         (b"O 22 23", b"O 22"), (b"23 1", b"23 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed_case(b"C 1 1 0", old, new))

    def test_later_planned_parent_name_does_not_keep_a_private_created_slot(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed_case(b"C 3 1 0", b"B 0 1 1 3 3", b"B 0 2 2 3 3"))

    def test_existing_public_slot_and_modeled_aborting_body_have_no_private_removal(self) -> None:
        for header in (b"C 2 1 0", b"C 4 1 0"):
            with self.subTest(header=header), self.assertRaises(ValueError):
                validate(self.changed_case(header, b"O\nW", b"O 23\nW"))

    def test_nested_developer_modes_report_before_body_or_global_changes(self) -> None:
        for nesting in (1, 2):
            header = f"C 1 1 {nesting}".encode("ascii")
            for old, new in ((b"M\nO", b"M 1\nO"),
                             (f"G {nesting} 123 1 0".encode("ascii"), b"G 0 123 0 0")):
                with self.subTest(nesting=nesting, old=old), self.assertRaises(ValueError):
                    validate(self.changed_case(header, old, new))

    def test_discarded_local_and_external_function_names_have_no_lookup_or_output(self) -> None:
        for header, old, new in ((b"F 0 0", b"G 2 100 0 0 0", b"G 2 100 1 0 0"),
                                 (b"F 1 0", b"W 170", b"W 170 0")):
            with self.subTest(header=header), self.assertRaises(ValueError):
                validate(self.changed_case(header, old, new))

    def test_discarded_name_ownership_and_file_pending_count_stay_separate(self) -> None:
        for header, old, new in ((b"F 0 0", b"T 2", b"T"), (b"F 1 0", b"T 2 3", b"T 2"),
                                 (b"F 1 1", b"G 2 99", b"G 2 100"), (b"F 0 1", b"T\nW", b"T 2\nW")):
            with self.subTest(header=header), self.assertRaises(ValueError):
                validate(self.changed_case(header, old, new))


if __name__ == "__main__":
    unittest.main()
