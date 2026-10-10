//! Borrowed selected-owner storage and detached native iteration.
use super::{AisSelectedOwner, AisSelectionFields, AisSelectionSource};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, exceptions::no_such_object},
};
use std::{ops::Deref, sync::RwLockReadGuard};
/// Read guard for the actual ordered list; `selection.objects().length()`.
pub struct AisSelectedObjects<'a>(RwLockReadGuard<'a, AisSelectionFields>);
impl Deref for AisSelectedObjects<'_> {
    type Target = List<AisSelectedOwner>;
    fn deref(&self) -> &Self::Target {
        &self.0.owners
    }
}
/// Read guard for the actual owner handle at the public cursor; `selection.value()?.is_selected()`.
pub struct AisSelectionValue<'a>(RwLockReadGuard<'a, AisSelectionFields>);
impl Deref for AisSelectionValue<'_> {
    type Target = AisSelectedOwner;
    fn deref(&self) -> &Self::Target {
        self.0
            .cursor
            .value(&self.0.owners)
            .expect("validated selected owner cursor")
    }
}
/// Nonvirtual queries and iteration; `selection.init(); while selection.more() { selection.next(); }`.
pub trait AisSelectionMembers: AisSelectionSource {
    /// Membership reflects the map, independently of an owner's selected flag; `selection.is_selected(&owner)`.
    fn is_selected(&self, owner: &AisSelectedOwner) -> bool {
        self.selection_base()
            .fields
            .read()
            .positions
            .is_bound(owner)
    }
    /// Borrow the actual ordered owner list; release before mutation; `selection.objects().length()`.
    fn objects(&self) -> AisSelectedObjects<'_> {
        AisSelectedObjects(self.selection_base().fields.read())
    }
    /// Count list entries, including native duplicates; `selection.extent()`.
    fn extent(&self) -> i32 {
        self.selection_base().fields.read().owners.length()
    }
    /// Test the list rather than the map; `selection.is_empty()`.
    fn is_empty(&self) -> bool {
        self.selection_base().fields.read().owners.is_empty()
    }
    /// Reset public iteration to the first live node; `selection.init()`.
    fn init(&self) {
        let mut fields = self.selection_base().fields.write();
        fields.cursor = fields.owners.cursor();
    }
    /// Test the cursor's current-node identity; `selection.more()`.
    fn more(&self) -> bool {
        self.selection_base().fields.read().cursor.more()
    }
    /// Advance over current live links, retaining native end predecessor; `selection.next()`.
    fn next(&self) {
        let mut fields = self.selection_base().fields.write();
        let AisSelectionFields { owners, cursor, .. } = &mut *fields;
        cursor.next(owners);
    }
    /// Borrow the current stored handle; an invalid cursor returns NoSuchObject.
    /// Example: `let current=selection.value()?; let owner=current.clone()`.
    fn value(&self) -> Result<AisSelectionValue<'_>, Failure> {
        let fields = self.selection_base().fields.read();
        if fields.cursor.value(&fields.owners).is_none() {
            return Err(no_such_object(&format!(
                "AIS selection cursor {:?}; expected a live selected-owner node",
                fields.cursor.current_node()
            )));
        }
        Ok(AisSelectionValue(fields))
    }
}
impl<T: AisSelectionSource + ?Sized> AisSelectionMembers for T {}
