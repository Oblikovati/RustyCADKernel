//! Seam orientation selection with native output-reference update order.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;

impl ShapeAnalysisWire {
    /// Detect swapped seam pcurves; `analyzer.check_seam_at(1)?`.
    pub fn check_seam_at(&self, index: i32) -> Result<bool, Failure> {
        self.check_seam_into(
            index,
            &mut Handle::default(),
            &mut Handle::default(),
            &mut (0., 0.),
        )
    }

    /// Return seam pcurves and the last queried range, preserving outputs on early exits.
    /// Example: `analyzer.check_seam_into(1, &mut first, &mut second, &mut range)?`.
    pub fn check_seam_into(
        &self,
        index: i32,
        first: &mut Handle<dyn Geom2dCurve>,
        second: &mut Handle<dyn Geom2dCurve>,
        range: &mut (f64, f64),
    ) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let selected = if index == 0 { self.nb_edges() } else { index };
        let edge = self.wire_data().edge(selected)?;
        if !ShapeAnalysisEdge::new().is_seam_on_face(&edge, &self.face())? {
            return Ok(false);
        }
        self.extract_seam_curves(&edge, first, second, range)?;
        self.classify_seam_curves(first, second)
    }

    fn extract_seam_curves(
        &self,
        edge: &rustycad_tkbrep::topods::Edge,
        first: &mut Handle<dyn Geom2dCurve>,
        second: &mut Handle<dyn Geom2dCurve>,
        range: &mut (f64, f64),
    ) -> Result<(), Failure> {
        let mut face = self.face();
        face.set_orientation(Orientation::Forward);
        let mut oriented = edge.clone();
        oriented.set_orientation(Orientation::Forward);
        let found = BRepTool::curve_on_face(&oriented, &face)?;
        *first = found.pcurve;
        *range = (found.first, found.last);
        oriented.set_orientation(Orientation::Reversed);
        let found = BRepTool::curve_on_face(&oriented, &face)?;
        *second = found.pcurve;
        *range = (found.first, found.last);
        Ok(())
    }

    fn classify_seam_curves(
        &self,
        first: &Handle<dyn Geom2dCurve>,
        second: &Handle<dyn Geom2dCurve>,
    ) -> Result<bool, Failure> {
        if first.is_null() || second.is_null() {
            return Ok(false);
        }
        if ShapeAnalysisCurve.select_forward_seam(first, second)? != 2 {
            return Ok(false);
        }
        self.set_last_status(ShapeExtendStatus::Done1);
        Ok(true)
    }
}
