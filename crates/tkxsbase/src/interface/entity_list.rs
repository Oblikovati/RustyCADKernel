// Rust port of DataExchange/TKXSBase/Interface/Interface_EntityList (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_EntityList`: a list of entities optimised for zero or one entity: the
//! value is the entity itself, or an `Interface_EntityCluster` chain beyond one.
use super::errors::interface_error;
use super::{InterfaceEntityCluster, InterfaceEntityIterator};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeType, StandardTransient, exceptions};

/// `Interface_EntityList`. Copies share the value (natively a handle copy).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceEntityList;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let mut list = InterfaceEntityList::new();
/// list.append(&Handle::new(StandardTransient::default())).unwrap();
/// list.append(&Handle::new(StandardTransient::default())).unwrap();
/// assert_eq!(list.nb_entities(), 2);
/// ```
#[derive(Clone, Default)]
pub struct InterfaceEntityList {
    theval: Handle<StandardTransient>,
}

fn null_entity(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "Interface_EntityList {member}: null entity; expected a live entity"
    ))
}

fn out_of_range(member: &str, num: i32) -> Failure {
    exceptions::out_of_range(&format!(
        "Interface EntityList : {member}: rank {num} is out of the list"
    ))
}

impl InterfaceEntityList {
    /// `Interface_EntityList()`: empty.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Clear`.
    pub fn clear(&mut self) {
        self.theval.nullify();
    }
    /// `Append(ent)`: at the end; `Standard_NullObject` for null.
    pub fn append(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("Append"));
        }
        if self.theval.is_null() {
            self.theval = ent.clone();
            return Ok(());
        }
        if let Some(cluster) = self.cluster().get() {
            return cluster.append(ent);
        }
        self.wrap_single(ent)
    }
    /// `Add(ent)`: at the end, or at the front when the first cluster is full (a new
    /// cluster then heads the chain); `Standard_NullObject` for null.
    pub fn add(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("Add"));
        }
        if self.theval.is_null() {
            self.theval = ent.clone();
            return Ok(());
        }
        let cluster = self.cluster();
        let Some(live) = cluster.get() else {
            return self.wrap_single(ent);
        };
        if live.is_local_full() {
            self.theval =
                Handle::new(InterfaceEntityCluster::with_entity_and_next(ent, &cluster)).cast();
            return Ok(());
        }
        live.append(ent)
    }
    /// `Remove(ent)`: `Standard_NullObject` for null.
    pub fn remove(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("Remove"));
        }
        if self.theval.is_null() {
            return Ok(());
        }
        if self.theval == *ent {
            self.theval.nullify();
            return Ok(());
        }
        let cluster = self.cluster();
        let Some(live) = cluster.get() else {
            return Ok(());
        };
        if live.remove(ent)? {
            self.theval.nullify();
        }
        Ok(())
    }
    /// `Remove(num)`: `Standard_OutOfRange` for an empty list or a bad rank.
    pub fn remove_rank(&mut self, num: i32) -> Result<(), Failure> {
        if self.theval.is_null() {
            return Err(out_of_range("Remove", num));
        }
        let cluster = self.cluster();
        let Some(live) = cluster.get() else {
            if num != 1 {
                return Err(out_of_range("Remove", num));
            }
            self.theval.nullify();
            return Ok(());
        };
        if live.remove_rank(num)? {
            self.theval.nullify();
        }
        Ok(())
    }
    /// `IsEmpty`.
    pub fn is_empty(&self) -> bool {
        self.theval.is_null()
    }
    /// `NbEntities`.
    pub fn nb_entities(&self) -> i32 {
        if self.theval.is_null() {
            return 0;
        }
        self.cluster()
            .get()
            .map_or(1, InterfaceEntityCluster::nb_entities)
    }
    /// `Value(num)`: `Standard_OutOfRange` out of the list.
    pub fn value(&self, num: i32) -> Result<Handle<StandardTransient>, Failure> {
        if self.theval.is_null() {
            return Err(out_of_range("Value", num));
        }
        if let Some(cluster) = self.cluster().get() {
            return cluster.value(num);
        }
        if num != 1 {
            return Err(out_of_range("Value", num));
        }
        Ok(self.theval.clone())
    }
    /// `SetValue(num, ent)`: `Standard_NullObject` for null, `Standard_OutOfRange` out of
    /// the list.
    pub fn set_value(&mut self, num: i32, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        if ent.is_null() {
            return Err(null_entity("SetValue"));
        }
        if self.theval.is_null() {
            return Err(out_of_range("SetValue", num));
        }
        if let Some(cluster) = self.cluster().get() {
            return cluster.set_value(num, ent);
        }
        if num != 1 {
            return Err(out_of_range("SetValue", num));
        }
        self.theval = ent.clone();
        Ok(())
    }
    /// `FillIterator(iter)`.
    pub fn fill_iterator(&self, iter: &mut InterfaceEntityIterator) {
        if self.theval.is_null() {
            return;
        }
        match self.cluster().get() {
            Some(cluster) => cluster.fill_iterator(iter),
            None => iter.get_one_item(&self.theval),
        }
    }
    /// `NbTypedEntities(atype)`: the entities kind of `atype`.
    pub fn nb_typed_entities(&self, atype: &RuntimeType) -> i32 {
        if self.theval.is_null() {
            return 0;
        }
        if self.cluster().is_null() {
            return i32::from(self.theval.is_kind(atype));
        }
        self.typed_walk(atype).len() as i32
    }
    /// `TypedEntity(atype, num = 0)`: the entity of rank `num` among those kind of
    /// `atype` (each cluster scanned from its last place, as natively); for 0, the only
    /// one. `Interface_InterfaceError` when none, several (for 0) or out of range.
    /// Native quirk kept: a rank beyond the count returns the last one found.
    pub fn typed_entity(
        &self,
        atype: &RuntimeType,
        num: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        if self.theval.is_null() {
            return Err(interface_error(
                "Interface EntityList : TypedEntity , none found",
            ));
        }
        if self.cluster().is_some() {
            return self.typed_in_clusters(atype, num);
        }
        if num > 1 {
            return Err(interface_error(
                "Interface EntityList : TypedEntity ,out of range",
            ));
        }
        if !self.theval.is_kind(atype) {
            return Err(interface_error(
                "Interface EntityList : TypedEntity , none found",
            ));
        }
        Ok(self.theval.clone())
    }
    fn typed_in_clusters(
        &self,
        atype: &RuntimeType,
        num: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let found = self.typed_walk(atype);
        if num == 0 && found.len() > 1 {
            return Err(interface_error(
                "Interface EntityList : TypedEntity , several found",
            ));
        }
        let rank = usize::try_from(num)
            .ok()
            .filter(|r| *r >= 1 && *r <= found.len());
        let chosen = rank.map_or_else(|| found.last().cloned(), |r| Some(found[r - 1].clone()));
        Ok(chosen.unwrap_or_default())
    }
    /// The entities kind of `atype` in the native scan order: each cluster from its last
    /// local place down to the first.
    fn typed_walk(&self, atype: &RuntimeType) -> Vec<Handle<StandardTransient>> {
        let mut found = Vec::new();
        let mut current = self.cluster();
        while let Some(cluster) = current.get() {
            for local in (1..=cluster.nb_local()).rev() {
                let entity = cluster.value(local).unwrap_or_default();
                if entity.is_kind(atype) {
                    found.push(entity);
                }
            }
            current = cluster.next();
        }
        found
    }
    fn cluster(&self) -> Handle<InterfaceEntityCluster> {
        self.theval.cast()
    }
    fn wrap_single(&mut self, ent: &Handle<StandardTransient>) -> Result<(), Failure> {
        let cluster = InterfaceEntityCluster::with_entity(&self.theval);
        cluster.append(ent)?;
        self.theval = Handle::new(cluster).cast();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::{standard::RuntimeTyped, tcollection::HAsciiString};

    fn entity() -> Handle<StandardTransient> {
        Handle::new(StandardTransient::default())
    }

    #[test]
    fn single_and_clustered_values() {
        let mut list = InterfaceEntityList::new();
        assert!(list.value(1).is_err());
        let first = entity();
        list.add(&first).unwrap();
        assert!(list.value(1).unwrap() == first);
        assert!(list.value(2).is_err());
        let text: Handle<StandardTransient> = Handle::new(HAsciiString::new("t")).cast();
        list.append(&text).unwrap();
        (0..3).for_each(|_| list.add(&entity()).unwrap());
        assert_eq!(list.nb_entities(), 5);
        // The fifth Add found the first cluster full and put a new cluster in front.
        assert!(list.value(1).unwrap() != first);
        assert_eq!(list.nb_typed_entities(HAsciiString::static_type()), 1);
        assert!(list.typed_entity(HAsciiString::static_type(), 0).unwrap() == text);
        let several = list
            .typed_entity(StandardTransient::static_type(), 0)
            .err()
            .unwrap();
        assert_eq!(several.exception_type(), "Interface_InterfaceError");
        let mut iter = InterfaceEntityIterator::new();
        list.fill_iterator(&mut iter);
        assert_eq!(iter.nb_entities(), 5);
    }

    #[test]
    fn single_entity_edits() {
        let mut list = InterfaceEntityList::new();
        let first = entity();
        list.append(&first).unwrap();
        assert_eq!(list.nb_typed_entities(StandardTransient::static_type()), 1);
        assert!(list.typed_entity(HAsciiString::static_type(), 0).is_err());
        assert!(
            list.typed_entity(StandardTransient::static_type(), 2)
                .is_err()
        );
        let other = entity();
        list.set_value(1, &other).unwrap();
        assert!(list.set_value(2, &first).is_err());
        list.remove(&first).unwrap();
        assert!(!list.is_empty());
        list.remove(&other).unwrap();
        assert!(list.is_empty());
        list.append(&first).unwrap();
        assert!(list.remove_rank(2).is_err());
        list.remove_rank(1).unwrap();
        assert!(list.is_empty());
        list.append(&first).unwrap();
        list.clear();
        assert_eq!(list.nb_entities(), 0);
        assert!(list.append(&Handle::default()).is_err());
    }
}
