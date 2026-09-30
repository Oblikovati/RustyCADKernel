//! The friend access of `Storage_Schema` to `Standard_Persistent::_typenum` and `_refnum`.
use crate::standard::StandardPersistent;
use std::sync::atomic::Ordering;

/// Plain reads and writes of the two storage counters of a persistent object.
pub(crate) trait PersistentNumbers {
    /// `_typenum`.
    fn type_number(&self) -> i32;
    /// `_typenum = value`.
    fn set_type_number(&self, value: i32);
    /// `_refnum`.
    fn reference_number(&self) -> i32;
    /// `_refnum = value`.
    fn set_reference_number(&self, value: i32);
}

impl PersistentNumbers for StandardPersistent {
    fn type_number(&self) -> i32 {
        self.type_num().load(Ordering::Relaxed)
    }

    fn set_type_number(&self, value: i32) {
        self.type_num().store(value, Ordering::Relaxed);
    }

    fn reference_number(&self) -> i32 {
        self.ref_num().load(Ordering::Relaxed)
    }

    fn set_reference_number(&self, value: i32) {
        self.ref_num().store(value, Ordering::Relaxed);
    }
}
