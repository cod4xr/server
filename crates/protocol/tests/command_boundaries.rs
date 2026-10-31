// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded command-line extraction and binary-tail preservation.
#![allow(
    clippy::expect_used,
    reason = "Test setup uses known command-line bounds."
)]

use cod4x_protocol::{CommandLine, MAX_COMMAND_BYTES};

#[test]
fn connectionless_normalization_stops_before_binary_tail() {
    let mut output = [0xa5; 64];
    let body = b"getinfo ab%cd\n\0\xffsecret";
    let line = CommandLine::read_connectionless(body, &mut output).expect("Valid line");
    assert_eq!(line.as_bytes(), b"getinfo ab.cd");
    assert_eq!(&body[line.consumed_bytes()..], b"\0\xffsecret");
    assert_eq!(line.tokens().count(), 2);
    assert_eq!(
        CommandLine::parse(b"a%b").expect("Direct line").as_bytes(),
        b"a%b"
    );
}

#[test]
fn command_extraction_is_atomic_and_bounded_before_copying() {
    let mut output = [0xa5; 2];
    assert!(CommandLine::read_connectionless(b"long", &mut output).is_err());
    assert_eq!(output, [0xa5; 2]);
    let oversized = vec![b'a'; MAX_COMMAND_BYTES + 1];
    assert!(CommandLine::read_connectionless(&oversized, &mut output).is_err());
    assert_eq!(output, [0xa5; 2]);
    let mut exact = vec![b'a'; MAX_COMMAND_BYTES];
    exact.push(b'\n');
    exact.extend_from_slice(&oversized);
    let line = CommandLine::parse(&exact).expect("Exact line with ignored tail");
    assert_eq!(line.as_bytes().len(), MAX_COMMAND_BYTES);
    assert_eq!(line.consumed_bytes(), MAX_COMMAND_BYTES + 1);
}

#[test]
fn token_iteration_is_fused_for_all_terminating_comment_cases() {
    for input in [b"//".as_slice(), b"/*", b"/* hidden */", b" \t", b""] {
        let line = CommandLine::parse(input).expect("Short line");
        let mut tokens = line.tokens();
        assert_eq!(tokens.next(), None);
        assert_eq!(tokens.next(), None);
    }
}
