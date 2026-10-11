//! Shared native object copy/assignment and hashed-key callback fixture.
use super::allocation_trace::{Payload, record_key_phase};
use rustycad_tkernel::ncollection::HashKey;
#[repr(transparent)]
#[derive(Debug, Default)]
/// Native object/key callbacks; `Key(Payload(1))`.
pub struct Key(
    /// Observable object identity.
    pub Payload<'O'>,
);
impl Clone for Key {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        record_key_phase('Q', self.0.0);
        self.0 == other.0
    }
}
impl HashKey for Key {
    fn occt_hash(&self) -> usize {
        record_key_phase('H', self.0.0);
        (self.0.0 % 2).occt_hash()
    }
}
