// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests distinguish canonical virtual identity, owned payloads and host filesystem authority."
)]
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cod4x_script::binding::{LegacyScriptPath, ScriptPathBudget, ScriptPathErrorKind};

fn hash(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn canonical_identity_hash_and_owned_names_preserve_segment_quirks_without_source_borrows() {
    let budget = ScriptPathBudget::default();
    let first = LegacyScriptPath::new(b"\\//MAPS\\\\MP//UTIL", budget)
        .expect("First view")
        .value;
    let second = LegacyScriptPath::new(b"maps/mp/util", budget)
        .expect("Second view")
        .value;
    assert_eq!(first, second);
    assert_eq!(hash(first), hash(second));
    let copied = first.copy_owned(budget).expect("Owned canonical identity");
    assert_eq!(hash(&copied.value), hash(second));
    assert_eq!(copied.value.as_path(), second);
    assert_eq!(copied.usage.retained_bytes, 12);
    for (raw, expected) in [
        (b"a\n/b".as_slice(), b"ab".as_slice()),
        (b"a/\n/b", b"a/b"),
        (b"/../A/./B", b"../a/./b"),
        (b"  A B  ", b"  a b  "),
        (b"A.GSC/", b"a.gsc/"),
    ] {
        let view = LegacyScriptPath::new(raw, budget)
            .expect("Canonicalizer domain")
            .value;
        assert_eq!(view.canonical_bytes().collect::<Vec<_>>(), expected);
        assert_eq!(
            view.copy_owned(budget)
                .expect("Owned canonical name")
                .value
                .as_bytes(),
            expected
        );
    }
    let owned = {
        let input = b"\\PrivateModule\\PrivateFunction".to_vec();
        let view = LegacyScriptPath::new(&input, budget)
            .expect("Private source view")
            .value;
        assert!(!format!("{view:?} {:?}", view.canonical_bytes()).contains("Private"));
        view.copy_owned(budget)
            .expect("Owned before source drop")
            .value
    };
    assert_eq!(owned.as_bytes(), b"privatemodule/privatefunction");
    assert!(!format!("{owned:?}").contains("private"));
}

#[test]
fn seventy_thousand_bytes_survive_canonicalization_and_ownership_without_native_wrapper_ceilings() {
    let raw = vec![b'A'; 70_000];
    let budget = ScriptPathBudget::default();
    let view = LegacyScriptPath::new(&raw, budget).expect("Beyond 64/1024-byte native wrappers");
    assert_eq!(view.value.canonical_len(), 70_000);
    assert_eq!(view.usage.work_steps, 140_000);
    let owned = view
        .value
        .copy_owned(budget)
        .expect("Complete wide payload");
    assert_eq!(owned.value.as_bytes(), vec![b'a'; 70_000]);
    assert_eq!(owned.usage.retained_bytes, 70_000);
    assert_eq!(owned.usage.work_steps, 140_000);
}

#[test]
fn all_non_ascii_bytes_and_embedded_nul_reject_without_exposing_private_names() {
    for byte in 128..=255 {
        let mut raw = b"PrivateModule/".to_vec();
        raw.push(byte);
        let error = LegacyScriptPath::new(&raw, ScriptPathBudget::default())
            .expect_err("Non-ASCII ctype is outside this profile");
        assert_eq!(error.kind, ScriptPathErrorKind::NonAscii);
        assert!(!format!("{error:?} {error}").contains("Private"));
    }
    assert_eq!(
        LegacyScriptPath::new(b"valid\0different", ScriptPathBudget::default())
            .expect_err("No ambiguous prefix identity")
            .kind,
        ScriptPathErrorKind::EmbeddedNul
    );
}
