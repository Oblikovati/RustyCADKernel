// Rust translation of ShapeFix_Wire::FixDegenerated at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Insert or replace wire edges that lie on a supporting-surface singularity.
use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Vector2};

impl ShapeFixWire {
    /// `FixDegenerated(num)`: mark or insert a collapsed edge at an analyzed surface singularity.
    /// `fixer.fix_degenerated_at(2)?`.
    pub fn fix_degenerated_at(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer_for_degenerated()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let (_, first, last) = analyzer.check_degenerated_at(index)?;
        self.transfer_check_failure(&analyzer);
        if analyzer.last_check_status(ShapeExtendStatus::Fail2) {
            return self.remove_invalid_degenerated(&analyzer, index);
        }
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.apply_degenerated_edge(&analyzer, index, first, last)
    }

    /// `FixDegenerated()`: process edges from the end, respecting the closed-mode boundary.
    /// `fixer.fix_degenerated()?`.
    pub fn fix_degenerated(&self) -> Result<bool, Failure> {
        self.set_degenerated_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer_for_degenerated()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        self.fix_degenerated_edges(&analyzer)
    }

    /// `StatusDegenerated(status)`: query accumulated bulk-fix flags.
    pub fn status_degenerated(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().degenerated_status, status)
    }

    /// `LastFixStatus(status)`: query flags from the most recent single-edge repair.
    /// `fixer.last_fix_status(ShapeExtendStatus::Done3)`.
    pub fn last_fix_status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().last_fix_status, status)
    }

    fn analyzer_for_degenerated(&self) -> Result<Handle<ShapeAnalysisWire>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() {
            return Err(Failure::new(
                "ShapeFix_Wire requires Init(analyzer) or Load(wire)",
            ));
        }
        Ok(analyzer)
    }

    fn fix_degenerated_edges(&self, analyzer: &ShapeAnalysisWire) -> Result<bool, Failure> {
        let mut last_coded = -1;
        let mut previous_coded = false;
        let stop = if self.state.read().closed_mode { 0 } else { 1 };
        let mut index = analyzer.nb_edges();
        while index > stop {
            self.fix_degenerated_at(index)?;
            self.accumulate_degenerated_status();
            (index, last_coded, previous_coded) =
                self.remove_duplicate_degenerated(analyzer, index, last_coded, previous_coded)?;
        }
        Ok(self.status_degenerated(ShapeExtendStatus::Done))
    }

    fn remove_duplicate_degenerated(
        &self,
        analyzer: &ShapeAnalysisWire,
        mut index: i32,
        last_coded: i32,
        previous_coded: bool,
    ) -> Result<(i32, i32, bool), Failure> {
        let coded = i32::from(self.last_fix_has_status(ShapeExtendStatus::Done2));
        let last_coded = if last_coded < 0 { coded } else { last_coded };
        let repeated = coded != 0 && (previous_coded || (index == 1 && last_coded != 0));
        if !repeated || analyzer.nb_edges() <= 1 {
            return Ok((index - 1, last_coded, coded != 0));
        }
        let data = analyzer.wire_data();
        data.remove(index)?;
        if !previous_coded {
            index = analyzer.nb_edges();
        }
        clear_degenerated_flag(&data.edge(index)?)?;
        Ok((index, last_coded, false))
    }

    fn apply_degenerated_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        first: Point2,
        last: Point2,
    ) -> Result<bool, Failure> {
        let vector = Vector2::between(first, last);
        let direction = Direction2::new(vector.x(), vector.y())?;
        let pcurve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::new(Axis2d::new(first, direction))).cast();
        let edge = self.make_degenerated_edge(analyzer, &pcurve, vector.magnitude(), index)?;
        self.commit_degenerated_edge(analyzer, index, &edge)
    }

    fn make_degenerated_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        pcurve: &Handle<dyn Geom2dCurve>,
        length: f64,
        index: i32,
    ) -> Result<Edge, Failure> {
        let mut edge = Edge::default();
        let builder = BRepBuilder::default();
        builder.make_edge(&mut edge)?;
        builder.degenerated(&edge, true)?;
        builder.update_edge_pcurve_on_face(
            &edge,
            pcurve,
            &analyzer.face(),
            rustycad_tkernel::precision::confusion(),
        )?;
        builder.range_on_face(&edge, &analyzer.face(), 0.0, length)?;
        self.add_degenerated_vertices(analyzer, &builder, &edge, index)?;
        Ok(edge)
    }

    fn add_degenerated_vertices(
        &self,
        analyzer: &ShapeAnalysisWire,
        builder: &BRepBuilder,
        edge: &Edge,
        index: i32,
    ) -> Result<(), Failure> {
        let data = analyzer.wire_data();
        let count = data.nb_edges();
        let current = if index > 0 { index } else { count };
        let previous = if current > 1 { current - 1 } else { count };
        let lack = analyzer.last_check_status(ShapeExtendStatus::Done1);
        let next = if lack {
            current
        } else if current < count {
            current + 1
        } else {
            1
        };
        let first = ShapeAnalysisEdge::new().last_vertex(&data.edge(previous)?);
        let last = ShapeAnalysisEdge::new().first_vertex(&data.edge(next)?);
        self.add_oriented_vertex(
            builder,
            edge,
            &first,
            rustycad_tkg3d::top_abs::Orientation::Forward,
        )?;
        self.add_oriented_vertex(
            builder,
            edge,
            &last,
            rustycad_tkg3d::top_abs::Orientation::Reversed,
        )
    }

    fn add_oriented_vertex(
        &self,
        builder: &BRepBuilder,
        edge: &Edge,
        vertex: &Vertex,
        orientation: rustycad_tkg3d::top_abs::Orientation,
    ) -> Result<(), Failure> {
        let oriented = topods::vertex(&vertex.clone().into_shape().oriented(orientation))?;
        let mut edge_shape = edge.clone().into_shape();
        builder.add(&mut edge_shape, &oriented.into_shape())
    }

    fn commit_degenerated_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        edge: &Edge,
    ) -> Result<bool, Failure> {
        let data = analyzer.wire_data();
        let rank = if index > 0 { index } else { data.nb_edges() };
        if analyzer.last_check_status(ShapeExtendStatus::Done1) {
            data.add_edge(edge, rank)?;
            self.or_last_fix_status(ShapeExtendStatus::Done1);
        } else {
            data.set(edge, rank)?;
            self.or_last_fix_status(ShapeExtendStatus::Done2);
        }
        Ok(true)
    }

    fn remove_invalid_degenerated(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
    ) -> Result<bool, Failure> {
        analyzer.wire_data().remove(index)?;
        self.or_last_fix_status(ShapeExtendStatus::Done3);
        Ok(true)
    }

    fn transfer_check_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail1) {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }

    pub(super) fn set_last_fix_status(&self, status: ShapeExtendStatus) {
        self.state.write().last_fix_status = ShapeExtend::encode_status(status);
    }

    pub(super) fn or_last_fix_status(&self, status: ShapeExtendStatus) {
        self.state.write().last_fix_status |= ShapeExtend::encode_status(status);
    }

    pub(super) fn last_fix_has_status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().last_fix_status, status)
    }

    fn set_degenerated_status(&self, status: ShapeExtendStatus) {
        self.state.write().degenerated_status = ShapeExtend::encode_status(status);
    }

    fn accumulate_degenerated_status(&self) {
        let last = self.state.read().last_fix_status;
        self.state.write().degenerated_status |= last;
    }
}

fn clear_degenerated_flag(edge: &Edge) -> Result<(), Failure> {
    BRepBuilder::default().degenerated(edge, false)
}
