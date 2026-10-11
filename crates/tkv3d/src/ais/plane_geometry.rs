//! Preserve native cached triangles and the shaded/wireframe origin asymmetry.
use super::{AisPlane, AisTypeOfPlane};
use crate::{prs_mgr::*, prs3d::Prs3dDatumParts};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomElementarySurface, GeomPlane, GeomSurface};
use rustycad_tkmath::{elslib, gp::Point3};
impl AisPlane {
    pub(super) fn compute_fields(&self) -> Result<(), Failure> {
        if !self.is_xyz_plane() {
            return Ok(());
        }
        let snapshot = self.fields.read().clone();
        let frame = snapshot.ax2.ax2()?;
        let mut f = self.fields.write();
        f.center = frame.location();
        f.pmin = Point3::default();
        f.pmax = Point3::default();
        let Some((first, second, normal)) = coordinate_plane_axes(snapshot.type_of_plane) else {
            return Ok(());
        };
        f.component = Handle::new(GeomPlane::from_coefficients(
            normal[0], normal[1], normal[2], 0.,
        )?);
        let [a, b] = self.xyz_endpoints(&frame, first, second);
        f.pmin = a;
        f.pmax = b;
        Ok(())
    }
    fn xyz_endpoints(
        &self,
        frame: &rustycad_tkmath::gp::Axis2,
        first: usize,
        second: usize,
    ) -> [Point3; 2] {
        let dirs = [frame.x_direction(), frame.y_direction(), frame.direction()];
        let parts = [
            Prs3dDatumParts::XAxis,
            Prs3dDatumParts::YAxis,
            Prs3dDatumParts::ZAxis,
        ];
        let datum = self.attributes().datum_aspect();
        [first, second].map(|axis| {
            Point3::from(frame.location().xyz() + dirs[axis].xyz() * datum.axis_length(parts[axis]))
        })
    }
    pub(super) fn centered_plane(&self) -> GeomPlane {
        let f = self.fields.read();
        let plane = GeomPlane::from_pln(f.component.pln());
        plane.set_location(f.center);
        plane
    }
    pub(super) fn compute_frame(&self) -> Result<(), Failure> {
        let f = self.fields.read().clone();
        if f.automatic_position {
            let (u, v) = elslib::parameters(&f.component.pln(), f.center);
            self.fields.write().center = f.component.d0(u, v)?;
            return Ok(());
        }
        let (u, v) = elslib::parameters(&self.centered_plane().pln(), f.pmin);
        let x = manual_plane_extent(u);
        let y = manual_plane_extent(v);
        self.set_size_xy(x, y)?;
        self.attributes().plane_aspect().set_plane_length(x, y);
        Ok(())
    }
}
fn coordinate_plane_axes(kind: AisTypeOfPlane) -> Option<(usize, usize, [f64; 3])> {
    match kind {
        AisTypeOfPlane::XYPlane => Some((0, 1, [0., 0., 1.])),
        AisTypeOfPlane::XZPlane => Some((0, 2, [0., 1., 0.])),
        AisTypeOfPlane::YZPlane => Some((1, 2, [1., 0., 0.])),
        AisTypeOfPlane::Unknown => None,
    }
}
fn manual_plane_extent(parameter: f64) -> f64 {
    let extent = 2.4 * parameter.abs();
    if extent < 10. * confusion() {
        0.1
    } else {
        extent
    }
}

use rustycad_tkernel::precision::confusion;
