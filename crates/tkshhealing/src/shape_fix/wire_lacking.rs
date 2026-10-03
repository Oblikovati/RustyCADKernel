// Rust translation of the single-edge and bulk ShapeFix_Wire::FixLacking overloads.

//! Repair missing p-curve connections by tolerance growth or a replacement edge.
#[path = "wire_lacking_context.rs"]
mod context;
#[path = "wire_lacking_types.rs"]
mod types;
mod wire_lacking_bending;

use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::ShapeAnalysisWire;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Builder, Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

use types::{
    LackingPair, LongGap, average_vertex_point, lacking_curve_endpoints, missing_line,
    pair_surface_midpoint,
};

impl ShapeFixWire {
    /// `FixLacking(num, force)`: repair one analyzed p-curve gap.
    /// Example: `let repaired = fixer.fix_lacking_at(2, false)?;`.
    pub fn fix_lacking_at(&self, index: i32, force: bool) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let tolerance = if force { self.precision() } else { 0.0 };
        let (_, first, last) = analyzer.check_lacking_at(index, tolerance)?;
        self.transfer_lacking_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.repair_lacking_pair(&analyzer, index, first, last)
    }

    /// `FixLacking(force)`: process all eligible adjacent edge pairs.
    /// Example: `let changed = fixer.fix_lacking(false)?;`.
    pub fn fix_lacking(&self, force: bool) -> Result<bool, Failure> {
        self.state.write().lacking_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let start = if self.closed_mode() { 1 } else { 2 };
        for index in start..=analyzer.nb_edges() {
            self.fix_lacking_at(index, force)?;
            let last_status = self.state.read().last_fix_status;
            self.state.write().lacking_status |= last_status;
        }
        Ok(self.status_lacking(ShapeExtendStatus::Done))
    }

    fn repair_lacking_pair(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        first: Point2,
        last: Point2,
    ) -> Result<bool, Failure> {
        let pair = LackingPair::load(analyzer, index, first, last)?;
        let needed = analyzer.max_distance_3d();
        let candidates =
            self.bending_candidates(&pair.previous, &pair.current, &pair.face, first, last)?;
        if self.bend_is_preferred(&pair, candidates.as_ref(), needed)? {
            let (left, right) = candidates.as_ref().expect("preferred bend has candidates");
            return self.apply_bent_pcurves(&pair.previous, &pair.current, &pair.face, left, right);
        }
        self.repair_pair_without_bend(&pair, analyzer, candidates.as_ref(), needed)
    }

    fn bend_is_preferred(
        &self,
        pair: &LackingPair,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
    ) -> Result<bool, Failure> {
        let Some((left, right)) = candidates else {
            return Ok(false);
        };
        if self.bent_curves_improve_pair(pair, left, right)? {
            return Ok(true);
        }
        Ok(needed < self.precision() && left.tolerance < needed && right.tolerance < needed)
    }

    fn bent_curves_improve_pair(
        &self,
        pair: &LackingPair,
        left: &wire_lacking_bending::BentPCurve,
        right: &wire_lacking_bending::BentPCurve,
    ) -> Result<bool, Failure> {
        let left_tolerance = BRepTool::edge_tolerance(&pair.previous)?;
        let right_tolerance = BRepTool::edge_tolerance(&pair.current)?;
        Ok(left.tolerance < left_tolerance && right.tolerance < right_tolerance)
    }

    fn repair_pair_without_bend(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
    ) -> Result<bool, Failure> {
        if needed < self.precision() {
            return self.increase_lacking_tolerance(&pair.first_vertex, &pair.last_vertex, needed);
        }
        let tolerance = pair.vertex_tolerance()?;
        if self.can_raise_lacking_tolerance(pair, analyzer, needed, tolerance)? {
            return self.try_bend_or_raise(pair, candidates, needed);
        }
        self.finish_lacking_edge_choice(pair, analyzer, candidates, needed, tolerance)
    }

    fn try_bend_or_raise(
        &self,
        pair: &LackingPair,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
    ) -> Result<bool, Failure> {
        if let Some((left, right)) = candidates
            && left.tolerance < needed
            && right.tolerance < needed
        {
            return self.apply_bent_pcurves(&pair.previous, &pair.current, &pair.face, left, right);
        }
        self.increase_lacking_tolerance(&pair.first_vertex, &pair.last_vertex, needed)
    }

    fn can_raise_lacking_tolerance(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        needed: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if !pair.both_regular()? || needed <= tolerance || needed >= self.max_tolerance() {
            return Ok(false);
        }
        analyzer
            .surface()
            .is_degenerated_segment(pair.first, pair.last, 2.0 * tolerance, 10.0)
            .map(|is_degenerate| !is_degenerate)
    }

    fn maybe_insert_midpoint_edge(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if pair.both_regular()? && self.classify_midpoint_edge(pair, analyzer, tolerance)? {
            let distance = self.midpoint_gap(pair)?;
            if !self.topology_mode() && distance > tolerance {
                self.increase_lacking_tolerance(&pair.first_vertex, &pair.last_vertex, distance)?;
            }
            return Ok(true);
        }
        self.finish_without_edge(pair, candidates, needed, tolerance)
    }

    fn classify_midpoint_edge(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if analyzer.surface().is_degenerated_segment(
            pair.first,
            pair.last,
            2.0 * tolerance,
            10.0,
        )? {
            return self.insert_classified_edge(pair, tolerance);
        }
        if analyzer.max_distance_3d() < self.max_tolerance() {
            return Ok(false);
        }
        self.insert_classified_edge(pair, tolerance)
    }

    fn insert_classified_edge(&self, pair: &LackingPair, tolerance: f64) -> Result<bool, Failure> {
        let distance = self.midpoint_gap(pair)?;
        let degenerated =
            distance <= tolerance || (!self.topology_mode() && distance <= self.max_tolerance());
        let closed = self.topology_mode() && !degenerated;
        if !degenerated && !closed {
            return Ok(false);
        }
        self.insert_lacking_edge(pair, degenerated)
    }

    fn finish_without_edge(
        &self,
        pair: &LackingPair,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if needed > tolerance && needed < self.max_tolerance() {
            return self.try_bend_or_raise(pair, candidates, needed);
        }
        self.set_last_fix_status(ShapeExtendStatus::Fail2);
        Ok(false)
    }

    fn finish_lacking_edge_choice(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        candidates: Option<&(
            wire_lacking_bending::BentPCurve,
            wire_lacking_bending::BentPCurve,
        )>,
        needed: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if let Some(changed) = self.try_long_edge(pair, analyzer)? {
            return Ok(changed);
        }
        self.maybe_insert_midpoint_edge(pair, analyzer, candidates, needed, tolerance)
    }

    fn try_long_edge(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
    ) -> Result<Option<bool>, Failure> {
        if !pair.both_regular()? || !self.topology_mode() {
            return Ok(None);
        }
        let Some(gap) = self.long_lacking_edge(pair, analyzer)? else {
            return Ok(self
                .last_fix_has_status(ShapeExtendStatus::Fail1)
                .then_some(false));
        };
        self.insert_long_lacking_edge(pair, gap).map(Some)
    }

    fn midpoint_gap(&self, pair: &LackingPair) -> Result<f64, Failure> {
        let vertex_midpoint = average_vertex_point(&pair.first_vertex, &pair.last_vertex)?;
        let surface_midpoint = pair_surface_midpoint(pair)?;
        Ok(vertex_midpoint.distance(&surface_midpoint))
    }

    fn long_lacking_edge(
        &self,
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
    ) -> Result<Option<LongGap>, Failure> {
        let Some((start, end)) = lacking_curve_endpoints(pair)? else {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(None);
        };
        let gap = LongGap::measure(pair, analyzer, start, end)?;
        Ok(gap
            .is_candidate(self.precision(), self.max_tolerance(), analyzer)
            .then_some(gap))
    }

    fn insert_long_lacking_edge(&self, pair: &LackingPair, gap: LongGap) -> Result<bool, Failure> {
        let vertices = self.make_long_vertices(&gap)?;
        let (pcurve, length) = missing_line(pair.first, pair.last)?;
        let Some(edge) =
            self.make_lacking_edge(&pcurve, &pair.face, &vertices.0, &vertices.1, length, false)?
        else {
            return Ok(false);
        };
        self.replace_long_neighbors(pair, &vertices.0, &vertices.1)?;
        let wire = pair
            .wire
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire lost wire data"))?;
        wire.add_edge(&edge, pair.current_index)?;
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        Ok(true)
    }

    fn make_long_vertices(&self, gap: &LongGap) -> Result<(Vertex, Vertex), Failure> {
        let builder = BRepBuilder::default();
        let mut first = Vertex::default();
        let mut last = Vertex::default();
        builder.make_vertex_at(&mut first, gap.start, 1.001 * gap.start_tolerance)?;
        builder.make_vertex_at(&mut last, gap.end, 1.001 * gap.end_tolerance)?;
        first.reverse();
        Ok((first, last))
    }

    fn insert_lacking_edge(&self, pair: &LackingPair, degenerated: bool) -> Result<bool, Failure> {
        let (pcurve, length) = missing_line(pair.first, pair.last)?;
        let Some(edge) = self.make_lacking_edge(
            &pcurve,
            &pair.face,
            &pair.first_vertex,
            &pair.last_vertex,
            length,
            degenerated,
        )?
        else {
            return Ok(false);
        };
        pair.wire
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire lost wire data"))?
            .add_edge(&edge, pair.current_index)?;
        self.record_lacking_edge_status(degenerated);
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        Ok(true)
    }

    fn record_lacking_edge_status(&self, degenerated: bool) {
        let status = if degenerated {
            ShapeExtendStatus::Done3
        } else {
            ShapeExtendStatus::Done4
        };
        self.or_last_fix_status(status);
    }

    fn make_lacking_edge(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &rustycad_tkbrep::topods::Face,
        first_vertex: &Vertex,
        last_vertex: &Vertex,
        length: f64,
        degenerated: bool,
    ) -> Result<Option<Edge>, Failure> {
        let builder = BRepBuilder::default();
        let mut edge = Edge::default();
        builder.make_edge(&mut edge)?;
        self.attach_lacking_edge_geometry(&builder, &edge, pcurve, face, length, degenerated)?;
        let edge = self.attach_lacking_edge_vertices(edge, first_vertex, last_vertex)?;
        if !degenerated && !crate::shape_build::ShapeBuildEdge.build_curve_3d(&edge) {
            self.set_last_fix_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        Ok(Some(edge))
    }

    fn attach_lacking_edge_geometry(
        &self,
        builder: &BRepBuilder,
        edge: &Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &rustycad_tkbrep::topods::Face,
        length: f64,
        degenerated: bool,
    ) -> Result<(), Failure> {
        if degenerated {
            builder.degenerated(edge, true)?;
        }
        builder.update_edge_pcurve_on_face(edge, pcurve, face, 0.0)?;
        builder.range_on_face(edge, face, 0.0, length)
    }

    fn attach_lacking_edge_vertices(
        &self,
        edge: Edge,
        first: &Vertex,
        last: &Vertex,
    ) -> Result<Edge, Failure> {
        let mut shape = edge.into_shape();
        Builder.add(&mut shape, &first.oriented(Orientation::Forward))?;
        Builder.add(&mut shape, &last.oriented(Orientation::Reversed))?;
        rustycad_tkbrep::topods::edge(&shape)
    }

    fn increase_lacking_tolerance(
        &self,
        first: &Vertex,
        last: &Vertex,
        needed: f64,
    ) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        builder.update_vertex_tolerance(first, 1.001 * needed)?;
        builder.update_vertex_tolerance(last, 1.001 * needed)?;
        self.set_last_fix_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

    fn transfer_lacking_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }
}
