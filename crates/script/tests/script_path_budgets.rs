// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exhaust virtual-name validation/copy work and exact input/output limits."
)]
use cod4x_script::binding::{
    LegacyScriptPath, ScriptPathBudget, ScriptPathErrorKind, ScriptPathResource,
};

#[test]
fn canonical_views_and_owned_copies_preflight_every_insufficient_work_prefix() {
    let raw = b"\\//PRIVATE\\\\MODULE/\n//Leaf";
    let measured = LegacyScriptPath::new(raw, ScriptPathBudget::default()).expect("Measured view");
    for maximum in 0..=measured.usage.work_steps {
        let result = LegacyScriptPath::new(
            raw,
            ScriptPathBudget {
                max_steps: maximum,
                ..ScriptPathBudget::default()
            },
        );
        if maximum < measured.usage.work_steps {
            let error = result.expect_err("Every incomplete view prefix");
            assert!(matches!(
                error.kind,
                ScriptPathErrorKind::BudgetExceeded {
                    resource: ScriptPathResource::WorkSteps,
                    ..
                }
            ));
            assert!(error.usage.work_steps <= maximum);
            assert_eq!(error.usage.retained_bytes, 0);
        } else {
            assert_eq!(result.expect("Exact work").usage, measured.usage);
        }
    }
    let copy = measured
        .value
        .copy_owned(ScriptPathBudget::default())
        .expect("Measured copy");
    for maximum in 0..=copy.usage.work_steps {
        let result = measured.value.copy_owned(ScriptPathBudget {
            max_steps: maximum,
            ..ScriptPathBudget::default()
        });
        if maximum < copy.usage.work_steps {
            let error = result.expect_err("Every incomplete owned-copy prefix");
            assert!(matches!(
                error.kind,
                ScriptPathErrorKind::BudgetExceeded {
                    resource: ScriptPathResource::WorkSteps,
                    ..
                }
            ));
            assert!(error.usage.work_steps <= maximum);
            assert_eq!(error.usage.retained_bytes, 0);
        } else {
            assert_eq!(result.expect("Exact copy work").usage, copy.usage);
        }
    }
}

#[test]
fn exact_input_and_canonical_payload_limits_and_zero_byte_names_are_admitted_separately() {
    let raw = b"//A\\B";
    let exact = ScriptPathBudget {
        max_input_bytes: raw.len() as u64,
        max_canonical_bytes: 3,
        max_steps: 2 * raw.len() as u64,
    };
    let view = LegacyScriptPath::new(raw, exact).expect("Exact view budgets");
    assert_eq!(
        view.value
            .copy_owned(exact)
            .expect("Exact owned budgets")
            .value
            .as_bytes(),
        b"a/b"
    );
    for (resource, limits) in [
        (
            ScriptPathResource::InputBytes,
            ScriptPathBudget {
                max_input_bytes: raw.len() as u64 - 1,
                ..exact
            },
        ),
        (
            ScriptPathResource::CanonicalBytes,
            ScriptPathBudget {
                max_canonical_bytes: 2,
                ..exact
            },
        ),
    ] {
        for error in [
            LegacyScriptPath::new(raw, limits).expect_err("One less view resource"),
            view.value
                .copy_owned(limits)
                .expect_err("One less owned resource"),
        ] {
            assert!(
                matches!(error.kind, ScriptPathErrorKind::BudgetExceeded {resource: actual, requested, maximum} if actual == resource && requested > maximum)
            );
            assert_eq!(error.usage.retained_bytes, 0);
        }
    }
    let zero = ScriptPathBudget {
        max_input_bytes: 0,
        max_canonical_bytes: 0,
        max_steps: 0,
    };
    let empty = LegacyScriptPath::new(b"", zero).expect("Empty virtual identifier domain");
    assert_eq!(
        empty
            .value
            .copy_owned(zero)
            .expect("No payload allocation")
            .value
            .as_bytes(),
        b""
    );
    assert_eq!(empty.usage.work_steps, 0);
}
