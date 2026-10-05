//! OCCT 3d097a0 constructs a policy before transfer, but assigns it after transfer.
use super::FlatTable;

impl<K, V, H> FlatTable<K, V, H> {
    pub fn move_from_with(source: &mut Self, take_policy: impl FnOnce(&mut H) -> H) -> Self {
        let hasher: H = take_policy(&mut source.hasher);
        Self {
            slots: std::mem::take(&mut source.slots),
            count: std::mem::replace(&mut source.count, 0),
            hasher,
        }
    }
    pub fn move_assign_with(
        &mut self,
        source: &mut Self,
        assign_policy: impl FnOnce(&mut H, &mut H),
    ) {
        self.clear(true);
        self.slots = std::mem::take(&mut source.slots);
        self.count = std::mem::replace(&mut source.count, 0);
        assign_policy(&mut self.hasher, &mut source.hasher);
    }
}
