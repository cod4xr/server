# SPDX-License-Identifier: AGPL-3.0-or-later
"""Disposable registration-policy fixtures; no Cargo compilation or target cache."""
from pathlib import Path
import tempfile
import unittest

import script_test_targets


MANIFEST = '''[package]
name = "suite-policy-fixture"
autotests = false
[[test]]
name = "frontend"
path = "tests/frontend.rs"
[[test]]
name = "legacy"
path = "tests/legacy.rs"
'''


class RegistrationTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="cod4x-suite-policy-")
        self.root = Path(temporary.name).resolve()
        self.assertTrue(self.root.is_relative_to(Path(tempfile.gettempdir()).resolve()))
        self.assertTrue(self.root.name.startswith("cod4x-suite-policy-"))
        self.addCleanup(temporary.cleanup)
        self.directory = self.root / "crates/script"
        self.tests = self.directory / "tests"
        self.tests.mkdir(parents=True)
        (self.directory / "Cargo.toml").write_text(MANIFEST, encoding="utf-8")
        self.write("alpha.rs", "#[test]\nfn alpha() {}\n")
        self.write("beta.rs", "#[test]\nfn beta() {}\n")
        self.write("frontend.rs", '#[path = "alpha.rs"]\nmod alpha;\n')
        self.write("legacy.rs", '#[path = "beta.rs"]\nmod beta;\n')

    def write(self, filename: str, text: str) -> None:
        path = self.tests / filename
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def valid(self) -> None:
        self.assertEqual(script_test_targets.validate(self.root), [])

    def invalid(self) -> None:
        self.assertTrue(script_test_targets.validate(self.root))

    def test_explicit_same_name_leaf_paths_remain_valid(self) -> None:
        self.valid()

    def test_standard_modules_and_separate_helper_folders_remain_complete(self) -> None:
        self.write("support/mod.rs", "pub fn helper() {}\n")
        self.write("frontend.rs", "// Example: #[cfg(test)] is not an attribute here.\nmod alpha;\nmod support;\n")
        self.write("legacy.rs", "mod beta; // separately owned leaf\n")
        self.valid()

    def test_nested_leaf_requires_and_accepts_explicit_matching_path(self) -> None:
        self.write("nested/gamma.rs", "#[test]\nfn gamma() {}\n")
        self.invalid()
        self.write("frontend.rs", 'mod alpha;\n#[path = "nested/gamma.rs"]\nmod gamma;\n')
        self.valid()

    def test_missing_suite_is_rejected(self) -> None:
        (self.tests / "frontend.rs").unlink()
        self.invalid()

    def test_missing_leaf_and_extra_non_test_leaf_are_rejected(self) -> None:
        self.write("frontend.rs", "mod absent;\n")
        self.invalid()
        self.write("empty.rs", "pub fn helper() {}\n")
        self.write("frontend.rs", "mod alpha;\nmod empty;\n")
        self.invalid()

    def test_duplicate_declaration_within_or_across_suites_is_rejected(self) -> None:
        for legacy, frontend in [("mod beta;", "mod alpha;\nmod alpha;"),
                                 ("mod beta;\nmod alpha;", "mod alpha;")]:
            self.write("frontend.rs", frontend)
            self.write("legacy.rs", legacy)
            self.invalid()

    def test_cfg_ignore_macro_and_inline_test_or_code_cannot_hide_registration(self) -> None:
        for prefix in ('#[cfg(any())]', '#[cfg_attr(test, ignore)]', '#[ignore]',
                       'include!("alpha.rs");', '#[test]', 'fn unexpected() {}'):
            with self.subTest(prefix=prefix):
                self.write("frontend.rs", prefix + "\nmod alpha;\n")
                self.invalid()

    def test_same_module_name_from_different_paths_is_rejected(self) -> None:
        self.write("a/case.rs", "#[test]\nfn first() {}\n")
        self.write("b/case.rs", "#[test]\nfn second() {}\n")
        self.write("frontend.rs", 'mod alpha;\n#[path = "a/case.rs"]\nmod case;\n'
                   '#[path = "b/case.rs"]\nmod case;\n')
        self.invalid()

    def test_ambiguous_module_and_tests_hidden_in_helper_modules_are_rejected(self) -> None:
        self.write("alpha/mod.rs", "pub fn helper() {}\n")
        self.write("frontend.rs", "mod alpha;\n")
        self.invalid()
        self.write("support/mod.rs", "#[test]\nfn hidden() {}\n")
        self.write("frontend.rs", '#[path = "alpha.rs"]\nmod alpha;\nmod support;\n')
        self.invalid()

    def test_renamed_absolute_traversal_and_noncanonical_paths_are_rejected_on_all_hosts(self) -> None:
        for filename, module in [("alpha.rs", "renamed"), ("../alpha.rs", "alpha"),
                                 ("/alpha.rs", "alpha"), ("C:/alpha.rs", "alpha"),
                                 ("nested\\alpha.rs", "alpha"), ("./alpha.rs", "alpha"),
                                 ("nested/../alpha.rs", "alpha"), ("alpha.txt", "alpha")]:
            with self.subTest(filename=filename):
                self.write("frontend.rs", f'#[path = "{filename}"]\nmod {module};\n')
                self.invalid()

    def test_dangling_and_repeated_path_attributes_are_rejected(self) -> None:
        for source in ('mod alpha;\n#[path = "alpha.rs"]\n',
                       '#[path = "alpha.rs"]\n#[path = "alpha.rs"]\nmod alpha;\n'):
            self.write("frontend.rs", source)
            self.invalid()

    def test_auto_discovery_extra_or_duplicate_cargo_targets_are_rejected(self) -> None:
        for manifest in (MANIFEST.replace("autotests = false", "autotests = true"),
                         MANIFEST + '[[test]]\nname = "extra"\npath = "tests/extra.rs"\n',
                         MANIFEST + '[[test]]\nname = "frontend"\npath = "tests/frontend.rs"\n'):
            (self.directory / "Cargo.toml").write_text(manifest, encoding="utf-8")
            self.invalid()


if __name__ == "__main__":
    unittest.main()
