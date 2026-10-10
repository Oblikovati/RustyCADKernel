//! Indexed sensitive entities, owner counts and native BVH ownership.
use super::{SelectMgrEntityOwnerSource, SelectMgrSelection, SelectMgrSensitiveEntity};
use crate::select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers};
use rustycad_tkernel::{
    ncollection::{DataMap, IndexedMap},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::bvh::{
    BvhBuilderHandle, BvhObjectTransientSource, BvhPrimitiveSet3d, BvhVec3d,
};
use std::sync::{Mutex, MutexGuard, RwLockReadGuard};

/// One selectable object's unique sensitives and shared live BVH.
/// Example: `let set = SelectMgrSensitiveEntitySet::new(Some(builder)); set.append(&entity)?;`.
pub struct SelectMgrSensitiveEntitySet {
    transient: StandardTransient,
    pub(super) content: Mutex<BvhPrimitiveSet3d>,
    sensitives: SharedState<IndexedMap<Handle<SelectMgrSensitiveEntity>>>,
    owners: SharedState<DataMap<Handle<dyn SelectMgrEntityOwnerSource>, i32>>,
    persistence_counts: SharedState<(i32, i32)>,
}
rustycad_tkernel::impl_transient!(SelectMgrSensitiveEntitySet, "SelectMgr_SensitiveEntitySet",
    dyn BvhObjectTransientSource, transient, interfaces: [dyn BvhObjectTransientSource]);

impl SelectMgrSensitiveEntitySet {
    /// Start clean with an empty tree and the caller's builder, including null.
    /// Example: `SelectMgrSensitiveEntitySet::new(None)`.
    pub fn new(builder: BvhBuilderHandle<BvhVec3d>) -> Self {
        Self {
            transient: StandardTransient::default(),
            content: Mutex::new(BvhPrimitiveSet3d::new(builder)),
            sensitives: SharedState::new(IndexedMap::default()),
            owners: SharedState::new(DataMap::default()),
            persistence_counts: SharedState::new((0, 0)),
        }
    }
    pub(super) fn lock_content(&self) -> MutexGuard<'_, BvhPrimitiveSet3d> {
        self.content
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Add one entity, deactivating rejected native runtime types; `set.append(&entity)?`.
    pub fn append(&self, entity: &Handle<SelectMgrSensitiveEntity>) -> Result<(), Failure> {
        if self.append_entry(entity)? {
            self.mark_dirty();
        }
        Ok(())
    }
    /// Add every selection entity; dirty marking occurs after the loop; `set.append_selection(&selection)?`.
    pub fn append_selection(&self, selection: &Handle<SelectMgrSelection>) -> Result<(), Failure> {
        let selection = selection
            .get()
            .ok_or_else(|| set_null_handle("selection"))?;
        for entity in selection.entities() {
            self.append_entry(&entity)?;
        }
        self.mark_dirty();
        Ok(())
    }
    fn append_entry(&self, entity: &Handle<SelectMgrSensitiveEntity>) -> Result<bool, Failure> {
        let sensitive = set_base_sensitive(entity)?;
        if !sensitive.is_kind_name("Select3D_SensitiveEntity") {
            entity.reset_selection_active_status();
            return Ok(false);
        }
        let inserted = {
            let mut map = self.sensitives.write();
            let old = map.extent();
            map.add(entity.clone()) > old
        };
        if inserted {
            self.add_owner(&sensitive.owner_id());
        }
        // OCCT 3d097a0 increments these even when IndexedMap rejects a duplicate.
        self.adjust_persistence_counts(&sensitive, 1);
        Ok(true)
    }
    /// Remove selection entities with native swap-last ordering; `set.remove(&selection)?`.
    pub fn remove(&self, selection: &Handle<SelectMgrSelection>) -> Result<(), Failure> {
        let selection = selection
            .get()
            .ok_or_else(|| set_null_handle("selection"))?;
        for entity in selection.entities() {
            self.remove_entry(&entity)?;
        }
        self.mark_dirty();
        Ok(())
    }
    fn remove_entry(&self, entity: &Handle<SelectMgrSensitiveEntity>) -> Result<(), Failure> {
        let mut map = self.sensitives.write();
        let index = map.find_index(entity);
        if index == 0 {
            return Ok(());
        }
        let last = map.len();
        if index != last {
            map.swap(index, last)?;
        }
        let sensitive = set_base_sensitive(entity)?;
        self.adjust_persistence_counts(&sensitive, -1);
        map.remove_last()?;
        self.remove_owner(&sensitive.owner_id());
        Ok(())
    }
    fn adjust_persistence_counts(
        &self,
        sensitive: &Handle<dyn Select3DSensitiveEntity>,
        change: i32,
    ) {
        let persistent = !sensitive.transform_persistence().is_null();
        let flipping = !sensitive.flipper().is_null();
        let mut counts = self.persistence_counts.write();
        counts.0 += i32::from(persistent) * change;
        counts.1 += i32::from(flipping) * change;
    }
    /// Read the unique-entity count; `set.size()`.
    pub fn size(&self) -> i32 {
        self.sensitives.read().len() as i32
    }
    /// Read a zero-based entity ID; `set.get_sensitive_by_id(0)?`.
    pub fn get_sensitive_by_id(
        &self,
        index: i32,
    ) -> Result<Handle<SelectMgrSensitiveEntity>, Failure> {
        self.sensitives
            .read()
            .find_key_int(set_map_index(index)?)
            .cloned()
    }
    /// Swap two zero-based BVH IDs; `set.swap(0,1)?`.
    pub fn swap(&self, first: i32, second: i32) -> Result<(), Failure> {
        self.sensitives
            .write()
            .swap_int(set_map_index(first)?, set_map_index(second)?)
    }
    /// Borrow the live indexed map; release the guard before changing the set.
    /// Example: `set.sensitives().len()`.
    pub fn sensitives(&self) -> RwLockReadGuard<'_, IndexedMap<Handle<SelectMgrSensitiveEntity>>> {
        self.sensitives.read()
    }
    /// Borrow live owner counts; release the guard before changing the set.
    /// Example: `set.owners().find(&owner)?`.
    pub fn owners(
        &self,
    ) -> RwLockReadGuard<'_, DataMap<Handle<dyn SelectMgrEntityOwnerSource>, i32>> {
        self.owners.read()
    }
    /// Whether the native persistence counter is positive; `set.has_entity_with_persistence()`.
    pub fn has_entity_with_persistence(&self) -> bool {
        self.persistence_counts.read().0 > 0
    }
    /// Whether the native flipping counter is positive; `set.has_entity_with_flipping()`.
    pub fn has_entity_with_flipping(&self) -> bool {
        self.persistence_counts.read().1 > 0
    }
    /// Protected owner-count addition, ignoring null; `set.add_owner(&owner)`.
    pub fn add_owner(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        if owner.is_null() {
            return;
        }
        let mut owners = self.owners.write();
        if let Some(count) = owners.change_seek(owner) {
            *count += 1;
        } else {
            owners.bind(owner.clone(), 1);
        }
    }
    /// Protected owner-count removal, unbinding at zero; `set.remove_owner(&owner)`.
    pub fn remove_owner(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        if owner.is_null() {
            return;
        }
        let mut owners = self.owners.write();
        let Some(count) = owners.change_seek(owner) else {
            return;
        };
        *count -= 1;
        if *count == 0 {
            owners.unbind(owner);
        }
    }
}
pub(super) fn set_base_sensitive(
    entity: &Handle<SelectMgrSensitiveEntity>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let sensitive = entity
        .get()
        .ok_or_else(|| set_null_handle("entity"))?
        .base_sensitive();
    if sensitive.is_null() {
        return Err(set_null_handle("base sensitive"));
    }
    Ok(sensitive)
}
fn set_null_handle(field: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("SensitiveEntitySet {field}=null; expected nonnull handle"),
    )
}
fn set_map_index(index: i32) -> Result<i32, Failure> {
    index.checked_add(1).ok_or_else(|| {
        Failure::with_message(
            "Standard_OutOfRange",
            &format!("SensitiveEntitySet index={index}; expected representable one-based index"),
        )
    })
}
