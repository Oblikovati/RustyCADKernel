//! Native virtual highlight dispatch uses the actual BRep owner and its lazily created presentable child.
use super::{StdSelectBRepOwner, StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, v3d::V3dViewer};
use rustycad_tkernel::standard::{Failure, Handle, raise_native_failure};
use rustycad_tkmath::toploc::Location;
impl SelectMgrEntityOwnerSource for StdSelectBRepOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        // Inherited state belongs to the actual owned BRep parent.
        &self.brep_owner_base().owner
    }
    fn is_hilighted(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) -> bool {
        let child = self.brep_owner_base().presentation.read().clone();
        let target = if child.is_null() {
            self.selectable().cast()
        } else {
            child.cast()
        };
        manager.is_highlighted(&target, brep_owner_effective_mode(self, mode))
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::brep_owner_highlight::highlight_brep_owner(self, manager, style, mode)
    }
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, _: i32) {
        let child = self.brep_owner_base().presentation.read().clone();
        let target = if child.is_null() || !self.comes_from_decomposition() {
            self.selectable().cast()
        } else {
            child.cast()
        };
        manager.unhighlight(&target);
    }
    fn clear(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) {
        let child = self.brep_owner_base().presentation.read().clone();
        if !child.is_null() {
            manager.clear(&child.cast(), brep_owner_effective_mode(self, mode));
        }
        let removed = self.brep_owner_base().presentation.write().nullify();
        let _ = removed;
    }
    fn set_location(&self, _: &Location) {
        // Native base SetLocation does nothing; only the existing child follows Location().
        let child = self.brep_owner_base().presentation.read().clone();
        if !child.is_null() {
            raise_native_failure(child.set_local_transformation(&self.location().transformation()));
        }
    }
    fn update_highlight_trsf(
        &self,
        viewer: &Handle<V3dViewer>,
        manager: &Handle<PrsMgrPresentationManager>,
        mode: i32,
    ) -> Result<(), Failure> {
        let child = self.brep_owner_base().presentation.read().clone();
        if child.is_null() && !self.has_selectable() {
            return Ok(());
        }
        manager.update_highlight_trsf(viewer, &self.selectable().cast(), mode, &child.cast())
    }
}
pub(super) fn brep_owner_effective_mode<S: StdSelectBRepOwnerSource + ?Sized>(
    owner: &S,
    mode: i32,
) -> i32 {
    if mode < 0 {
        return owner.hilight_mode();
    }
    mode
}
