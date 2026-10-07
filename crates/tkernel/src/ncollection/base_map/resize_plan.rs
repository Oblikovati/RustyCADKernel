//! The native `beginResize` decision and table sizing shared by every hashed map.
use crate::{ncollection::next_map_prime, standard::Failure};

/// Heads in `tables` tables of `buckets + 1` heads, or Standard_OutOfMemory on overflow.
pub(in crate::ncollection) fn table_slot_count(
    buckets: usize,
    tables: usize,
) -> Result<usize, Failure> {
    buckets.checked_add(1).and_then(|count| count.checked_mul(tables)).ok_or_else(|| {
        Failure::with_message("Standard_OutOfMemory", &format!(
            "BaseMap modulus {buckets} with {tables} tables overflowed; expected addressable bucket storage"
        ))
    })
}

/// Native `beginResize` decision: `None` when the allocated table already has enough buckets,
/// otherwise the modulus of the replacement table (the current one when no table exists yet).
pub(in crate::ncollection) fn planned_bucket_count(
    extent: usize,
    current: usize,
    allocated: bool,
) -> Option<usize> {
    let requested = next_map_prime(extent);
    if requested <= current && allocated {
        return None;
    }
    if requested <= current {
        return Some(current);
    }
    Some(requested)
}

#[cfg(test)]
mod tests {
    use super::{planned_bucket_count, table_slot_count};

    #[test]
    fn allocated_tables_grow_only_past_the_current_prime() {
        assert_eq!(planned_bucket_count(0, 101, true), None);
        assert_eq!(planned_bucket_count(200, 101, true), Some(1009));
    }

    #[test]
    fn a_missing_table_keeps_a_larger_requested_modulus() {
        assert_eq!(planned_bucket_count(0, 13, false), Some(101));
        assert_eq!(planned_bucket_count(0, 2003, false), Some(2003));
    }

    #[test]
    fn table_size_overflow_is_a_named_failure() {
        assert_eq!(table_slot_count(101, 2).ok(), Some(204));
        assert!(table_slot_count(usize::MAX, 1).is_err());
    }
}
