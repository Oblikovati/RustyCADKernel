//! `AIS_ManipulatorMode` and `AIS_ManipulatorOwner`: the manipulation modes and the owner of
//! one manipulator part.
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// `AIS_ManipulatorMode`: the manipulation (and selection) modes of a manipulator.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AisManipulatorMode {
    /// `AIS_MM_None`.
    #[default]
    None = 0,
    /// `AIS_MM_Translation`.
    Translation = 1,
    /// `AIS_MM_Rotation`.
    Rotation = 2,
    /// `AIS_MM_Scaling`.
    Scaling = 3,
    /// `AIS_MM_TranslationPlane`.
    TranslationPlane = 4,
}
impl AisManipulatorMode {
    /// The mode of a selection mode number (none for other numbers).
    /// Example: `AisManipulatorMode::from_i32(2) == AisManipulatorMode::Rotation`.
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Translation,
            2 => Self::Rotation,
            3 => Self::Scaling,
            4 => Self::TranslationPlane,
            _ => Self::None,
        }
    }
}

/// `AIS_ManipulatorOwner`: the owner of a manipulator part (axis index and mode), highlighted
/// by the manipulator itself; `AisManipulatorOwner::new(&manipulator.cast(), 0, mode, 9)`.
pub struct AisManipulatorOwner {
    transient: StandardTransient,
    owner: SelectMgrEntityOwnerBase,
    index: i32,
    mode: AisManipulatorMode,
}
rustycad_tkernel::impl_transient!(AisManipulatorOwner, "AIS_ManipulatorOwner",
    dyn SelectMgrEntityOwnerSource, transient, interfaces: [dyn SelectMgrEntityOwnerSource]);
impl AisManipulatorOwner {
    /// The owner of an axis part of a selectable object.
    /// Example: `AisManipulatorOwner::new(&object, 1, AisManipulatorMode::Rotation, 9)`.
    pub fn new(
        object: &Handle<dyn SelectMgrSelectableObject>,
        index: i32,
        mode: AisManipulatorMode,
        priority: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            owner: SelectMgrEntityOwnerBase::new(object, priority),
            index,
            mode,
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
    /// `Mode`; `owner.mode()`.
    pub fn mode(&self) -> AisManipulatorMode {
        self.mode
    }
    /// `Index`: the axis index; `owner.index()`.
    pub fn index(&self) -> i32 {
        self.index
    }
}
impl SelectMgrEntityOwnerSource for AisManipulatorOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.owner
    }
    /// `HilightWithColor`: the base highlight in mode 0, else the part highlight of the
    /// manipulator.
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode == 0 {
            return select_mgr_owner_hilight_with_color(self, manager, style, mode);
        }
        let selectable = self.selectable();
        let object = selectable.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_ManipulatorOwner: no selectable")
        })?;
        object.hilight_owner_with_color(manager, style, &self.owner_base().this())
    }
    /// `IsHilighted`: the highlight of the selectable in the manipulation mode.
    fn is_hilighted(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) -> bool {
        self.has_selectable() && manager.is_highlighted(&self.selectable().cast(), self.mode as i32)
    }
    /// `Unhilight`: unhighlights the selectable.
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) {
        if self.has_selectable() {
            manager.unhighlight(&self.selectable().cast());
        }
    }
}
