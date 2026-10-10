//! Ordered composition evaluates until the first decision or callback failure.
use super::{
    ais_selection::fixture::AisStateFilter,
    boxed_selectable::BoxedSelectable,
    common::{Reader, attempt, end, flag, int},
};
use rustycad_tkernel::{
    ncollection::Shared,
    standard::{Handle, SharedState},
};
use rustycad_tkv3d::{ais::AisSelection, select_mgr::*};
use std::{collections::BTreeSet, sync::atomic::Ordering};
pub fn ais_filters_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, disabled, count) = (r.integer(), r.integer(), r.integer());
    let selection = Handle::new(AisSelection::new());
    let object = BoxedSelectable::new(0., false).cast::<dyn SelectMgrSelectableObject>();
    let owner = SelectMgrEntityOwner::with_selectable(&object, 0).cast();
    let callbacks: Vec<_> = (0..count)
        .map(|_| {
            let filter = AisStateFilter::new(&selection.cast(), &owner);
            filter.mode.store(r.integer(), Ordering::Release);
            filter
        })
        .collect();
    let and = SelectMgrAndFilter::default();
    let or = SelectMgrOrFilter::default();
    let combined = SelectMgrAndOrFilter::new(if kind == 2 {
        SelectMgrFilterType::And
    } else {
        SelectMgrFilterType::Or
    });
    let blocked = Handle::new(Shared::new(SharedState::new(BTreeSet::new())));
    if disabled != 0 {
        blocked.write().insert(object.object_address() as usize);
    }
    combined.set_disabled_objects(&blocked);
    for filter in &callbacks {
        and.add(&filter.cast());
        or.add(&filter.cast());
        combined.add(&filter.cast());
    }
    attempt(|| {
        flag(match kind {
            0 => and.is_ok(&owner)?,
            1 => or.is_ok(&owner)?,
            _ => combined.is_ok(&owner)?,
        });
        Ok(())
    });
    for filter in callbacks {
        int(filter.observed.read().len() as i32);
    }
    end();
}
