//! Ordered native edge consistency checks and their seven aggregate status pairs.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkbrep::topods::Edge;

impl ShapeAnalysisWire {
    /// Check every edge's geometry and boundary gaps; `analyzer.check_edge_curves()?`.
    pub fn check_edge_curves(&self) -> Result<bool, Failure> {
        self.fields.write().status_edge_curves = 0;
        if !self.is_ready() {
            return Ok(false);
        }
        let mut analysis = ShapeAnalysisEdge::new();
        for index in 1..=self.wire_data().nb_edges() {
            let edge = self.wire_data().edge(index)?;
            self.check_edge_endpoints(&mut analysis, &edge)?;
            self.check_edge_seam_and_gaps(index)?;
            analysis.check_same_parameter_into(&self.wire_data().edge(index)?, 23, &mut 0.)?;
            self.record_edge_analysis(
                &analysis,
                ShapeExtendStatus::Done7,
                ShapeExtendStatus::Fail7,
            );
        }
        Ok(self.status_edge_curves(ShapeExtendStatus::Done))
    }

    /// Query aggregate edge consistency flags; `analyzer.status_edge_curves(ShapeExtendStatus::Fail)`.
    pub fn status_edge_curves(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_edge_curves, status)
    }

    fn check_edge_endpoints(
        &self,
        analysis: &mut ShapeAnalysisEdge,
        edge: &Edge,
    ) -> Result<(), Failure> {
        analysis.check_curve_3d_with_pcurve_on_face(edge, &self.face())?;
        self.record_edge_analysis(analysis, ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1);
        analysis.check_vertices_with_pcurve_on_face(edge, &self.face(), -1., 0)?;
        self.record_edge_analysis(analysis, ShapeExtendStatus::Done2, ShapeExtendStatus::Fail2);
        analysis.check_vertices_with_curve_3d(edge, -1., 0)?;
        self.record_edge_analysis(analysis, ShapeExtendStatus::Done3, ShapeExtendStatus::Fail3);
        Ok(())
    }

    fn check_edge_seam_and_gaps(&self, index: i32) -> Result<(), Failure> {
        self.check_seam_at(index)?;
        self.record_edge_last_status(ShapeExtendStatus::Done4, ShapeExtendStatus::Fail4);
        self.check_gap_3d_at(index)?;
        self.record_edge_last_status(ShapeExtendStatus::Done5, ShapeExtendStatus::Fail5);
        self.check_gap_2d_at(index)?;
        self.record_edge_last_status(ShapeExtendStatus::Done6, ShapeExtendStatus::Fail6);
        Ok(())
    }

    fn record_edge_analysis(
        &self,
        analysis: &ShapeAnalysisEdge,
        done: ShapeExtendStatus,
        fail: ShapeExtendStatus,
    ) {
        self.record_edge_flags(
            analysis.status_flag(ShapeExtendStatus::Done),
            analysis.status_flag(ShapeExtendStatus::Fail),
            done,
            fail,
        );
    }

    fn record_edge_last_status(&self, done: ShapeExtendStatus, fail: ShapeExtendStatus) {
        self.record_edge_flags(
            self.last_check_status(ShapeExtendStatus::Done),
            self.last_check_status(ShapeExtendStatus::Fail),
            done,
            fail,
        );
    }

    fn record_edge_flags(
        &self,
        is_done: bool,
        is_fail: bool,
        done: ShapeExtendStatus,
        fail: ShapeExtendStatus,
    ) {
        let mut fields = self.fields.write();
        if is_done {
            fields.status_edge_curves |= ShapeExtend::encode_status(done);
        }
        if is_fail {
            fields.status_edge_curves |= ShapeExtend::encode_status(fail);
        }
    }
}
