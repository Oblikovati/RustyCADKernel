//! The const-reference Bind/TryBind overloads copy only after duplicate validation.
use super::{Debug, DoubleMap, HashPolicy};
use crate::standard::Failure;

impl<A: Debug + Clone, B: Debug + Clone, H1: HashPolicy<A>, H2: HashPolicy<B>>
    DoubleMap<A, B, H1, H2>
{
    /// Copies both keys only when the pair can be bound; `map.bind_ref(&first, &second)?`.
    pub fn bind_ref(&mut self, first: &A, second: &B) -> Result<(), Failure> {
        if self.try_bind_ref(first, second) {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_MultiplyDefined",
            &format!("DoubleMap pair ({first:?}, {second:?}); expected two unbound keys"),
            "",
        ))
    }
    /// Checks both keys before invoking either copy; `assert!(map.try_bind_ref(&1, &2))`.
    pub fn try_bind_ref(&mut self, first: &A, second: &B) -> bool {
        self.prepare_insert();
        if self.is_bound1(first) || self.is_bound2(second) {
            return false;
        }
        // Native DoubleMapNode constructs its base key2 before its key1 member.
        let second_copy = second.clone();
        let first_copy = first.clone();
        self.insert_pair(first_copy, second_copy);
        true
    }
}
