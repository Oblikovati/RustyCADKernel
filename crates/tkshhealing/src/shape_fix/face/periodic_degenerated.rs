//! The public native apex-edge repair without Perform's subsequent seam repair.
use super::{
    FixRoot, ShapeFixFace,
    periodic_cone::{build_apex_face, conical_bounds_on_surface, face_wires},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::ReShape,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomConicalSurface, top_abs::ShapeEnum};
impl ShapeFixFace {
    /// Add the missing degenerated apex boundary to a periodic cone; `tool.fix_periodic_degenerated()?`.
    pub fn fix_periodic_degenerated(&self) -> Result<bool, Failure> {
        let face = self.apply_periodic_context()?;
        let Some(result) = periodic_apex_face(&face, self.precision())? else {
            return Ok(false);
        };
        *self.result.write() = result.clone();
        if !self.context().is_null() {
            self.context().replace(&face, &result)?;
        }
        // Native updates Result, retaining Face until the context is applied again.
        Ok(true)
    }
    fn apply_periodic_context(&self) -> Result<Face, Failure> {
        if !self.context().is_null() {
            let face = self.face()?;
            let applied = self.context().apply(&face, ShapeEnum::Shape)?;
            *self.face_state.write() = topods::face(&applied)?.into_shape();
        }
        self.face()
    }
}
fn periodic_apex_face(face: &Face, tolerance: f64) -> Result<Option<Shape>, Failure> {
    let wires = face_wires(face)?;
    let surface = BRepTool::surface(face)?;
    let cone = Handle::<GeomConicalSurface>::down_cast(&surface);
    let Some(cone) = cone.get().filter(|_| wires.len() == 1) else {
        return Ok(None);
    };
    let Some(bounds) = conical_bounds_on_surface(&wires[0], &surface, tolerance)? else {
        return Ok(None);
    };
    if cone.semi_angle().abs() <= precision::confusion() {
        return Ok(None);
    }
    let apex_v = -cone.ref_radius() / cone.semi_angle().sin();
    if (apex_v - bounds.min_v).abs() <= tolerance
        || (apex_v - bounds.max_v).abs() <= tolerance
        || (apex_v < bounds.max_v && apex_v > bounds.min_v)
    {
        return Ok(None);
    }
    build_apex_face(face, &wires[0], cone, &bounds, apex_v, false, tolerance).map(Some)
}
