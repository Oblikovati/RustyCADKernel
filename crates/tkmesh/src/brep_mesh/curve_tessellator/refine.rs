//! Native pcurve midpoint-to-chord refinement, depth limited to ten subdivisions.
use super::*;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::Point2;
impl BRepMeshCurveTessellator {
    pub(super) fn split_by_deflection_2d(&mut self, edge: &EdgeHandle) -> Result<(), Failure> {
        let count = self.distribution.nb_points();
        if edge.is_free() || !edge.get_same_param() || !edge.get_same_range() || count <= 1 {
            return Ok(());
        }
        for index in 0..edge.pcurves_nb() {
            self.refine_face_pcurve(&edge.get_pcurve(index)?, count)?;
        }
        Ok(())
    }
    fn refine_face_pcurve(
        &mut self,
        pcurve: &crate::imesh_data::PCurveHandle,
        count: i32,
    ) -> Result<(), Failure> {
        let face = pcurve.get_face().upgrade().get_face()?;
        let (surface, _) = BRepTool::surface_with_location(&face)?;
        if surface.is_instance(GeomPlane::static_type()) {
            return Ok(());
        }
        let oriented = topods::edge(&self.edge.oriented(pcurve.get_orientation()))?;
        let curve = BRepTool::curve_on_face(&oriented, &face)?.pcurve;
        let parameters: Vec<f64> = (1..=count)
            .map(|index| self.distribution.parameter(index))
            .collect::<Result<_, _>>()?;
        for segment in parameters.windows(2) {
            self.split_segment(&surface, &curve, segment[0], segment[1], 1)?;
        }
        Ok(())
    }
    fn split_segment(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        depth: i32,
    ) -> Result<(), Failure> {
        if depth > 10 || (last - first).abs() < 2.0 * precision::p_confusion() {
            return Ok(());
        }
        if curve.first_parameter() - first > precision::p_confusion()
            || last - curve.last_parameter() > precision::p_confusion()
        {
            return Ok(());
        }
        let first_uv = curve.value(first)?;
        let last_uv = curve.value(last)?;
        if !self.needs_midpoint(surface, first_uv, last_uv)? {
            return Ok(());
        }
        let middle = (first + last) * 0.5;
        self.distribution
            .add_point(self.adaptor.d0(middle)?, middle, false)?;
        self.split_segment(surface, curve, first, middle, depth + 1)?;
        self.split_segment(surface, curve, middle, last, depth + 1)
    }
    fn needs_midpoint(
        &self,
        surface: &Handle<dyn GeomSurface>,
        first: Point2,
        last: Point2,
    ) -> Result<bool, Failure> {
        let first_point = surface.value(first.x(), first.y())?;
        let last_point = surface.value(last.x(), last.y())?;
        if first_point.square_distance(&last_point) < self.square_min_size {
            return Ok(false);
        }
        let middle = Point2::from((first.xy() + last.xy()) * 0.5);
        let offset = surface.value(middle.x(), middle.y())?.xyz() - first_point.xyz();
        if offset.square_modulus() < self.square_min_size {
            return Ok(false);
        }
        let mut chord = last_point.xyz() - first_point.xyz();
        chord.normalize()?;
        let projection = chord * offset.dot(chord);
        // Keep the negated comparison: a NaN distance does not suppress the native subdivision.
        Ok((offset - projection)
            .square_modulus()
            .partial_cmp(&self.square_edge_deflection)
            != Some(std::cmp::Ordering::Less))
    }
}
