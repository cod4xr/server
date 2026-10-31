// SPDX-License-Identifier: AGPL-3.0-or-later
// Token boundaries follow Cmd_TokenizeStringInternal in pinned cmd.c.
use std::iter::FusedIterator;

/// Borrowed tokens with upstream whitespace, quote, and comment semantics.
///
/// Quotes delimit tokens without escape processing. Unterminated quotes consume
/// the remainder. Line comments and unterminated block comments end iteration.
#[derive(Clone, Debug)]
pub struct CommandTokens<'line> {
    pub(super) remaining: &'line [u8],
}

impl<'line> Iterator for CommandTokens<'line> {
    type Item = &'line [u8];

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let skipped = self
                .remaining
                .iter()
                .take_while(|&&byte| byte <= b' ')
                .count();
            self.remaining = &self.remaining[skipped..];
            if self.remaining.starts_with(b"//") {
                self.remaining = &[];
            }
            if self.remaining.is_empty() {
                return None;
            }
            if self.remaining.starts_with(b"/*") {
                if let Some(end) = self.remaining.windows(2).position(|pair| pair == b"*/") {
                    self.remaining = &self.remaining[end + 2..];
                    continue;
                }
                self.remaining = &[];
                return None;
            }
            break;
        }
        if let Some(quoted) = self.remaining.strip_prefix(b"\"") {
            let end = quoted
                .iter()
                .position(|&byte| byte == b'"')
                .unwrap_or(quoted.len());
            self.remaining = &quoted[(end + 1).min(quoted.len())..];
            return Some(&quoted[..end]);
        }
        let mut end = 0;
        while end < self.remaining.len() {
            let rest = &self.remaining[end..];
            if rest[0] <= b' '
                || rest[0] == b'"'
                || rest.starts_with(b"//")
                || rest.starts_with(b"/*")
            {
                break;
            }
            end += 1;
        }
        let token = &self.remaining[..end];
        self.remaining = &self.remaining[end..];
        Some(token)
    }
}

impl FusedIterator for CommandTokens<'_> {}
