//! Owned context state with non-owning links from interactive objects.
use super::{AisGlobalStatus, AisInteractiveObject, AisSelection, AisSelectionSource};
use crate::{
    prs_mgr::PrsMgrPresentationManager,
    prs3d::Prs3dDrawer,
    select_mgr::*,
    v3d::{V3dView, V3dViewer},
};
use rustycad_tkernel::{
    ncollection::{DataMap, Sequence},
    standard::{
        Failure, Handle, SharedState, StandardTransient, WeakHandle, exceptions::null_object,
    },
};
/// Native protected context storage; `context.fields.read().objects.len()`.
pub struct AisInteractiveContextFields {
    /// `myObjects`.
    pub objects: DataMap<Handle<dyn AisInteractiveObject>, Handle<AisGlobalStatus>>,
    /// `mgrSelector`.
    pub selection_manager: Handle<SelectMgrSelectionManager>,
    /// `myMainPM`.
    pub presentation_manager: Handle<PrsMgrPresentationManager>,
    /// `myMainVwr`.
    pub viewer: Handle<V3dViewer>,
    /// `myLastActiveView`: native uninitialized until a selection operation; safely null here.
    pub last_active_view: WeakHandle<V3dView>,
    /// `myLastPicked`.
    pub last_picked: Handle<dyn SelectMgrEntityOwnerSource>,
    /// `myToHilightSelected`.
    pub to_hilight_selected: bool,
    /// `mySelection`.
    pub selection: Handle<dyn AisSelectionSource>,
    /// `myFilters`.
    pub filters: Handle<SelectMgrAndOrFilter>,
    /// `myDefaultDrawer`.
    pub default_drawer: Handle<Prs3dDrawer>,
    /// `myStyles`.
    pub styles: [Handle<Prs3dDrawer>; 6],
    /// `myDetectedSeq`.
    pub detected: Sequence<i32>,
    /// `myCurDetected`.
    pub current_detected: i32,
    /// `myCurHighlighted`.
    pub current_highlighted: i32,
    /// `myPickingStrategy`.
    pub picking_strategy: SelectMgrPickingStrategy,
    /// `myAutoHilight`.
    pub automatic_hilight: bool,
    /// `myIsAutoActivateSelMode`.
    pub auto_activate_selection: bool,
}
impl AisInteractiveContextFields {
    fn new(viewer: &Handle<V3dViewer>) -> Result<Self, Failure> {
        let source = viewer
            .get()
            .ok_or_else(|| null_object("context viewer is null; expected a live V3d viewer"))?;
        let presentation_manager = PrsMgrPresentationManager::new(&source.structure_manager());
        let selection_manager = Handle::new(SelectMgrSelectionManager::new(
            &Handle::new(SelectMgrViewerSelector::new()).cast(),
        ));
        let styles = super::context_styles::context_default_styles()?;
        Ok(Self {
            objects: DataMap::default(),
            selection_manager,
            presentation_manager,
            viewer: viewer.clone(),
            last_active_view: WeakHandle::default(),
            last_picked: Handle::default(),
            to_hilight_selected: true,
            selection: Handle::new(AisSelection::new()).cast(),
            filters: Handle::new(SelectMgrAndOrFilter::new(SelectMgrFilterType::Or)),
            default_drawer: styles[0].clone(),
            styles,
            detected: Sequence::default(),
            current_detected: 0,
            current_highlighted: 0,
            picking_strategy: SelectMgrPickingStrategy::FirstAcceptable,
            automatic_hilight: true,
            auto_activate_selection: true,
        })
    }
}
/// Interactive context; construction and object ownership are native, while picking remains partial.
/// Example: `let context=AisInteractiveContext::new(&viewer)?`.
pub struct AisInteractiveContext {
    transient: StandardTransient,
    /// Protected state is shared by all handles; release its guard before callbacks.
    pub fields: SharedState<AisInteractiveContextFields>,
}
rustycad_tkernel::impl_transient!(
    AisInteractiveContext,
    "AIS_InteractiveContext",
    StandardTransient,
    transient
);
impl AisInteractiveContext {
    /// Create all native managers, selection, filters, drawers and initial styles.
    /// Example: `AisInteractiveContext::new(&viewer)?`.
    pub fn new(viewer: &Handle<V3dViewer>) -> Result<Handle<Self>, Failure> {
        let context = Handle::new(Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(AisInteractiveContextFields::new(viewer)?),
        });
        context
            .fields
            .read()
            .selection_manager
            .selector()
            .set_pixel_tolerance(2);
        Ok(context)
    }
}
impl Drop for AisInteractiveContext {
    fn drop(&mut self) {
        let selection = self.fields.read().selection.clone();
        // OCCT 3d097a0 calls virtual Clear from a noexcept destructor: failure terminates.
        if selection.clear().is_err() {
            std::process::abort();
        }
        let manager = std::mem::take(&mut self.fields.write().selection_manager);
        drop(manager);
        let objects: Vec<_> = self
            .fields
            .read()
            .objects
            .items()
            .map(|(object, _)| object.clone())
            .collect();
        for object in objects {
            object.set_context(&Handle::default());
            for selection in object.selections() {
                selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Renew);
            }
        }
    }
}
