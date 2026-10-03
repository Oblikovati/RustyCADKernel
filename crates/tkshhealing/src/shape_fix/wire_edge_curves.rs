//! Ordered curve and p-curve maintenance from `ShapeFix_Wire::FixEdgeCurves`.
#[path = "wire_edge_curves/singularity.rs"]
mod singularity;
use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};

impl ShapeFixWire {
    /// Runs native edge-curve repairs in upstream order; `fixer.fix_edge_curves()?`.
    pub fn fix_edge_curves(&self) -> Result<bool, Failure> {
        self.set_edge_curves_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() {
            return Ok(false);
        }
        self.run_edge_curve_passes()
    }

    /// Reports the aggregate flags from the last `FixEdgeCurves` call.
    /// Example: `fixer.status_edge_curves(ShapeExtendStatus::Done3)`.
    pub fn status_edge_curves(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().edge_curves_status, status)
    }

    /// Returns the shared `ShapeFix_Edge` tool used by curve repair.
    /// Example: `let edge_tool = fixer.fix_edge_tool()`.
    pub fn fix_edge_tool(&self) -> Handle<crate::shape_fix::ShapeFixEdge> {
        self.edge_fixer.clone()
    }

    fn run_edge_curve_passes(&self) -> Result<bool, Failure> {
        let analyzer = self.working_analyzer();
        let ready = analyzer.is_ready();
        let wire_data = analyzer.wire_data();
        let mut count = wire_data.nb_edges();
        let face = analyzer.face();
        self.run_reversed_2d(&wire_data, &face, count, ready)?;
        self.run_remove_pcurves(&wire_data, &face, count, ready)?;
        self.run_add_pcurves(&wire_data, &face, &analyzer.surface(), &mut count, ready)?;
        self.run_remove_3d_curves(&wire_data, &count, ready)?;
        self.run_add_3d_curves(&wire_data, &mut count)?;
        self.run_seams(&count, ready)?;
        self.run_shifted(ready)?;
        self.run_same_parameter(&wire_data, &face, count, ready)?;
        self.run_vertex_tolerances(&wire_data, &face, count)?;
        Ok(self.status_edge_curves(ShapeExtendStatus::Done))
    }

    fn run_reversed_2d(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        count: i32,
        ready: bool,
    ) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().reversed_2d_mode, true) {
            return Ok(());
        }
        for index in 1..=count {
            self.edge_fixer.fix_reversed_2d(&data.edge(index)?, face)?;
            self.record_edge_tool_status(ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1);
        }
        Ok(())
    }

    fn run_remove_pcurves(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        count: i32,
        ready: bool,
    ) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().remove_pcurve_mode, false) {
            return Ok(());
        }
        for index in 1..=count {
            self.edge_fixer
                .fix_remove_pcurve(&data.edge(index)?, face)?;
            self.record_edge_tool_status(ShapeExtendStatus::Done2, ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn run_add_pcurves(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        analysis: &Handle<ShapeAnalysisSurface>,
        count: &mut i32,
        ready: bool,
    ) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().add_pcurve_mode, true) {
            return Ok(());
        }
        let mut overdegenerated = 0;
        let mut index = 1;
        while index <= *count {
            let edge = data.edge(index)?;
            let seam = data.is_seam(index);
            self.edge_fixer.fix_add_pcurve_on_face_with_analysis(
                &edge,
                face,
                seam,
                analysis,
                self.precision(),
            )?;
            self.record_edge_tool_status(ShapeExtendStatus::Done3, ShapeExtendStatus::Fail3);
            if !seam && self.edge_fixer.status_flag(ShapeExtendStatus::Done2) {
                if self.split_edge_at_surface_singularities(data, index, face)? {
                    *count = data.nb_edges();
                    index -= 1;
                } else {
                    overdegenerated = index;
                }
            }
            index += 1;
        }
        if overdegenerated > 0 {
            self.repair_overdegenerated_pcurve(data, overdegenerated, face)?;
        }
        Ok(())
    }

    fn run_remove_3d_curves(
        &self,
        data: &ShapeExtendWireData,
        count: &i32,
        ready: bool,
    ) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().remove_curve_3d_mode, false) {
            return Ok(());
        }
        for index in 1..=*count {
            self.edge_fixer.fix_remove_curve_3d(&data.edge(index)?)?;
            self.record_edge_tool_status(ShapeExtendStatus::Done4, ShapeExtendStatus::Fail4);
        }
        Ok(())
    }

    fn run_add_3d_curves(
        &self,
        data: &ShapeExtendWireData,
        count: &mut i32,
    ) -> Result<(), Failure> {
        if !need_fix(self.state.read().add_curve_3d_mode, true) {
            return Ok(());
        }
        let mut index = 1;
        while index <= *count {
            let edge = data.edge(index)?;
            self.edge_fixer.fix_add_curve_3d(&edge)?;
            if self.edge_fixer.status_flag(ShapeExtendStatus::Fail)
                && self.handle_incomplete_edge(data, index, count, &edge)?
            {
                index -= 1;
            }
            self.record_edge_tool_status(ShapeExtendStatus::Done5, ShapeExtendStatus::Fail5);
            index += 1;
        }
        Ok(())
    }

    fn handle_incomplete_edge(
        &self,
        data: &ShapeExtendWireData,
        index: i32,
        count: &mut i32,
        edge: &Edge,
    ) -> Result<bool, Failure> {
        if edge_has_no_pcurve_span(edge)? {
            data.remove(index)?;
            *count -= 1;
            self.add_edge_curves_status(ShapeExtendStatus::Done5);
            self.add_edge_curves_status(ShapeExtendStatus::Fail5);
            self.repair_after_removed_edge(index, *count)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn repair_after_removed_edge(&self, index: i32, count: i32) -> Result<(), Failure> {
        if index == count + 1 {
            self.fix_closed(self.precision())?;
            return Ok(());
        }
        self.fix_connected_after_removal(index, self.precision())
    }

    fn run_seams(&self, count: &i32, ready: bool) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().seam_mode, false) {
            return Ok(());
        }
        for index in 1..=*count {
            self.fix_seam(index)?;
            self.record_last_fix_status(ShapeExtendStatus::Done6, ShapeExtendStatus::Fail6);
        }
        Ok(())
    }

    fn run_shifted(&self, ready: bool) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().shifted_mode, true) {
            return Ok(());
        }
        self.fix_shifted()?;
        self.record_last_fix_status(ShapeExtendStatus::Done7, ShapeExtendStatus::Fail7);
        Ok(())
    }

    fn run_same_parameter(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        count: i32,
        ready: bool,
    ) -> Result<(), Failure> {
        if !ready || !need_fix(self.state.read().same_parameter_mode, true) {
            return Ok(());
        }
        for index in 1..=count {
            let edge = data.edge(index)?;
            self.repair_invalid_pcurve_range(&edge, face, data.is_seam(index))?;
            self.edge_fixer
                .fix_same_parameter_on_face_default(&edge, face)?;
            self.record_edge_tool_status(ShapeExtendStatus::Done8, ShapeExtendStatus::Fail8);
        }
        Ok(())
    }

    fn repair_invalid_pcurve_range(
        &self,
        edge: &Edge,
        face: &Face,
        seam: bool,
    ) -> Result<(), Failure> {
        let mut analysis = ShapeAnalysisEdge::new();
        if !analysis.has_pcurve_on_face(edge, face)? {
            return Ok(());
        }
        let (pcurve, first, last) = analysis.pcurve_on_face(edge, face, false)?;
        let (_, curve_first, curve_last) = BRepTool::curve(edge)?;
        if range_differs(curve_first, curve_last, first, last) {
            BRepBuilder::default().same_range(edge, false)?;
            return Ok(());
        }
        if analysis.check_pcurve_range(curve_first, curve_last, &pcurve)? {
            return Ok(());
        }
        self.rebuild_pcurve(edge, face, seam)
    }

    fn rebuild_pcurve(&self, edge: &Edge, face: &Face, seam: bool) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        ShapeBuildEdge.remove_pcurve_on_located_surface(edge, &surface, &location)?;
        let analysis = self.working_analyzer().surface();
        self.edge_fixer.fix_add_pcurve_on_face_with_analysis(
            edge,
            face,
            seam,
            &analysis,
            self.precision(),
        )?;
        self.record_edge_tool_status(ShapeExtendStatus::Done3, ShapeExtendStatus::Fail3);
        Ok(())
    }

    fn run_vertex_tolerances(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        count: i32,
    ) -> Result<(), Failure> {
        if !need_fix(self.state.read().vertex_tolerance_mode, true) {
            return Ok(());
        }
        for index in 1..=count {
            self.edge_fixer
                .fix_vertex_tolerance_on_face(&data.edge(index)?, face)?;
            self.record_edge_tool_status(ShapeExtendStatus::Done8, ShapeExtendStatus::Fail8);
        }
        if !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(())
    }

    fn record_edge_tool_status(&self, done: ShapeExtendStatus, failed: ShapeExtendStatus) {
        if self.edge_fixer.status_flag(ShapeExtendStatus::Done) {
            self.add_edge_curves_status(done);
        }
        if self.edge_fixer.status_flag(ShapeExtendStatus::Fail) {
            self.add_edge_curves_status(failed);
        }
    }

    fn record_last_fix_status(&self, done: ShapeExtendStatus, failed: ShapeExtendStatus) {
        if self.last_fix_status(ShapeExtendStatus::Done) {
            self.add_edge_curves_status(done);
        }
        if self.last_fix_status(ShapeExtendStatus::Fail) {
            self.add_edge_curves_status(failed);
        }
    }

    fn set_edge_curves_status(&self, status: ShapeExtendStatus) {
        self.state.write().edge_curves_status = ShapeExtend::encode_status(status);
    }

    fn add_edge_curves_status(&self, status: ShapeExtendStatus) {
        self.state.write().edge_curves_status |= ShapeExtend::encode_status(status);
    }
}

fn need_fix(mode: i32, default: bool) -> bool {
    if mode < 0 { default } else { mode > 0 }
}

fn edge_has_no_pcurve_span(edge: &Edge) -> Result<bool, Failure> {
    let found = BRepTool::first_curve_on_surface(edge)?;
    Ok(found.pcurve.is_null() || (found.last - found.first).abs() < precision::p_confusion())
}

fn range_differs(first: f64, last: f64, pc_first: f64, pc_last: f64) -> bool {
    (first - pc_first).abs() > precision::p_confusion()
        || (last - pc_last).abs() > precision::p_confusion()
}

#[cfg(test)]
#[path = "wire_edge_curves_tests.rs"]
mod tests;
