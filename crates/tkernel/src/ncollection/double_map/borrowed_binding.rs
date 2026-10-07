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
        Err(duplicate_pair(first, second))
    }
    /// Checks both keys before invoking either copy; `assert!(map.try_bind_ref(&1, &2))`.
    pub fn try_bind_ref(&mut self, first: &A, second: &B) -> bool {
        let Some((one, two)) = self.prepare_binding(first, second) else {
            return false;
        };
        self.storage
            .insert((one, two), || first.clone(), || second.clone());
        true
    }
}
impl<A: Debug + Clone, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Copies Key1 after allocation while transferring Key2; `map.bind_key1_ref(&1,5)?`.
    pub fn bind_key1_ref(&mut self, first: &A, second: B) -> Result<(), Failure> {
        let Some((one, two)) = self.prepare_binding(first, &second) else {
            return Err(duplicate_pair(first, &second));
        };
        self.storage.insert((one, two), || first.clone(), || second);
        Ok(())
    }
    /// Copies only the first key on successful insertion; `map.try_bind_key1_ref(&1,5)`.
    pub fn try_bind_key1_ref(&mut self, first: &A, second: B) -> bool {
        let Some((one, two)) = self.prepare_binding(first, &second) else {
            return false;
        };
        self.storage.insert((one, two), || first.clone(), || second);
        true
    }
}
impl<A: Debug, B: Debug + Clone, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Transfers Key1 while copying Key2 into its allocated base field; `map.bind_key2_ref(1,&5)?`.
    pub fn bind_key2_ref(&mut self, first: A, second: &B) -> Result<(), Failure> {
        let Some((one, two)) = self.prepare_binding(&first, second) else {
            return Err(duplicate_pair(&first, second));
        };
        self.storage.insert((one, two), || first, || second.clone());
        Ok(())
    }
    /// Copies only the second key on successful insertion; `map.try_bind_key2_ref(1,&5)`.
    pub fn try_bind_key2_ref(&mut self, first: A, second: &B) -> bool {
        let Some((one, two)) = self.prepare_binding(&first, second) else {
            return false;
        };
        self.storage.insert((one, two), || first, || second.clone());
        true
    }
}
pub(super) fn duplicate_pair<A: Debug, B: Debug>(first: &A, second: &B) -> Failure {
    Failure::with_type(
        "Standard_MultiplyDefined",
        &format!("DoubleMap pair ({first:?}, {second:?}); expected two unbound keys"),
        "",
    )
}
