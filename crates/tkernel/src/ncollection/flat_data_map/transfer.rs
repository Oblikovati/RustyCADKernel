//! Explicit copy and injected move policies preserve custom hasher state.
use super::{Debug, FlatDataMap, HashPolicy};

impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Moves an injected policy before reserving without Default; `FlatDataMap::with_hasher_move_from(&mut policy, 10, take_policy)`.
    pub fn with_hasher_move_from(
        source: &mut H,
        hint: usize,
        take_policy: impl FnOnce(&mut H) -> H,
    ) -> Self {
        Self::with_hasher(take_policy(source), hint)
    }
    /// Transfers slots after moving the policy without requiring Default; `FlatDataMap::move_from_with(&mut source, take_policy)`.
    pub fn move_from_with(source: &mut Self, take_policy: impl FnOnce(&mut H) -> H) -> Self {
        Self {
            table: super::FlatTable::move_from_with(&mut source.table, take_policy),
        }
    }
    /// Clears then transfers slots before assigning the policy, as OCCT 3d097a0; `map.move_assign_with(&mut source, assign_policy)`.
    pub fn move_assign_with(
        &mut self,
        source: &mut Self,
        assign_policy: impl FnOnce(&mut H, &mut H),
    ) -> &mut Self {
        self.table
            .move_assign_with(&mut source.table, assign_policy);
        self
    }
    /// Preserves native self-move without aliased mutable borrows; `map.move_assign_self()`.
    pub fn move_assign_self(&mut self) -> &mut Self {
        self
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Clone> FlatDataMap<K, V, H> {
    /// Copies a borrowed policy before reserving slots; `FlatDataMap::with_hasher_ref(&policy, 10)`.
    pub fn with_hasher_ref(hasher: &H, hint: usize) -> Self {
        Self::with_hasher(hasher.clone(), hint)
    }
}
impl<K: Debug, V, H: HashPolicy<K> + Default> FlatDataMap<K, V, H> {
    /// Returns the destination after transferring storage and replacing the donor policy; `map.move_assign_returning(&mut source)`.
    pub fn move_assign_returning(&mut self, source: &mut Self) -> &mut Self {
        self.move_assign_with(source, |target, donor| *target = std::mem::take(donor))
    }
}
