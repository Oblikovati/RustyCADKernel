//! A pending resize owns its replacement table until committed or dropped.
use super::{MapLinkNode, bucket_table::BucketTable};
use crate::{ncollection::next_map_prime, standard::Failure};

/// Result of native `beginResize`, retaining the computed modulus even on a no-op.
/// Example: `let plan = storage.begin_resize(100)?; assert_eq!(plan.bucket_count(), 101);`.
/// Dropping an uncommitted replacement releases only its table, never its nodes.
pub struct BaseMapResize {
    pub(super) bucket_count: usize,
    pub(super) replacement: Option<BucketTable>,
}

impl BaseMapResize {
    pub(super) fn prepare(
        extent: usize,
        current: usize,
        allocated: bool,
        tables: usize,
    ) -> Result<Self, Failure> {
        let bucket_count = next_map_prime(extent);
        if bucket_count <= current && allocated {
            return Ok(Self {
                bucket_count,
                replacement: None,
            });
        }
        let bucket_count = next_resize_count(extent, current, allocated);
        let count = table_slot_count(bucket_count, tables)?;
        Ok(Self {
            bucket_count,
            replacement: Some(BucketTable::zeroed(count)?),
        })
    }
    /// The native output modulus, including no-op requests; `plan.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    /// Whether `beginResize` allocated a replacement; `assert!(plan.is_resize())`.
    pub fn is_resize(&self) -> bool {
        self.replacement.is_some()
    }
    /// Borrows replacement pointer slots, or an empty slice on a no-op; `plan.bucket_slots()`.
    pub fn bucket_slots(&self) -> &[*mut MapLinkNode] {
        self.replacement.as_ref().map_or(&[], BucketTable::as_slice)
    }
    /// Allows relinking nodes before commit; `plan.bucket_slots_mut()[0] = node`.
    pub fn bucket_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        self.replacement
            .as_mut()
            .map_or(&mut [], BucketTable::as_mut_slice)
    }
    /// Returns pending raw `Buckets`, null for a no-op; `plan.buckets().is_null()`.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.replacement
            .as_ref()
            .map_or(std::ptr::null(), |table| table.as_ptr().cast_const())
    }
    /// Returns writable pending `Buckets`, null for a no-op; `plan.buckets_mut()`.
    pub fn buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.replacement
            .as_ref()
            .map_or(std::ptr::null_mut(), BucketTable::as_ptr)
    }
}

pub(super) fn table_slot_count(buckets: usize, tables: usize) -> Result<usize, Failure> {
    buckets.checked_add(1).and_then(|count| count.checked_mul(tables)).ok_or_else(|| {
        Failure::with_message("Standard_OutOfMemory", &format!(
            "BaseMap modulus {buckets} with {tables} tables overflowed; expected addressable bucket storage"
        ))
    })
}

pub(super) fn next_resize_count(extent: usize, current: usize, allocated: bool) -> usize {
    let requested = next_map_prime(extent);
    if requested <= current && !allocated {
        current
    } else {
        requested
    }
}
