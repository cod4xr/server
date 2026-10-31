// SPDX-License-Identifier: AGPL-3.0-or-later
// Seed and key evolution follow original SV_Netchan_Encode/Decode; see NOTICE.
use super::ObfuscationError;
use crate::{MAX_COMMAND_BYTES, MAX_MESSAGE_BYTES};

/// Applies the server-to-client transform to bytes after the plaintext header.
///
/// `sequence` is the message's reserved channel sequence, not the subsequent one.
/// The acknowledged client command must be the exact legacy-cleaned command bytes.
/// Only low seed bits participate, matching the original byte-sized XOR key.
///
/// # Errors
/// Rejects oversized payload/command slices before changing any payload byte.
pub fn apply_server_payload(
    payload: &mut [u8],
    challenge: u32,
    sequence: u32,
    last_client_command: &[u8],
) -> Result<(), ObfuscationError> {
    apply(
        payload,
        (challenge ^ sequence).to_le_bytes()[0],
        last_client_command,
    )
}

/// Applies the client-to-server transform to bytes after its plaintext header.
///
/// Header size is profile-specific: do not assume all `CoD4X` versions use only
/// the three original acknowledgement fields. Select and validate that profile
/// before calling this function. The command is the last acknowledged server
/// command, not arbitrary current player input.
///
/// # Errors
/// Rejects oversized payload/command slices before changing any payload byte.
pub fn apply_client_payload(
    payload: &mut [u8],
    challenge: u32,
    server_id: u32,
    message_acknowledge: u32,
    last_server_command: &[u8],
) -> Result<(), ObfuscationError> {
    let seed = (challenge ^ server_id ^ message_acknowledge).to_le_bytes()[0];
    apply(payload, seed, last_server_command)
}

fn apply(payload: &mut [u8], mut key: u8, command: &[u8]) -> Result<(), ObfuscationError> {
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(ObfuscationError::MessageTooLarge);
    }
    let length = command
        .iter()
        .take(MAX_COMMAND_BYTES + 1)
        .position(|&byte| byte == 0)
        .unwrap_or(command.len());
    if length > MAX_COMMAND_BYTES {
        return Err(ObfuscationError::CommandTooLong);
    }
    let command = &command[..length];
    let mut command_index = 0;
    for (position, byte) in payload.iter_mut().enumerate() {
        let value = command.get(command_index).copied().unwrap_or(0);
        let shift = u32::from(!position.is_multiple_of(2));
        key ^= value.wrapping_shl(shift);
        *byte ^= key;
        command_index += 1;
        if command_index >= command.len() {
            command_index = 0;
        }
    }
    Ok(())
}
