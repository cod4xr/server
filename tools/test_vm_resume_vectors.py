# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reject altered selected native resume ordering and resource-service traces."""
import re
import unittest

from generate_vm_resume_vectors import OUTPUT
from vm_resume_model import validate


class VmResumeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.trace = b"\n".join(row for row in OUTPUT.read_bytes().splitlines()
                                 if not row.startswith(b"#")) + b"\n"

    def changed(self, scenario: int, old: bytes, new: bytes) -> bytes:
        start = self.trace.index(f"R {scenario}\n".encode("ascii"))
        boundary = re.search(rb"\nR ", self.trace[start:])
        end = start + boundary.start() if boundary else len(self.trace)
        case = self.trace[start:end]
        self.assertIn(old, case)
        return self.trace[:start] + case.replace(old, new, 1) + self.trace[end:]

    def test_complete_trace(self) -> None:
        validate(self.trace)

    def test_complete_unique_bounded_inventory(self) -> None:
        for trace in (b"", self.trace + self.trace, self.trace.replace(b"R 9", b"R 10"),
                      self.trace[:self.trace.index(b"R 9\n")], b"P 1\n" + self.trace):
            with self.subTest(trace=trace[:20]), self.assertRaises(ValueError):
                validate(trace)

    def test_dequeue_clear_free_precede_execution(self) -> None:
        for old, new in ((b"P 1\nW 1", b"W 1\nP 1"), (b"W 1\nF 1", b"F 1\nW 1"),
                         (b"S 1 1000 1 0 0\nE 1", b"E 1\nS 1 1000 1 0 0")):
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate(self.changed(1, old, new))

    def test_requeued_work_is_not_lost_to_a_snapshot(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed(3, b"P 1\nW 1\nF 1\nS 1 1000 1 0 0\nE 1\n", b""))

    def test_front_and_tail_modeled_insertions_are_distinct(self) -> None:
        for scenario, first, second in ((4, 3, 2), (5, 2, 3)):
            old = f"E 1\nP {first}\nW {first}\nF {first}".encode("ascii")
            new = old.replace(str(first).encode("ascii"), b"9")
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed(scenario, old, new))

    def test_scalar_archive_restoration_has_exact_words_types_and_depth(self) -> None:
        old = b"S 1 1000 1 0 3 40 6 41 6 42 0"
        for new in (b"S 1 1001 1 0 3 40 6 41 6 42 0", b"S 1 1000 2 0 3 40 6 41 6 42 0",
                    b"S 1 1000 1 1 3 40 6 41 6 42 0", b"S 1 1000 1 0 2 40 6 41 6",
                    b"S 1 1000 1 0 3 40 6 41 6 42 6", b"S 1 1000 1 0 3 40 6 42 6 42 0"):
            with self.subTest(new=new), self.assertRaises(ValueError):
                validate(self.changed(8, old, new))

    def test_bucket_clears_only_after_all_bodies(self) -> None:
        with self.assertRaises(ValueError):
            validate(self.changed(2, b"E 3\nC 88", b"C 88\nE 3"))

    def test_each_archive_and_result_requests_one_release(self) -> None:
        for new in (b"G 3 1 2 3 3 3 3 1", b"G 3 1 3 2 3 3 3 1",
                    b"G 3 1 3 3 2 3 3 1", b"G 3 1 3 3 3 2 3 1", b"G 3 1 3 3 3 3 2 1"):
            with self.subTest(new=new), self.assertRaises(ValueError):
                validate(self.changed(2, b"G 3 1 3 3 3 3 3 1", new))

    def test_timestamp_mismatch_refreshes_watchdog(self) -> None:
        for scenario, old, new in ((8, b"G 1 1 1 1 1 1 1 1", b"G 1 2 1 1 1 1 1 2"),
                                   (9, b"G 1 2 1 1 1 1 1 2", b"G 1 1 1 1 1 1 1 1")):
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate(self.changed(scenario, old, new))


if __name__ == "__main__":
    unittest.main()
