//! Small-edge classification, preserving OCCT's combined DONE and FAIL states.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkmath::gp::Point3;

impl ShapeAnalysisWire {
    /// Accumulate small-edge statuses without resetting them; `analyzer.check_small(1e-6)?`.
    pub fn check_small(&self, tolerance: f64) -> Result<bool, Failure> {
        for index in 1..=self.wire_data().nb_edges() {
            self.check_small_at(index, tolerance)?;
            let mut fields = self.fields.write();
            fields.status_small |= fields.status;
        }
        Ok(self.status_small(ShapeExtendStatus::Done))
    }

    /// Check one edge, with zero selecting the last; `analyzer.check_small_at(1, 1e-6)?`.
    pub fn check_small_at(&self, index: i32, tolerance: f64) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() || self.nb_edges() <= 1 {
            return Ok(false);
        }
        let selected = if index != 0 { index } else { self.nb_edges() };
        let edge = self.wire_data().edge(selected)?;
        if self.small_degenerated_guard(&edge)? {
            return Ok(false);
        }
        self.classify_small_vertices(&edge, tolerance)
    }

    /// Query accumulated small-edge flags; `analyzer.status_small(ShapeExtendStatus::Done)`.
    pub fn status_small(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_small, status)
    }

    fn small_degenerated_guard(&self, edge: &Edge) -> Result<bool, Failure> {
        if !BRepTool::degenerated(edge)? {
            return Ok(false);
        }
        let face = self.face();
        if !face.is_null() && ShapeAnalysisEdge::new().has_pcurve_on_face(edge, &face)? {
            return Ok(true);
        }
        // OCCT ma-test5.igs (1999-01-22): degenerate edges without pcurves are removable.
        self.set_last_status(ShapeExtendStatus::Fail1);
        Ok(false)
    }

    fn classify_small_vertices(&self, edge: &Edge, tolerance: f64) -> Result<bool, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first = analysis.first_vertex(edge);
        let last = analysis.last_vertex(edge);
        if first.is_null() || last.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        let pfirst = BRepTool::pnt(&first)?;
        let plast = BRepTool::pnt(&last)?;
        if pfirst.distance(&plast) > tolerance {
            return Ok(false);
        }
        if !self.small_midpoint_fits(edge, pfirst, plast, tolerance)? {
            return Ok(false);
        }
        self.mark_small_done(first.is_same(&last));
        Ok(true)
    }

    fn small_midpoint_fits(
        &self,
        edge: &Edge,
        first: Point3,
        last: Point3,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let midpoint = self.small_edge_midpoint(edge, first)?;
        // Preserve native comparisons for non-finite coordinates and tolerances.
        Ok(!(midpoint.distance(&first) > tolerance || midpoint.distance(&last) > tolerance))
    }

    fn mark_small_done(&self, same_vertex: bool) {
        let done = if same_vertex {
            ShapeExtendStatus::Done1
        } else {
            ShapeExtendStatus::Done2
        };
        self.fields.write().status |= ShapeExtend::encode_status(done);
    }

    fn small_edge_midpoint(&self, edge: &Edge, fallback: Point3) -> Result<Point3, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (curve, first, last) = analysis.curve_3d(edge, false)?;
        if !curve.is_null() {
            return curve.value((first + last) / 2.);
        }
        let face = self.face();
        let surface = self.surface();
        if !face.is_null() && !surface.is_null() {
            let (pcurve, first, last) = analysis.pcurve_on_face(edge, &face, false)?;
            if !pcurve.is_null() {
                return surface.value_uv(pcurve.value((first + last) / 2.)?);
            }
        }
        self.set_last_status(ShapeExtendStatus::Fail1);
        Ok(fallback)
    }
}
