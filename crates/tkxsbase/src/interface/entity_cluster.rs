// Rust port of DataExchange/TKXSBase/Interface/Interface_EntityCluster (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_EntityCluster`: up to four entities and a link to the next cluster, the
//! storage of `Interface_EntityList` beyond one entity.
use super::InterfaceEntityIterator;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
};

/// The fields of a cluster.
#[derive(Default)]
struct ClusterState {
    theents: [Handle<StandardTransient>; 4],
    thenext: Handle<InterfaceEntityCluster>,
}

/// `Interface_EntityCluster`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceEntityCluster;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let cluster = InterfaceEntityCluster::new();
/// for _ in 0..6 {
///     cluster.append(&Handle::new(StandardTransient::default())).unwrap();
/// }
/// assert_eq!(cluster.nb_entities(), 6);
/// ```
#[derive(Default)]
pub struct InterfaceEntityCluster {
    transient: StandardTransient,
    state: SharedState<ClusterState>,
}
impl_transient!(
    InterfaceEntityCluster,
    "Interface_EntityCluster",
    StandardTransient,
    transient
);

fn null_entity(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "Interface_EntityCluster {member}: null entity; expected a live entity"
    ))
}

fn out_of_range(member: &str, num: i32) -> Failure {
    exceptions::out_of_range(&format!(
        "Interface EntityCluster : {member}: rank {num} is out of the cluster"
    ))
}

impl InterfaceEntityCluster {
    /// `Interface_EntityCluster()`: empty.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_EntityCluster(ent)`: holds `ent`.
    pub fn with_entity(ent: &Handle<StandardTransient>) -> Self {
        Self::with_entity_and_next(ent, &Handle::default())
    }
    /// `Interface_EntityCluster(ec)`: empty, followed by `ec`.
    pub fn with_next(ec: &Handle<InterfaceEntityCluster>) -> Self {
        Self::with_entity_and_next(&Handle::default(), ec)
    }
    /// `Interface_EntityCluster(ent, ec)`.
    pub fn with_entity_and_next(
        ent: &Handle<StandardTransient>,
        ec: &Handle<InterfaceEntityCluster>,
    ) -> Self {
        let cluster = Self::default();
        let mut state = cluster.state.write();
        state.theents[0] = ent.clone();
        state.thenext = ec.clone();
        drop(state);
        cluster
    }
    /// `Append(ent)`: in the first free place of this cluster, else in the first cluster
    /// of the chain that is not full (or the last one); `Standard_NullObject` for null.
    pub fn append(&self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("Append"));
        }
        let next = {
            let mut state = self.state.write();
            if let Some(slot) = state.theents.iter_mut().find(|e| e.is_null()) {
                *slot = ent.clone();
                return Ok(());
            }
            if state.thenext.is_null() {
                state.thenext = Handle::new(Self::with_entity(ent));
                return Ok(());
            }
            state.thenext.clone()
        };
        let mut current = next;
        while current.has_next() && current.is_local_full() {
            current = current.next();
        }
        current.append(ent)
    }
    /// `Remove(ent)`: `Standard_NullObject` for null; false when not found here.
    /// Native quirks kept: a removal in a next cluster returns false (and unlinks that
    /// cluster when it became empty); see `remove_rank` for a local removal.
    pub fn remove(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        if ent.is_null() {
            return Err(null_entity("Remove"));
        }
        let (local, next) = {
            let state = self.state.read();
            let local = state.theents.iter().position(|e| e == ent);
            (local, state.thenext.clone())
        };
        if let Some(index) = local {
            return self.remove_rank(index as i32 + 1);
        }
        if next.is_null() {
            return Ok(false);
        }
        if next.remove(ent)? {
            self.state.write().thenext = next.next();
        }
        Ok(false)
    }
    /// `Remove(num)`: `Standard_OutOfRange` below 1 or beyond the chain; true when this
    /// cluster had a single entity. Native quirks kept: only the fourth place is cleared
    /// (the last entity is removed only when the cluster is full), a removal in a next
    /// cluster returns false; and the native shift loop runs downwards out of the array
    /// for a rank below the local count (memory corruption natively), which the port
    /// reports as `Standard_OutOfRange`.
    pub fn remove_rank(&self, num: i32) -> Result<bool, Failure> {
        if num < 1 {
            return Err(out_of_range("Remove", num));
        }
        let n = self.nb_local();
        if num > n {
            let next = self.next();
            if next.is_null() {
                return Err(out_of_range("Remove", num));
            }
            if next.remove_rank(num - n)? {
                self.state.write().thenext = next.next();
            }
            return Ok(false);
        }
        if num < n {
            return Err(exceptions::out_of_range(&format!(
                "Interface EntityCluster : Remove: rank {num} below the local count {n}; the native shift loop leaves the array"
            )));
        }
        self.state.write().theents[3].nullify();
        Ok(n == 1)
    }
    /// `NbEntities`: in this cluster and the next ones.
    pub fn nb_entities(&self) -> i32 {
        let mut nb = self.nb_local();
        let mut current = self.next();
        while let Some(cluster) = current.get() {
            nb += cluster.nb_local();
            current = cluster.next();
        }
        nb
    }
    /// `Value(num)`: `Standard_OutOfRange` out of the chain. Native quirk kept: the rank
    /// is reduced by the local count of this cluster at each step of the chain.
    pub fn value(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        let (cluster, local) = self.locate(num, "Value")?;
        match cluster {
            Some(other) => Ok(other.local_value(local)),
            None => Ok(self.local_value(local)),
        }
    }
    /// `SetValue(num, ent)`: `Standard_NullObject` for null, `Standard_OutOfRange` out of
    /// the chain.
    pub fn set_value(&self, num: i32, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("SetValue"));
        }
        let (cluster, local) = self.locate(num, "SetValue")?;
        let target = cluster.as_ref().map_or(&self.state, |c| &c.state);
        target.write().theents[(local - 1) as usize] = ent.clone();
        Ok(())
    }
    /// `FillIterator(iter)`: the entities of the chain, in order.
    pub fn fill_iterator(&self, iter: &mut InterfaceEntityIterator) {
        let mut current: Option<Handle<InterfaceEntityCluster>> = None;
        loop {
            let (ents, next) = {
                let state = current.as_ref().map_or(&self.state, |c| &c.state).read();
                (state.theents.clone(), state.thenext.clone())
            };
            ents.iter()
                .filter(|e| e.is_some())
                .for_each(|e| iter.get_one_item(e));
            if next.is_null() {
                return;
            }
            current = Some(next);
        }
    }
    /// `IsLocalFull` (private, used by `Interface_EntityList`): the fourth place is set.
    pub(super) fn is_local_full(&self) -> bool {
        self.state.read().theents[3].is_some()
    }
    /// `NbLocal` (private): the rank of the last set place.
    pub(super) fn nb_local(&self) -> i32 {
        let state = self.state.read();
        state
            .theents
            .iter()
            .rposition(|e| e.is_some())
            .map_or(0, |i| i as i32 + 1)
    }
    /// `HasNext` (private).
    pub(super) fn has_next(&self) -> bool {
        self.state.read().thenext.is_some()
    }
    /// `Next` (private).
    pub(super) fn next(&self) -> Handle<InterfaceEntityCluster> {
        self.state.read().thenext.clone()
    }
    fn local_value(&self, local: i32) -> Handle<StandardTransient> {
        self.state.read().theents[(local - 1) as usize].clone()
    }
    /// The cluster (None for this one) and local rank of `num`, with the native walk.
    fn locate(
        &self,
        num: i32,
        member: &str,
    ) -> Result<(Option<Handle<InterfaceEntityCluster>>, i32), Failure> {
        let nb = self.nb_local();
        if num <= 0 {
            return Err(out_of_range(member, num));
        }
        if num <= nb {
            return Ok((None, num));
        }
        let mut current = self.next();
        let mut local = num - nb;
        loop {
            let cluster = current.get().ok_or_else(|| out_of_range(member, num))?;
            if local <= cluster.nb_local() {
                return Ok((Some(current.clone()), local));
            }
            let next = cluster.next();
            current = next;
            local -= nb;
        }
    }
}

impl Drop for InterfaceEntityCluster {
    /// `~Interface_EntityCluster`: unlinks the chain iteratively, so a long chain does not
    /// recurse (the native destructor does the same).
    fn drop(&mut self) {
        let mut next = std::mem::take(&mut self.state.write().thenext);
        while let Some(cluster) = next.get() {
            let following = std::mem::take(&mut cluster.state.write().thenext);
            next = following;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity() -> Handle<StandardTransient> {
        Handle::new(StandardTransient::default())
    }

    #[test]
    fn chain_values_and_iteration() {
        let cluster = InterfaceEntityCluster::new();
        let all: Vec<Handle<StandardTransient>> = (0..9).map(|_| entity()).collect();
        all.iter().for_each(|e| cluster.append(e).unwrap());
        assert_eq!(cluster.nb_entities(), 9);
        assert!(cluster.value(9).unwrap() == all[8]);
        assert!(cluster.value(5).unwrap() == all[4]);
        assert_eq!(
            cluster.value(10).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
        let replaced = entity();
        cluster.set_value(6, &replaced).unwrap();
        let mut iter = InterfaceEntityIterator::new();
        cluster.fill_iterator(&mut iter);
        let content = iter.content();
        assert_eq!(content.len(), 9);
        assert!(content[5] == replaced);
        assert!(cluster.append(&Handle::default()).is_err());
    }

    #[test]
    fn removal_quirks() {
        let cluster = InterfaceEntityCluster::with_entity(&entity());
        let last = entity();
        cluster.append(&last).unwrap();
        // Only the fourth place is cleared: a cluster of two keeps its entities.
        assert!(!cluster.remove(&last).unwrap());
        assert_eq!(cluster.nb_local(), 2);
        assert!(cluster.remove_rank(1).is_err());
        assert!(cluster.remove_rank(0).is_err());
        assert!(!cluster.remove(&entity()).unwrap());
        let head = InterfaceEntityCluster::with_next(&Handle::new(cluster));
        assert_eq!(head.nb_entities(), 2);
        let single = InterfaceEntityCluster::with_entity_and_next(&entity(), &Handle::default());
        assert!(single.remove_rank(1).unwrap());
    }
}
