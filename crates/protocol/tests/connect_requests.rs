// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy connect framing, typed field domains, preservation, and disclosure tests.
#![allow(
    clippy::expect_used,
    reason = "Tests exercise explicit connect wire inputs."
)]

use cod4x_protocol::CommandLine;
use cod4x_protocol::connection::{ConnectError, ConnectField, ConnectRequest};

#[test]
fn tokenization_and_typed_fields_match_original_c_lookup_and_decimal_conversion() {
    let mut count = 0;
    for row in include_str!("fixtures/wire_vectors.txt")
        .lines()
        .filter_map(|row| row.strip_prefix("connect "))
    {
        let fields: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(fields.len(), 4);
        let source: Vec<u8> = fields[0]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
            })
            .collect();
        let request = ConnectRequest::parse(CommandLine::parse(&source).expect("Line"))
            .expect("Original valid connect fields");
        assert_eq!(
            request.protocol(),
            fields[1].parse::<i32>().expect("Protocol")
        );
        assert_eq!(
            request.challenge(),
            fields[2].parse::<i32>().expect("Challenge")
        );
        assert_eq!(request.qport(), fields[3].parse::<u16>().expect("Qport"));
        count += 1;
    }
    assert_eq!(count, 12);
}

#[test]
fn legacy_userinfo_preserves_unknown_fields_and_signed_short_qport_spelling() {
    for (spelling, expected) in [
        ("0", 0),
        ("32767", 32767),
        ("32768", 32768),
        ("65535", 65535),
        ("-1", 65535),
        ("-32768", 32768),
        (" +00042 ", 42),
    ] {
        let bytes = format!(
            "CoNnEcT \"\\protocol\\21\\challenge\\-2147483648\\qport\\{spelling}\\xver\\21.2\\future\\value\" ignored"
        );
        let line = CommandLine::parse(bytes.as_bytes()).expect("Command");
        let request = ConnectRequest::parse(line).expect("Valid connect fields");
        assert_eq!(request.protocol(), 21);
        assert_eq!(request.challenge(), i32::MIN);
        assert_eq!(request.qport(), expected);
        assert_eq!(request.userinfo().get(b"future"), Some(b"value".as_slice()));
        assert_eq!(request.userinfo().get(b"xver"), Some(b"21.2".as_slice()));
    }
}

#[test]
fn malformed_overflowing_and_ambiguous_security_fields_are_rejected() {
    for (info, error) in [
        (
            "\\protocol\\-1\\challenge\\0\\qport\\1",
            ConnectError::InvalidField(ConnectField::Protocol),
        ),
        (
            "\\protocol\\2147483648\\challenge\\0\\qport\\1",
            ConnectError::InvalidField(ConnectField::Protocol),
        ),
        (
            "\\protocol\\21junk\\challenge\\0\\qport\\1",
            ConnectError::InvalidField(ConnectField::Protocol),
        ),
        (
            "\\protocol\\21\\challenge\\2147483648\\qport\\1",
            ConnectError::InvalidField(ConnectField::Challenge),
        ),
        (
            "\\protocol\\21\\challenge\\-2147483649\\qport\\1",
            ConnectError::InvalidField(ConnectField::Challenge),
        ),
        (
            "\\protocol\\21\\challenge\\0\\qport\\65536",
            ConnectError::InvalidField(ConnectField::Qport),
        ),
        (
            "\\protocol\\21\\challenge\\0\\qport\\-32769",
            ConnectError::InvalidField(ConnectField::Qport),
        ),
        (
            "\\protocol\\21\\challenge\\0",
            ConnectError::MissingField(ConnectField::Qport),
        ),
        (
            "\\protocol\\21\\challenge\\0\\qport\\1\\Protocol\\6",
            ConnectError::DuplicateField(ConnectField::Protocol),
        ),
        (
            "\\protocol\\21\\challenge\\0\\qport\\1\\CHALLENGE\\1",
            ConnectError::DuplicateField(ConnectField::Challenge),
        ),
        (
            "\\protocol\\21\\challenge\\0\\qport\\1\\QPORT\\2",
            ConnectError::DuplicateField(ConnectField::Qport),
        ),
    ] {
        let bytes = format!("connect \"{info}\"");
        let line = CommandLine::parse(bytes.as_bytes()).expect("Line");
        assert_eq!(ConnectRequest::parse(line).err(), Some(error));
    }
}

#[test]
fn credentials_and_challenge_values_are_excluded_from_request_debug_and_errors() {
    let source = b"connect \"\\protocol\\21\\challenge\\195487653\\qport\\13\\password\\do-not-log-secret\\name\\Player\xff\"";
    let request =
        ConnectRequest::parse(CommandLine::parse(source).expect("Line")).expect("Request");
    assert_eq!(
        request.userinfo().get(b"password"),
        Some(b"do-not-log-secret".as_slice())
    );
    assert_eq!(
        request.userinfo().get(b"name"),
        Some(b"Player\xff".as_slice())
    );
    let debug = format!("{request:?}");
    for private in ["do-not-log-secret", "195487653", "Player"] {
        assert!(!debug.contains(private));
    }
    assert_eq!(
        ConnectRequest::parse(CommandLine::parse(b"status").expect("Line")).err(),
        Some(ConnectError::UnexpectedCommand)
    );
    assert_eq!(
        ConnectRequest::parse(CommandLine::parse(b"connect").expect("Line")).err(),
        Some(ConnectError::MissingUserinfo)
    );
}
