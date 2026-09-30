//! The `std::hash` / `std::equal_to` specializations for `occ::handle<TCollection_HExtendedString>`.
use super::HExtendedString;
use crate::{ncollection::HashPolicy, standard::Handle};

/// Hashes a handle by its string (0 for a null handle) and equates handles that share an
/// object or hold equal strings; `DataMap::with_hasher(HExtendedStringHasher, 1)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HExtendedStringHasher;
impl HashPolicy<Handle<HExtendedString>> for HExtendedStringHasher {
    fn hash(&self, key: &Handle<HExtendedString>) -> usize {
        key.get().map_or(0, |object| object.string().hash_code())
    }
    fn equal(&self, first: &Handle<HExtendedString>, second: &Handle<HExtendedString>) -> bool {
        if first == second {
            return true;
        }
        match (first.get(), second.get()) {
            (Some(first), Some(second)) => first.is_same_state(second),
            _ => false,
        }
    }
}
