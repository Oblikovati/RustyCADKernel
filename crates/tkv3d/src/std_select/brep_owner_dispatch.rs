//! Native virtual highlight dispatch uses the actual BRep owner and its lazily created presentable child.
use super::{StdSelectBRepOwner, StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, v3d::V3dViewer};
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, raise_native_failure};
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
        // Native Clear releases the retained child after the manager callback.
        self.brep_owner_base().presentation.write().nullify();
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
        viewer: &dyn HandleReference<V3dViewer>,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        mode: i32,
    ) -> Result<(), Failure> {
        let child = self.brep_owner_base().presentation.read().clone();
        if child.is_null() && !self.has_selectable() {
            return Ok(());
        }
        manager.observe().update_highlight_trsf(
            &viewer.observe(),
            &self.selectable().cast(),
            mode,
            &child.cast(),
        )
    }
    fn dump_json(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
        depth: i32,
    ) -> std::io::Result<()> {
        std_select_brep_owner_dump_json(self, stream, depth)
    }
}
/// The native `StdSelect_BRepOwner::DumpJson` body: the shape, the highlight presentation
/// and the highlight mode (the owner parent is not dumped natively).
/// Example: `std_select_brep_owner_dump_json(&*owner, &mut stream, -1)?`.
pub fn std_select_brep_owner_dump_json<S: StdSelectBRepOwnerSource + ?Sized>(
    owner: &S,
    stream: &mut rustycad_tkernel::standard::dump::DumpStream,
    depth: i32,
) -> std::io::Result<()> {
    use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer};
    class_begin(stream, "StdSelect_BRepOwner")?;
    let shape = owner.shape_ref().clone_shape();
    field_dumped(stream, depth, "&myShape", &|s, d| shape.dump_json(s, d))?;
    let child = owner.brep_owner_base().presentation.read().clone();
    if let Some(child) = child.get() {
        field_dumped(stream, depth, "myPrsSh.get()", &|s, d| {
            child.dump_json(s, d)
        })?;
    }
    field_integer(stream, "myCurMode", owner.hilight_mode())
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
