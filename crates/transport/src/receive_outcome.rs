// SPDX-License-Identifier: AGPL-3.0-or-later
/// A channel drop category without received bytes or peer identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReceiveDrop {
    /// Sequence was already delivered or belongs to an older partial message.
    StaleSequence,
    /// A fragment does not begin at the next required offset.
    OutOfOrderFragment,
    /// The caller's reassembly storage cannot hold this complete operation.
    Capacity,
    /// The packet direction or qport differs from the established channel.
    WrongRoute,
}

/// A fully received logical message, borrowing its packet or reassembly storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceivedMessage<'message> {
    pub(crate) sequence: u32,
    pub(crate) skipped_sequences: u32,
    pub(crate) body: &'message [u8],
}

impl<'message> ReceivedMessage<'message> {
    /// Returns the delivered sequence, without the fragment flag.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Returns sequence numbers skipped since the previous delivered message.
    /// This is not a count of UDP datagrams or lost individual fragments.
    #[must_use]
    pub fn skipped_sequences(&self) -> u32 {
        self.skipped_sequences
    }

    /// Returns the validated, reassembled body without channel headers.
    #[must_use]
    pub fn body(&self) -> &'message [u8] {
        self.body
    }
}

/// A complete message, accepted partial fragment, or dropped envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReceiveOutcome<'message> {
    /// A full fragment was retained; no logical message is ready yet.
    Pending,
    /// A datagram was rejected without changing channel state or storage.
    Dropped(ReceiveDrop),
    /// A complete message is ready. Its borrow keeps packet/reassembly data alive.
    Message(ReceivedMessage<'message>),
}
