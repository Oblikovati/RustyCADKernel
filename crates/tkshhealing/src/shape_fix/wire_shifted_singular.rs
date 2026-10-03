//! Singularity decisions precede the cyclic traversal stop update in native FixShifted.
use super::*;
use rustycad_tkg3d::geom::GeomSurface;

impl ShapeFixWire {
    pub(super) fn period_pair_singularity(
        &self,
        edges: &[Edge],
        index: usize,
        closed: (bool, bool),
    ) -> Result<Option<(Point3, i32)>, Failure> {
        let vertex = ShapeAnalysisEdge::new().first_vertex(&edges[index]);
        if vertex.is_null() {
            return Ok(None);
        }
        let point = BRepTool::pnt(&vertex)?;
        let axis = self.singular_axis_for_edge(&edges[index])?;
        if axis != 0 {
            return Ok(Some((point, axis)));
        }
        let previous = &edges[(index + edges.len() - 1) % edges.len()];
        Ok(self
            .near_degenerate_axis(previous, closed)?
            .map(|axis| (point, axis)))
    }

    fn near_degenerate_axis(
        &self,
        previous: &Edge,
        closed: (bool, bool),
    ) -> Result<Option<i32>, Failure> {
        let analysis = self.analyzer_for_shifted()?.surface();
        if !analysis.surface().is_kind_name("Geom_SurfaceOfRevolution") || (closed.0 && closed.1) {
            return Ok(Some(0));
        }
        let face = self.analyzer_for_shifted()?.face();
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(previous, &face, true)?;
        let Some(curve) = curve.get() else {
            return Ok(None);
        };
        Ok(Some(self.classify_revolution_boundaries(
            &analysis, curve, first, last, closed,
        )?))
    }

    fn singular_axis_for_edge(&self, edge: &Edge) -> Result<i32, Failure> {
        let vertex = ShapeAnalysisEdge::new().first_vertex(edge);
        if vertex.is_null() {
            return Ok(0);
        }
        let point = BRepTool::pnt(&vertex)?;
        let tolerance = BRepTool::vertex_tolerance(&vertex)?.max(self.precision());
        let analysis = self.analyzer_for_shifted()?.surface();
        let Some(singularity) = analysis.degenerated_values(point, tolerance, true)? else {
            return Ok(0);
        };
        let du = (singularity.first_uv.x() - singularity.last_uv.x()).abs();
        let dv = (singularity.first_uv.y() - singularity.last_uv.y()).abs();
        Ok(if du > dv { 1 } else { 2 })
    }

    fn classify_revolution_boundaries(
        &self,
        analysis: &crate::shape_analysis::ShapeAnalysisSurface,
        previous: &dyn Geom2dCurve,
        first: f64,
        last: f64,
        closed: (bool, bool),
    ) -> Result<i32, Failure> {
        let previous_start = previous.value(first)?;
        let previous_end = previous.value(last)?;
        let bounds = analysis.surface().get().map(GeomSurface::bounds);
        let Some(bounds) = bounds else { return Ok(0) };
        let tolerance = self.max_tolerance();
        if !closed.1 {
            let (uf, ul, _, _) = bounds;
            let boundary = (
                rustycad_tkmath::gp::Point2::new(uf, previous_end.y()),
                rustycad_tkmath::gp::Point2::new(ul, previous_end.y()),
            );
            if analysis.is_degenerated_segment(boundary.0, boundary.1, tolerance, 10.)?
                && !analysis.is_degenerated_segment(previous_start, previous_end, tolerance, 10.)?
            {
                return Ok(1);
            }
        }
        if !closed.0 {
            let (_, _, vf, vl) = bounds;
            let boundary = (
                rustycad_tkmath::gp::Point2::new(previous_end.x(), vf),
                rustycad_tkmath::gp::Point2::new(previous_end.x(), vl),
            );
            if analysis.is_degenerated_segment(boundary.0, boundary.1, tolerance, 10.)?
                && !analysis.is_degenerated_segment(previous_start, previous_end, tolerance, 10.)?
            {
                return Ok(2);
            }
        }
        Ok(0)
    }
}
