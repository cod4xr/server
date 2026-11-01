// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::heap::objects::ObjectHeapErrorKind;
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::FieldSlot;
use crate::heap::objects::object_work::ObjectWork;

#[derive(Clone, Copy)]
pub(super) enum Bucket {
    Empty,
    Deleted,
    Occupied(usize),
}

#[derive(Default)]
pub(super) struct FieldIndex {
    pub(super) buckets: Vec<Bucket>,
    pub(super) live: usize,
    pub(super) filled: usize,
}

pub(super) struct Lookup {
    pub(super) field: Option<usize>,
    pub(super) bucket: Option<usize>,
}

impl FieldIndex {
    pub(super) fn lookup(
        &self,
        fields: &[FieldSlot],
        owner: usize,
        hash: u64,
        key: &[u8],
        work: &mut ObjectWork,
    ) -> Result<Lookup, ObjectHeapErrorKind> {
        self.lookup_key(fields, owner, hash, FieldKey::Bytes(key), work)
    }

    pub(super) fn lookup_key(
        &self,
        fields: &[FieldSlot],
        owner: usize,
        hash: u64,
        key: FieldKey<'_>,
        work: &mut ObjectWork,
    ) -> Result<Lookup, ObjectHeapErrorKind> {
        let key_len = key.len()?;
        if self.buckets.is_empty() {
            return Ok(Lookup {
                field: None,
                bucket: None,
            });
        }
        let mut position = self.start(hash)?;
        let mut deleted = None;
        for _ in 0..self.buckets.len() {
            work.charge(1)?;
            match self.buckets[position] {
                Bucket::Empty => {
                    return Ok(Lookup {
                        field: None,
                        bucket: Some(deleted.unwrap_or(position)),
                    });
                }
                Bucket::Deleted => {
                    deleted.get_or_insert(position);
                }
                Bucket::Occupied(index) => {
                    let Some(FieldSlot::Occupied(field)) = fields.get(index) else {
                        return Err(ObjectHeapErrorKind::InvalidState);
                    };
                    if field.owner == owner && field.hash == hash && field.key.len() == key_len {
                        work.charge(key_len as u64)?;
                        if key.matches(&field.key) {
                            return Ok(Lookup {
                                field: Some(index),
                                bucket: Some(position),
                            });
                        }
                    }
                }
            }
            position = self.next(position);
        }
        deleted
            .map(|position| Lookup {
                field: None,
                bucket: Some(position),
            })
            .ok_or(ObjectHeapErrorKind::InvalidState)
    }

    pub(super) fn target_length(&self) -> Result<usize, ObjectHeapErrorKind> {
        if self.buckets.is_empty() {
            return Ok(8);
        }
        let filled = self
            .filled
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        if filled <= self.buckets.len() / 2 {
            return Ok(self.buckets.len());
        }
        if self.live < self.buckets.len() / 2 {
            return Ok(self.buckets.len());
        }
        self.buckets
            .len()
            .checked_mul(2)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)
    }

    pub(super) fn needs_rebuild(&self) -> bool {
        self.buckets.is_empty() || self.filled >= self.buckets.len() / 2
    }

    pub(super) fn empty_position(
        &self,
        hash: u64,
        work: &mut ObjectWork,
    ) -> Result<usize, ObjectHeapErrorKind> {
        let mut position = self.start(hash)?;
        for _ in 0..self.buckets.len() {
            work.charge(1)?;
            if matches!(self.buckets[position], Bucket::Empty) {
                return Ok(position);
            }
            position = self.next(position);
        }
        Err(ObjectHeapErrorKind::InvalidState)
    }

    pub(super) fn insert(
        &mut self,
        position: usize,
        field: usize,
    ) -> Result<(), ObjectHeapErrorKind> {
        let bucket = self
            .buckets
            .get_mut(position)
            .ok_or(ObjectHeapErrorKind::InvalidState)?;
        let extra = match bucket {
            Bucket::Empty => 1,
            Bucket::Deleted => 0,
            Bucket::Occupied(_) => return Err(ObjectHeapErrorKind::InvalidState),
        };
        let filled = self
            .filled
            .checked_add(extra)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        let live = self
            .live
            .checked_add(1)
            .ok_or(ObjectHeapErrorKind::SizeOverflow)?;
        *bucket = Bucket::Occupied(field);
        self.filled = filled;
        self.live = live;
        Ok(())
    }

    pub(super) fn start(&self, hash: u64) -> Result<usize, ObjectHeapErrorKind> {
        if !self.buckets.len().is_power_of_two() {
            return Err(ObjectHeapErrorKind::InvalidState);
        }
        usize::try_from(hash & (self.buckets.len() as u64 - 1))
            .map_err(|_| ObjectHeapErrorKind::SizeOverflow)
    }

    fn next(&self, position: usize) -> usize {
        (position + 1) & (self.buckets.len() - 1)
    }
}
