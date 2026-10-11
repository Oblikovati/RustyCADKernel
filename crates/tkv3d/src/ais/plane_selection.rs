//! Selection ignores its mode and uses cached geometry, not freshly computed fields.
use super::AisPlane;
use crate::{
    prs_mgr::*,
    select_mgr::*,
    select3d::{Select3DSensitiveTriangulation, Select3DTypeOfSensitivity},
};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::{
    poly::{Triangle, Triangulation},
    toploc::Location,
};
impl AisPlane {
    fn plane_selection_mesh(&self) -> Result<Handle<Triangulation>, Failure> {
        let f = self.fields.read().clone();
        let (points, triangles) = if f.is_xyz_plane {
            (vec![f.center, f.pmin, f.pmax], vec![Triangle::new(1, 2, 3)])
        } else {
            let pa = self.attributes().plane_aspect();
            let x = pa.plane_x_length() / 2.;
            let y = pa.plane_y_length() / 2.;
            let plane = self.centered_plane();
            let points = [(x, y), (x, -y), (-x, -y), (-x, y)]
                .into_iter()
                .map(|(u, v)| plane.d0(u, v))
                .collect::<Result<Vec<_>, _>>()?;
            (points, vec![Triangle::new(1, 2, 3), Triangle::new(1, 3, 4)])
        };
        Ok(Handle::new(Triangulation::from_nodes(
            &Array1::from_shared(&points, 1),
            &Array1::from_shared(&triangles, 1),
        )?))
    }
}
impl SelectMgrSelectableObject for AisPlane {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        selection.clear();
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 10);
        let mesh = self.plane_selection_mesh()?;
        let sensitive = Handle::new(Select3DSensitiveTriangulation::new(
            &owner.cast(),
            &mesh,
            &Location::default(),
            self.type_of_sensitivity() == Select3DTypeOfSensitivity::Interior,
        )?);
        selection.add(&sensitive.cast())
    }
}
