// SPDX-License-Identifier: AGPL-3.0-or-later
//! Atomic compression failures, reserved symbols, and expansion budgets.

use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4x_protocol::compression::{self, CompressionError};

#[test]
fn insufficient_output_never_receives_partial_compression_or_decompression() {
    for capacity in 0..2 {
        let mut output = vec![0xa5; capacity];
        assert_eq!(
            compression::compress(b"g", &mut output),
            Err(CompressionError::OutputCapacity)
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
    }
    // Original C decodes 0x01 as zero followed by a final padded symbol six.
    let mut output = [0xa5; 1];
    assert_eq!(
        compression::decompress(&[1], &mut output),
        Err(CompressionError::OutputCapacity)
    );
    assert_eq!(output, [0xa5]);
}

#[test]
fn reserved_nyt_symbol_is_rejected_without_output_mutation() {
    let mut output = [0xa5; 32];
    assert_eq!(
        compression::decompress(&[0xbd, 0x01], &mut output),
        Err(CompressionError::ReservedSymbol)
    );
    assert_eq!(output, [0xa5; 32]);
}

#[test]
fn input_and_expansion_limits_are_wire_profile_budgets() {
    let oversized = vec![0; MAX_MESSAGE_BYTES + 1];
    let mut output = [0xa5; 8];
    assert_eq!(
        compression::compress(&oversized, &mut output),
        Err(CompressionError::MessageTooLarge)
    );
    assert_eq!(
        compression::decompress(&oversized, &mut output),
        Err(CompressionError::MessageTooLarge)
    );
    assert_eq!(output, [0xa5; 8]);
    // All-zero codes repeatedly expand to symbol six, eventually exhausting the
    // logical output budget even with a larger caller-provided allocation.
    let compressed = vec![0; MAX_MESSAGE_BYTES];
    let mut large_output = vec![0xa5; MAX_MESSAGE_BYTES + 1];
    assert_eq!(
        compression::decompress(&compressed, &mut large_output),
        Err(CompressionError::MessageTooLarge)
    );
    assert!(large_output.iter().all(|&byte| byte == 0xa5));
}

#[test]
fn empty_streams_need_no_storage_and_do_not_touch_unused_bytes() {
    assert_eq!(compression::compress(&[], &mut []), Ok(0));
    assert_eq!(compression::decompress(&[], &mut []), Ok(0));
    let mut output = [0xa5; 4];
    assert_eq!(compression::compress(&[], &mut output), Ok(0));
    assert_eq!(compression::decompress(&[], &mut output), Ok(0));
    assert_eq!(output, [0xa5; 4]);
}
