# SPDX-License-Identifier: AGPL-3.0-or-later
"""Keep selectable pinned assembly and modeled global boundaries fail closed."""
from types import SimpleNamespace
import unittest

from local_scope_reference import assembly

SOURCE = (
    "extern service\nSECTION .text\nselected:\n mov eax, [state]\n call service\n ret\n"
    ";other()\nother:\n ret\nSECTION .data\nSECTION .bss\nstate: resb 0x4\n"
)


class NativeBoundaryTests(unittest.TestCase):
    def generate(self, source: str = SOURCE) -> str:
        reference = SimpleNamespace(files={"vm.asm": source})
        return assembly(reference, ("selected",), {"service", "state"},
                        source_name="vm.asm", modeled_globals=frozenset({"state"}))

    def test_selected_zero_initialized_storage_is_explicitly_modeled(self) -> None:
        generated = self.generate()
        self.assertIn("extern state\n", generated)
        self.assertIn("global selected\n", generated)
        self.assertNotIn("state:", generated)
        self.assertNotIn("other:", generated)

    def test_new_calls_change_the_explicit_service_boundary(self) -> None:
        with self.assertRaises(ValueError):
            self.generate(SOURCE.replace("call service", "call service\n call unknown"))

    def test_modeled_global_must_be_used_and_have_a_native_bss_declaration(self) -> None:
        for source in (SOURCE.replace("[state]", "[eax]"), SOURCE.replace("state: resb 0x4", "state: dd 0"),
                       SOURCE.replace("state: resb 0x4", "another: resb 0x4")):
            with self.subTest(source=source), self.assertRaises(ValueError):
                self.generate(source)

    def test_unmodeled_extra_service_is_rejected(self) -> None:
        reference = SimpleNamespace(files={"vm.asm": SOURCE})
        with self.assertRaises(ValueError):
            assembly(reference, ("selected",), {"service", "state", "unused"},
                     source_name="vm.asm", modeled_globals=frozenset({"state"}))

    def test_default_compiler_selection_stays_unchanged(self) -> None:
        source = SOURCE.replace("[state]", "[eax]")
        reference = SimpleNamespace(files={"src/asmsource/_scr_compiler.asm": source})
        generated = assembly(reference, ("selected",), {"service"})
        self.assertIn("extern service\n", generated)
        self.assertNotIn("extern state\n", generated)


if __name__ == "__main__":
    unittest.main()
