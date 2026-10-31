// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact source byte locations and line views, with no lexer/Unicode-width claims.
#![allow(
    clippy::expect_used,
    reason = "Tests use known valid source positions."
)]

use cod4x_script::source::{LineEnding, SourceBudget, SourceLocation, SourceText};

#[test]
fn mixed_endings_tabs_binary_bytes_and_eof_keep_exact_locations_and_borrows() {
    let bytes = b"a\r\n\n\t\xff\0b\r\nlast\r";
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    assert_eq!(source.bytes(), bytes);
    assert_eq!(source.line_count(), 4);
    let expected = [
        (0, b"a".as_slice(), b"a\r\n".as_slice(), LineEnding::CrLf),
        (3, b"".as_slice(), b"\n".as_slice(), LineEnding::Lf),
        (
            4,
            b"\t\xff\0b".as_slice(),
            b"\t\xff\0b\r\n".as_slice(),
            LineEnding::CrLf,
        ),
        (
            10,
            b"last\r".as_slice(),
            b"last\r".as_slice(),
            LineEnding::None,
        ),
    ];
    for (index, (offset, content, full, ending)) in expected.into_iter().enumerate() {
        let line = source.line(index as u64 + 1).expect("Line");
        assert_eq!(line.offset(), offset);
        assert_eq!(line.content(), content);
        assert_eq!(line.full_bytes(), full);
        assert_eq!(line.ending(), ending);
        assert_eq!(
            line.full_bytes().as_ptr(),
            source.bytes()[usize::try_from(offset).expect("Offset")..].as_ptr()
        );
    }
    for (offset, line, byte_column) in [
        (0, 1, 1),
        (1, 1, 2),
        (2, 1, 3),
        (3, 2, 1),
        (4, 3, 1),
        (6, 3, 3),
        (9, 3, 6),
        (10, 4, 1),
        (15, 4, 6),
    ] {
        assert_eq!(
            source.location(offset).expect("Location"),
            SourceLocation {
                offset,
                line,
                byte_column
            }
        );
    }
    assert_eq!(source.span(4..8).expect("Binary span"), b"\t\xff\0b");
    assert_eq!(source.span(15..15).expect("EOF span"), b"");
}

#[test]
fn empty_sources_trailing_lf_and_utf8_bytes_have_explicit_byte_columns() {
    for (bytes, lines) in [
        (b"".as_slice(), 1),
        (b"\n".as_slice(), 2),
        (b"x\n".as_slice(), 2),
    ] {
        let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
        assert_eq!(source.line_count(), lines);
        assert!(source.line(lines).expect("Final line").content().is_empty());
        assert_eq!(
            source.location(bytes.len() as u64).expect("EOF").line,
            lines
        );
    }
    let source = SourceText::index("é🙂\t".as_bytes(), SourceBudget::default()).expect("Raw UTF-8");
    assert_eq!(
        source
            .location(6)
            .expect("After six UTF-8 bytes")
            .byte_column,
        7
    );
    assert_eq!(
        source
            .location(1)
            .expect("Inside a multibyte sequence")
            .byte_column,
        2
    );
}

#[test]
fn independent_sources_and_wide_line_numbers_preserve_content_without_debug_disclosure() {
    let first = SourceText::index(b"private-script\n", SourceBudget::default()).expect("First");
    let bytes = b"line\n".repeat(70_000);
    let second = SourceText::index(&bytes, SourceBudget::default()).expect("Wide");
    assert_eq!(second.line_count(), 70_001);
    assert_eq!(
        second.location(bytes.len() as u64).expect("EOF").line,
        70_001
    );
    assert_eq!(second.line(70_000).expect("Wide line").content(), b"line");
    assert_eq!(
        first.line(1).expect("Original source").content(),
        b"private-script"
    );
    assert!(!format!("{first:?} {:?}", first.line(1).expect("Line")).contains("private"));
}
