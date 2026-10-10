//! Selection orchestration with native child-first ordering (OCCT 3d097a0).
use super::*;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    ncollection::Map,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions::null_object},
};

pub(super) type ManagedSelectable = Handle<dyn SelectMgrSelectableObject>;
/// Owns a selector and the set of globally loaded objects; `SelectMgrSelectionManager::new(&selector)`.
pub struct SelectMgrSelectionManager {
    transient: StandardTransient,
    pub(super) selector: Handle<dyn SelectMgrViewerSelectorSource>,
    pub(super) global: SharedState<Map<ManagedSelectable>>,
}
rustycad_tkernel::impl_transient!(
    SelectMgrSelectionManager,
    "SelectMgr_SelectionManager",
    StandardTransient,
    transient
);
impl SelectMgrSelectionManager {
    /// Retain the injected selector, including a native null handle; `SelectMgrSelectionManager::new(&selector)`.
    pub fn new(selector: &Handle<dyn SelectMgrViewerSelectorSource>) -> Self {
        Self {
            transient: StandardTransient::default(),
            selector: selector.clone(),
            global: SharedState::default(),
        }
    }
    /// Borrow the owned selector handle; `manager.selector().is_null()`.
    pub fn selector(&self) -> &Handle<dyn SelectMgrViewerSelectorSource> {
        &self.selector
    }
    /// Check global membership independently of the selector; `manager.contains(&object)`.
    pub fn contains(&self, object: &ManagedSelectable) -> bool {
        self.global.read().contains(object)
    }
    pub(super) fn required_selector(&self) -> Result<&dyn SelectMgrViewerSelectorSource, Failure> {
        self.selector.get().ok_or_else(|| {
            null_object("selection manager selector is null; expected a viewer selector")
        })
    }
    pub(super) fn owns_presentations(&self, object: &ManagedSelectable) -> Result<bool, Failure> {
        Ok(managed_object(object)?.has_own_presentations() && self.contains(object))
    }
}
pub(super) fn managed_object(
    object: &ManagedSelectable,
) -> Result<&dyn SelectMgrSelectableObject, Failure> {
    object
        .get()
        .ok_or_else(|| null_object("managed selectable is null; expected a selectable object"))
}
pub(super) fn managed_children(
    object: &ManagedSelectable,
) -> Result<Vec<ManagedSelectable>, Failure> {
    Ok(managed_object(object)?
        .children()
        .into_iter()
        .map(|child| child.cast())
        .collect())
}
pub(super) fn managed_modes(
    object: &ManagedSelectable,
    mode: i32,
) -> Result<Vec<Handle<SelectMgrSelection>>, Failure> {
    let object = managed_object(object)?;
    if mode == -1 {
        return Ok(object.selections());
    }
    let selection = object.selection(mode);
    Ok(if selection.is_null() {
        Vec::new()
    } else {
        vec![selection]
    })
}
