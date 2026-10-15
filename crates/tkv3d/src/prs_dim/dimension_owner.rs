//! `PrsDim_DimensionOwner`: the owner of the line or the label of a dimension, highlighting
//! that part only.
use super::{PrsDimComputeMode, PrsDimDimensionSelectionMode};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOPMOST;

/// `HighlightMode`: the compute mode highlighted for a selection mode.
fn highlight_mode(mode: PrsDimDimensionSelectionMode) -> i32 {
    match mode {
        PrsDimDimensionSelectionMode::Line => PrsDimComputeMode::Line as i32,
        PrsDimDimensionSelectionMode::Text => PrsDimComputeMode::Text as i32,
        PrsDimDimensionSelectionMode::All => PrsDimComputeMode::All as i32,
    }
}

/// `PrsDim_DimensionOwner`; `PrsDimDimensionOwner::new(&dimension.cast(), mode, 6)`.
pub struct PrsDimDimensionOwner {
    transient: StandardTransient,
    owner: SelectMgrEntityOwnerBase,
    selection_mode: PrsDimDimensionSelectionMode,
}
rustycad_tkernel::impl_transient!(PrsDimDimensionOwner, "PrsDim_DimensionOwner",
    dyn SelectMgrEntityOwnerSource, transient, interfaces: [dyn SelectMgrEntityOwnerSource]);
impl PrsDimDimensionOwner {
    /// The owner of a dimension part; `PrsDimDimensionOwner::new(&object, mode, 6)`.
    pub fn new(
        object: &Handle<dyn SelectMgrSelectableObject>,
        mode: PrsDimDimensionSelectionMode,
        priority: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            owner: SelectMgrEntityOwnerBase::new(object, priority),
            selection_mode: mode,
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
    /// `SelectionMode`; `owner.selection_mode()`.
    pub fn selection_mode(&self) -> PrsDimDimensionSelectionMode {
        self.selection_mode
    }
}
impl SelectMgrEntityOwnerSource for PrsDimDimensionOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.owner
    }
    /// `HilightWithColor`: the presentation of the owner's part colored with the style.
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        let manager = manager.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "PrsDim_DimensionOwner: null manager")
        })?;
        manager.color(
            &self.selectable().cast(),
            style,
            highlight_mode(self.selection_mode),
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOPMOST,
        )
    }
    /// `IsHilighted`: the highlight of the owner's part.
    fn is_hilighted(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) -> bool {
        self.has_selectable()
            && manager.is_highlighted(
                &self.selectable().cast(),
                highlight_mode(self.selection_mode),
            )
    }
    /// `Unhilight`: unhighlights the dimension.
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) {
        if self.has_selectable() {
            manager.unhighlight(&self.selectable().cast());
        }
    }
}
