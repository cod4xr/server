# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reject altered native clock-phase observations and incomplete inventories."""
import re
import unittest

from generate_scheduler_phase_vectors import OUTPUT
from scheduler_phase_model import validate


class SchedulerPhaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed(self, header: bytes, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(header + b"\n")
        boundary = re.search(rb"\nT ", self.trace[start:])
        end = start + boundary.start() if boundary else len(self.trace)
        case = self.trace[start:end]
        self.assertIn(old, case)
        return self.trace[:start] + case.replace(old, new, 1) + self.trace[end:]

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_complete_unique_bounded_inventory(self) -> None:
        for trace in (b"", self.trace + self.trace, self.trace.replace(b"T 4 1 2", b"T 5 1 2"),
                      self.trace[:self.trace.index(b"T 4 1 2\n")], b"B 7\n" + self.trace):
            with self.subTest(trace=trace[:20]), self.assertRaises(ValueError):
                validate(trace)

    def test_disabled_scripts_still_free_entities_and_advance_time(self) -> None:
        for old, new in ((b"F 7", b""), (b"G 8 100 0 0 0 1 1", b"G 7 100 0 0 0 1 1"),
                         (b"F 7", b"B 7\nR 7\nF 7")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed(b"T 0 0 1", old, new))

    def test_run_without_increment_leaves_time_unchanged(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed(b"T 3 0 0", b"G 7 101", b"G 8 101"))

    def test_resume_precedes_bucket_remove_and_entity_cleanup(self) -> None:
        for old, new in ((b"R 7\nD 7", b"D 7\nR 7"),
                         (b"D 7\nB 7\nF 7", b"D 7\nF 7\nB 7"), (b"F 7", b"F 8")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed(b"T 3 0 1", old, new))

    def test_empty_bucket_and_missing_time_array_do_not_resume(self) -> None:
        for header in (b"T 1 0 0", b"T 2 0 0"):
            with self.subTest(header=header), self.assertRaises(ValueError):
                validate(self.changed(header, b"B 7", b"B 7\nR 7"))

    def test_second_run_observes_bucket_already_removed(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed(b"T 3 0 2", b"B 7\nL 7\nB 7\nF 7",
                                  b"B 7\nL 7\nR 7\nD 7\nB 7\nF 7"))

    def test_legacy_wrap_occurs_after_old_time_cleanup(self) -> None:
        for old, new in ((b"F 16777215", b"F 0"),
                         (b"G 0 101 2 1 0 1 0", b"G 16777216 101 2 1 0 1 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed(b"T 3 1 1", old, new))

    def test_logging_and_elapsed_counters_are_observable(self) -> None:
        for old, new in ((b"O 7\nR 7", b"R 7\nO 7"),
                         (b"G 7 101 2 1 1 0 0", b"G 7 100 2 1 0 0 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed(b"T 4 0 0", old, new))


if __name__ == "__main__":
    unittest.main()
