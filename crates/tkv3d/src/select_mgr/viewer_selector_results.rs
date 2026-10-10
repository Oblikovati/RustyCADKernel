//! Lazy sorting and borrowed picked criteria preserve native invalidation boundaries.
use super::*;
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle, exceptions::out_of_range},
    stl::try_sort_by,
};
use rustycad_tkmath::gp::Point3;
use std::{ops::Deref, sync::RwLockReadGuard};

/// Native SortResult body for subclasses that extend the virtual sorter.
/// Example: `select_mgr_sort_selector_results(self.viewer_selector_base())?`.
pub fn select_mgr_sort_selector_results(base: &SelectMgrViewerSelectorBase) -> Result<(), Failure> {
    let mut fields = base.fields.write();
    let prefer_closest = fields.settings.to_prefer_closest;
    let results = &mut fields.results;
    let count = results.stored.extent();
    if count == 0 {
        results.is_sorted = true;
        return Ok(());
    }
    if results.indexes.len() != count {
        results.indexes = Array1::with_bounds(1, count as i32).expect("positive picked extent");
    }
    for (index, entry) in results.indexes.as_mut_slice().iter_mut().enumerate() {
        *entry = index as i32 + 1;
    }
    try_sort_by(results.indexes.as_mut_slice(), |a, b| {
        compare_selector_results(
            results.stored.find_from_index(*a as usize)?,
            results.stored.find_from_index(*b as usize)?,
            prefer_closest,
        )
    })?;
    results.is_sorted = true;
    Ok(())
}
fn compare_selector_results(
    first: &SelectMgrSortCriterion,
    second: &SelectMgrSortCriterion,
    prefer_closest: bool,
) -> Result<bool, Failure> {
    if (first.is_prefer_priority && second.is_prefer_priority) || !prefer_closest {
        return Ok(first.is_higher_priority(second));
    }
    first.is_closer_depth(second)
}

/// Borrowed native `PickedData` reference; `selector.picked_data(1)?.point`.
pub struct SelectMgrPickedCriterion<'a> {
    fields: RwLockReadGuard<'a, SelectMgrViewerSelectorFields>,
    index: usize,
}
impl Deref for SelectMgrPickedCriterion<'_> {
    type Target = SelectMgrSortCriterion;
    fn deref(&self) -> &Self::Target {
        self.fields
            .results
            .stored
            .find_from_index(self.index)
            .expect("retained picked criterion")
    }
}
/// Borrowed native `PickedEntity` handle; `selector.picked_entity(1)?.is_null()`.
pub struct SelectMgrPickedEntity<'a>(SelectMgrPickedCriterion<'a>);
impl Deref for SelectMgrPickedEntity<'_> {
    type Target = Handle<dyn Select3DSensitiveEntity>;
    fn deref(&self) -> &Self::Target {
        &self.0.entity
    }
}
/// Nonvirtual result operations dispatch to a subclass's `sort_result` when needed.
pub trait SelectMgrViewerSelectorResultMembers: SelectMgrViewerSelectorSource {
    /// Count detected owners; `selector.nb_picked()`.
    fn nb_picked(&self) -> i32 {
        self.viewer_selector_base()
            .fields
            .read()
            .results
            .stored
            .extent() as i32
    }
    /// Clear detected owners while retaining the index allocation; `selector.clear_picked()`.
    fn clear_picked(&self) {
        let mut fields = self.viewer_selector_base().fields.write();
        fields.results.stored.clear(true);
        fields.results.is_sorted = true;
    }
    /// Native Clear only clears picks; `selector.clear()`.
    fn clear(&self) {
        self.clear_picked();
    }
    /// Return the highest-ranked owner or null; `selector.one_picked()?`.
    fn one_picked(&self) -> Result<Handle<dyn SelectMgrEntityOwnerSource>, Failure> {
        self.picked(1)
    }
    /// Out-of-range ranks return null before sorting; `selector.picked(1)?`.
    fn picked(&self, rank: i32) -> Result<Handle<dyn SelectMgrEntityOwnerSource>, Failure> {
        if rank < 1 || rank > self.nb_picked() {
            return Ok(Handle::default());
        }
        if !self.viewer_selector_base().fields.read().results.is_sorted {
            self.sort_result()?;
        }
        let fields = self.viewer_selector_base().fields.read();
        let index = selector_rank_index(&fields, rank)?;
        Ok(fields.results.stored.find_key(index)?.clone())
    }
    /// Borrow a criterion after lazy sorting; invalid ranks raise OutOfRange.
    /// Example: `let criterion = selector.picked_data(1)?; let point = criterion.point`.
    fn picked_data(&self, rank: i32) -> Result<SelectMgrPickedCriterion<'_>, Failure> {
        if rank < 1 || rank > self.nb_picked() {
            return Err(out_of_range(&format!(
                "SelectMgr_ViewerSelector::PickedData() index {rank}; expected 1..={}",
                self.nb_picked()
            )));
        }
        if !self.viewer_selector_base().fields.read().results.is_sorted {
            self.sort_result()?;
        }
        let fields = self.viewer_selector_base().fields.read();
        let index = selector_rank_index(&fields, rank)?;
        fields.results.stored.find_from_index(index)?;
        Ok(SelectMgrPickedCriterion { fields, index })
    }
    /// Borrow the stored sensitive handle of a rank; `selector.picked_entity(1)?`.
    fn picked_entity(&self, rank: i32) -> Result<SelectMgrPickedEntity<'_>, Failure> {
        Ok(SelectMgrPickedEntity(self.picked_data(rank)?))
    }
    /// Copy the intersection point of a rank; `selector.picked_point(1)?`.
    fn picked_point(&self, rank: i32) -> Result<Point3, Failure> {
        Ok(self.picked_data(rank)?.point)
    }
    /// Remove every picked owner of a contained object; `selector.remove_picked(&object)?`.
    fn remove_picked(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<bool, Failure> {
        let base = self.viewer_selector_base();
        if self.nb_picked() == 0 || !base.selectable_objects.contains(object) {
            return Ok(false);
        }
        let mut fields = base.fields.write();
        let mut index = 1;
        let old_count = fields.results.stored.extent();
        while index <= fields.results.stored.extent() {
            let owner = fields.results.stored.find_key(index)?;
            if owner
                .get()
                .is_some_and(|owner| owner.is_same_selectable(object))
            {
                fields.results.stored.remove_from_index(index)?;
            } else {
                index += 1;
            }
        }
        let removed = old_count != fields.results.stored.extent();
        if removed {
            fields.results.is_sorted = false;
        }
        Ok(removed)
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorResultMembers for T {}

fn selector_rank_index(
    fields: &SelectMgrViewerSelectorFields,
    rank: i32,
) -> Result<usize, Failure> {
    fields
        .results
        .indexes
        .value(rank)
        .map(|&index| index as usize)
        .map_err(|error| out_of_range(&error.to_string()))
}
