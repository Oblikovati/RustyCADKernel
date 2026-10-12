//! Native selection priorities and constructor-time child identities.
use super::AisPlaneTrihedron;
use crate::{
    prs_mgr::*,
    select_mgr::*,
    select3d::{Select3DSensitivePoint, Select3DSensitiveSegment},
};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
impl AisPlaneTrihedron {
    fn select_plane_segments(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let whole =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 5);
        let axes = self.plane_axis_geometry()?;
        for (index, (origin, direction, length)) in axes.into_iter().enumerate() {
            let owner = if mode == 0 {
                whole.clone()
            } else {
                SelectMgrEntityOwner::with_selectable(&self.children.read()[index + 1], 7)
            };
            let last = Point3::from(origin.xyz() + direction.xyz() * length);
            let segment = Select3DSensitiveSegment::new(&owner.cast(), origin, last);
            selection.observe().add(&Handle::new(segment).cast())?;
        }
        Ok(())
    }
}
impl SelectMgrSelectableObject for AisPlaneTrihedron {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        match mode {
            0 | 2 => self.select_plane_segments(&selection.observe(), mode),
            1 => self.select_plane_origin(selection),
            -1 => {
                selection.observe().clear();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl AisPlaneTrihedron {
    fn select_plane_origin(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
    ) -> Result<(), Failure> {
        let owner = SelectMgrEntityOwner::with_selectable(&self.children.read()[0], 8);
        let point = Select3DSensitivePoint::new(&owner.cast(), self.component().pln().location());
        selection.observe().add(&Handle::new(point).cast())
    }
}
