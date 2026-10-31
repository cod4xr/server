// SPDX-License-Identifier: AGPL-3.0-or-later
//! Empty/self operands, binary bytes, localized rejection, and private failures.
#![allow(
    clippy::expect_used,
    reason = "Tests own synthetic decoded string operands."
)]

mod string_support;

use cod4x_script::literal::{LegacyStringError, LegacyStringKind};

#[test]
fn empty_and_self_operands_preserve_exact_bytes_without_terminators_or_repair() -> Result<(), String>
{
    let empty = string_support::decoded(b"", LegacyStringKind::Plain)?;
    let result = empty.concatenate(&empty, 0).expect("No bytes required");
    assert!(result.as_bytes().is_empty());
    assert_eq!(result.kind(), LegacyStringKind::Plain);
    let bytes = b"private-secret\"\n\t\\\xff\x80";
    let value = string_support::decoded(bytes, LegacyStringKind::Plain)?;
    let result = value
        .concatenate(&value, bytes.len() as u64 * 2)
        .expect("Self concatenation");
    assert_eq!(result.as_bytes().len(), bytes.len() * 2);
    assert_eq!(&result.as_bytes()[..bytes.len()], bytes);
    assert_eq!(&result.as_bytes()[bytes.len()..], bytes);
    assert_eq!(
        empty
            .concatenate(&value, bytes.len() as u64)
            .expect("Empty prefix")
            .as_bytes(),
        bytes
    );
    assert_eq!(
        value
            .concatenate(&empty, bytes.len() as u64)
            .expect("Empty suffix")
            .as_bytes(),
        bytes
    );
    assert!(!format!("{result:?}").contains("private"));
    Ok(())
}

#[test]
fn localized_operands_fail_without_implicit_coercion_or_mutation() -> Result<(), String> {
    let plain = string_support::decoded(b"private-plain", LegacyStringKind::Plain)?;
    let localized = string_support::decoded(b"private-localized", LegacyStringKind::Localized)?;
    for (left, right) in [
        (&plain, &localized),
        (&localized, &plain),
        (&localized, &localized),
    ] {
        let error = left
            .concatenate(right, 1_000_000)
            .expect_err("Ordinary strings required");
        assert_eq!(error, LegacyStringError::ExpectedPlainString);
        assert!(!format!("{error:?} {error}").contains("private"));
    }
    assert_eq!(plain.as_bytes(), b"private-plain");
    assert_eq!(localized.as_bytes(), b"private-localized");
    Ok(())
}

#[test]
fn byte_budget_reports_actual_resource_counts_without_operand_contents() -> Result<(), String> {
    let value = string_support::decoded(b"private-secret", LegacyStringKind::Plain)?;
    let error = value.concatenate(&value, 0).expect_err("Zero budget");
    assert_eq!(
        error,
        LegacyStringError::BudgetExceeded {
            requested: 28,
            maximum: 0
        }
    );
    assert!(!format!("{error:?} {error}").contains("private"));
    assert_eq!(value.as_bytes(), b"private-secret");
    Ok(())
}
