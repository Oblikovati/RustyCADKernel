//! Trihedron part owners dispatch highlighting through the selectable object's virtual hook.
use crate::{prs_mgr::*, prs3d::Prs3dDatumParts, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
/// Native protected part field; `base.part.write()`.
pub struct AisTrihedronOwnerBase {
    /// Inherited priority and selectable identity.
    pub owner: SelectMgrEntityOwnerBase,
    /// Mutable protected datum part.
    pub part: SharedState<Prs3dDatumParts>,
}
impl AisTrihedronOwnerBase {
    /// Retain native owner state; `AisTrihedronOwnerBase::new(&object.cast(), part, 7)`.
    pub fn new(
        object: &Handle<dyn SelectMgrSelectableObject>,
        part: Prs3dDatumParts,
        priority: i32,
    ) -> Self {
        Self {
            owner: SelectMgrEntityOwnerBase::new(object, priority),
            part: SharedState::new(part),
        }
    }
}
/// Derived-owner interface; `owner.datum_part()`.
pub trait AisTrihedronOwnerSource: SelectMgrEntityOwnerSource {
    /// Embedded protected state; `owner.trihedron_owner_base()`.
    fn trihedron_owner_base(&self) -> &AisTrihedronOwnerBase;
    /// Selected native part; `owner.datum_part()`.
    fn datum_part(&self) -> Prs3dDatumParts {
        *self.trihedron_owner_base().part.read()
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisTrihedronOwnerSource,
    "AIS_TrihedronOwner",
    dyn SelectMgrEntityOwnerSource
);
/// Part-specific owner; `AisTrihedronOwner::new(&trihedron.cast(), part, 7)`.
pub struct AisTrihedronOwner {
    transient: StandardTransient,
    base: AisTrihedronOwnerBase,
}
rustycad_tkernel::impl_transient!(AisTrihedronOwner,"AIS_TrihedronOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn AisTrihedronOwnerSource,dyn SelectMgrEntityOwnerSource]);
impl AisTrihedronOwner {
    /// Bind selectable and part identity; `AisTrihedronOwner::new(&trihedron.cast(), part, 7)`.
    pub fn new(
        object: &Handle<dyn SelectMgrSelectableObject>,
        part: Prs3dDatumParts,
        priority: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisTrihedronOwnerBase::new(object, part, priority),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
}
impl AisTrihedronOwnerSource for AisTrihedronOwner {
    fn trihedron_owner_base(&self) -> &AisTrihedronOwnerBase {
        &self.base
    }
}
/// Dispatch the selectable's virtual highlight hook; `ais_trihedron_owner_hilight(&*owner, &manager, &style)?`.
pub fn ais_trihedron_owner_hilight<O: AisTrihedronOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let selectable = owner.selectable();
    let Some(selectable) = selectable.get() else {
        return Err(Failure::with_type(
            "Standard_ProgramError",
            "AIS_TrihedronOwner selectable null: expected a live selectable object",
            "",
        ));
    };
    selectable.hilight_owner_with_color(manager, style, &owner.owner_base().this())
}
impl SelectMgrEntityOwnerSource for AisTrihedronOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base.owner
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_owner_hilight(self, manager, style)
    }
    fn is_hilighted(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) -> bool {
        self.has_selectable() && manager.is_highlighted(&self.selectable().cast(), mode)
    }
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) {
        if self.has_selectable() {
            manager.unhighlight(&self.selectable().cast());
        }
    }
}
