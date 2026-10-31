// SPDX-License-Identifier: AGPL-3.0-or-later
/// Decoded command in the current legacy peer profile, not engine-owned state.
///
/// Angles preserve unsigned 16-bit turns. Buttons use 21 bits; weapon/offhand
/// slots use seven. Wider future engine identifiers need explicit peer mapping.
/// Commands are untrusted player input and must not be attached to runtime logs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyUserCommand {
    /// Client command timestamp, with legacy signed 32-bit wrapping arithmetic.
    pub server_time: i32,
    /// Legacy input button bits, not server authorization.
    pub buttons: u32,
    /// Pitch, yaw, and roll encoded as unsigned turns scaled by 65536.
    pub angles: [u16; 3],
    /// Legacy weapon slot, not an internal asset handle.
    pub weapon: u8,
    /// Legacy offhand slot, not an internal asset handle.
    pub offhand: u8,
    /// Forward axis. Changed movement masks decode to -127, 0, or 127.
    pub forward_move: i8,
    /// Right axis. Changed movement masks decode to -127, 0, or 127.
    pub right_move: i8,
    /// Legacy melee charge yaw in degrees; unchanged fields retain their bits.
    pub melee_charge_yaw: f32,
    /// Legacy melee charge distance byte.
    pub melee_charge_distance: u8,
    /// Legacy selected-location bytes, present only for the corresponding button.
    pub selected_location: [u8; 2],
}
