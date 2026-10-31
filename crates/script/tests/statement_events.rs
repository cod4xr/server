// SPDX-License-Identifier: AGPL-3.0-or-later
//! Specialized event/profile roles grant no scheduling or debugger authority.
#![allow(
    clippy::expect_used,
    reason = "Tests supply valid synthetic event/profile statements."
)]

use cod4x_script::expression::{ExpressionListBudget, LegacyExpressionListSyntax};
use cod4x_script::statement::{LegacyEventKind, LegacyProfileKind, LegacyStatementKind};
use cod4x_script::syntax::{LegacySyntaxMode, LegacySyntaxNodeKind};

use crate::expression_support;
use crate::statement_support;

#[test]
fn event_families_preserve_distinct_opaque_argument_profiles_and_receiver() -> Result<(), String> {
    for (text, expected, rule) in [
        (
            "self waittill(\"PrivateEvent\",out)",
            LegacyEventKind::WaitTill,
            119,
        ),
        (
            "self waittill(\"PrivateEvent\")",
            LegacyEventKind::WaitTill,
            120,
        ),
        (
            "self waittillmatch(\"PrivateEvent\",1)",
            LegacyEventKind::WaitTillMatch,
            121,
        ),
        (
            "self waittillmatch(\"PrivateEvent\")",
            LegacyEventKind::WaitTillMatch,
            122,
        ),
        (
            "self notify(\"PrivateEvent\",1)",
            LegacyEventKind::Notify,
            123,
        ),
        (
            "self notify(\"PrivateEvent\")",
            LegacyEventKind::Notify,
            124,
        ),
    ] {
        let text = format!("main(){{{text};}}");
        let tree = expression_support::parse(text.as_bytes(), LegacySyntaxMode::Script);
        let view = statement_support::first(&tree);
        let LegacyStatementKind::Event {
            kind,
            receiver,
            arguments,
        } = view.kind()
        else {
            return Err("Expected event statement".into());
        };
        assert_eq!(kind, expected);
        assert_eq!(expression_support::bytes(&tree, receiver), b"self");
        assert!(
            matches!(arguments.kind(), LegacySyntaxNodeKind::Production(value) if value.as_u16() == rule)
        );
        assert!(
            LegacyExpressionListSyntax::lower(arguments, ExpressionListBudget::default()).is_err()
        );
        assert!(!format!("{view:?}").contains("PrivateEvent"));
    }
    let tree = expression_support::parse(
        b"main(){self endon(\"PrivateEvent\");}",
        LegacySyntaxMode::Script,
    );
    let LegacyStatementKind::EndOn { receiver, event } = statement_support::first(&tree).kind()
    else {
        return Err("Expected endon".into());
    };
    assert_eq!(expression_support::bytes(&tree, receiver), b"self");
    assert_eq!(expression_support::bytes(&tree, event), b"\"PrivateEvent\"");
    Ok(())
}

#[test]
fn wait_frame_break_continue_debugger_and_raw_profile_labels_are_only_syntax() -> Result<(), String>
{
    for text in ["waittillframeend", "break", "continue", "breakpoint"] {
        let source = format!("main(){{{text};}}");
        let tree = expression_support::parse(source.as_bytes(), LegacySyntaxMode::Script);
        assert!(matches!(
            (text, statement_support::first(&tree).kind()),
            ("waittillframeend", LegacyStatementKind::WaitTillFrameEnd)
                | ("break", LegacyStatementKind::Break)
                | ("continue", LegacyStatementKind::Continue)
                | ("breakpoint", LegacyStatementKind::Breakpoint)
        ));
    }
    for (text, expected) in [
        ("prof_begin", LegacyProfileKind::Begin),
        ("prof_end", LegacyProfileKind::End),
    ] {
        let source = format!("main(){{{text}(\"PrivateLabel\");}}");
        let tree = expression_support::parse(source.as_bytes(), LegacySyntaxMode::Script);
        let view = statement_support::first(&tree);
        let LegacyStatementKind::Profile { kind, label } = view.kind() else {
            return Err("Expected profile statement".into());
        };
        assert_eq!(kind, expected);
        assert_eq!(label.as_bytes(), b"\"PrivateLabel\"");
        assert!(!format!("{view:?}").contains("PrivateLabel"));
    }
    Ok(())
}
