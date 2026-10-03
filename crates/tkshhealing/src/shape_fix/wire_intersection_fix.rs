//! Tolerance repair of one adjacent-edge crossing after `ShapeAnalysis_Wire` detects it.
use super::{
    FixRoot, ShapeFixWire,
    wire_intersection_geometry::{crossing_action, local_deviation},
    wire_split_tool::{cut_adjacent_pair, same_vertex},
};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::Point3;

pub(super) struct CrossingState {
    pub(super) edges: [Edge; 2],
    pub(super) shared: [Vertex; 2],
    pub(super) ranges: [(f64, f64); 2],
    pub(super) forwards: [bool; 2],
    pub(super) previous_ranges: [f64; 2],
    pub(super) point: Point3,
    pub(super) tolerance: f64,
    wire_indices: [i32; 2],
    changed_edges: bool,
}

type AdjacentIntersectionRecords = (Sequence<IntersectionPoint>, Sequence<Point3>, Sequence<f64>);

pub(super) struct CrossingAction {
    pub(super) point: Point3,
    pub(super) radius: f64,
    pub(super) param1: f64,
    pub(super) param2: f64,
    pub(super) tolerance: f64,
    pub(super) improved: bool,
}

impl ShapeFixWire {
    /// Apply the non-topological `FixIntersectingEdges(index)` tolerance path.
    /// `fixer.fix_intersecting_edges_at(2)?`.
    pub(super) fn fix_intersecting_edges_at(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() || analyzer.nb_edges() < 2 {
            return Ok(false);
        }
        let (uv, xyz, errors) = adjacent_intersections(&analyzer, index)?;
        self.transfer_intersection_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        let mut state = CrossingState::load(&analyzer, index, &self.context())?;
        self.repair_adjacent_tolerances(&mut state, index, &uv, &xyz, &errors)?;
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    fn repair_adjacent_tolerances(
        &self,
        state: &mut CrossingState,
        index: i32,
        uv: &Sequence<IntersectionPoint>,
        xyz: &Sequence<Point3>,
        errors: &Sequence<f64>,
    ) -> Result<(), Failure> {
        for item in 1..=xyz.len() {
            self.repair_one_adjacent_crossing(state, index, uv, xyz, errors, item)?;
        }
        self.store_adjacent_shared_point(state)
    }

    fn repair_one_adjacent_crossing(
        &self,
        state: &mut CrossingState,
        index: i32,
        uv: &Sequence<IntersectionPoint>,
        xyz: &Sequence<Point3>,
        errors: &Sequence<f64>,
        item: usize,
    ) -> Result<(), Failure> {
        let Some(mut action) = crossing_action(state, index, uv, xyz, errors, item)? else {
            return Ok(());
        };
        if let Some((tolerance, changed)) = self.try_improve_edge_tolerance(state, &action)? {
            action.tolerance = tolerance;
            action.improved = changed;
        }
        self.apply_intersection_action(state, &action)
    }

    fn try_improve_edge_tolerance(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
    ) -> Result<Option<(f64, bool)>, Failure> {
        if action.tolerance <= state.tolerance {
            return Ok(None);
        }
        let tolerances = self.local_edge_tolerances(state, action)?;
        let maximum = tolerances.0.max(tolerances.1);
        if maximum >= self.max_tolerance() || maximum >= action.tolerance {
            return Ok(None);
        }
        let changed = self.adjacent_edge_tolerances_need_update(state, tolerances)?;
        if changed {
            self.raise_adjacent_edge_tolerances(state, tolerances)?;
            self.or_last_fix_status(ShapeExtendStatus::Done6);
        }
        Ok(Some((1.000001 * maximum, changed)))
    }

    fn adjacent_edge_tolerances_need_update(
        &self,
        state: &CrossingState,
        tolerances: (f64, f64),
    ) -> Result<bool, Failure> {
        Ok(BRepTool::edge_tolerance(&state.edges[0])? < tolerances.0
            || BRepTool::edge_tolerance(&state.edges[1])? < tolerances.1)
    }

    fn local_edge_tolerances(
        &self,
        state: &CrossingState,
        action: &CrossingAction,
    ) -> Result<(f64, f64), Failure> {
        let face = self.analyzer()?.face();
        let ends = local_deviation_ends(state);
        let first = edge_local_tolerance(state, &face, action, 0, action.param1, ends.0)?;
        let second = edge_local_tolerance(state, &face, action, 1, action.param2, ends.1)?;
        Ok((first, second))
    }

    fn raise_adjacent_edge_tolerances(
        &self,
        state: &mut CrossingState,
        tolerances: (f64, f64),
    ) -> Result<(), Failure> {
        if !self.context().is_null() {
            self.copy_adjacent_edges(state)?;
        }
        self.raise_edge_tolerance(&state.edges[0], tolerances.0)?;
        self.raise_edge_tolerance(&state.edges[1], tolerances.1)
    }

    fn copy_adjacent_edges(&self, state: &mut CrossingState) -> Result<(), Failure> {
        let context = self.context();
        let reshape = context
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire context is null"))?;
        let (first_copy, shared, last_copy) = self.copy_adjacent_vertices(state, reshape)?;
        self.replace_adjacent_edges(state, reshape, &first_copy, &shared, &last_copy)?;
        self.update_wire()?;
        self.reload_adjacent_edges(state)?;
        state.changed_edges = true;
        Ok(())
    }

    fn copy_adjacent_vertices(
        &self,
        state: &CrossingState,
        reshape: &ShapeBuildReShape,
    ) -> Result<(Vertex, Vertex, Vertex), Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let previous = analysis.first_vertex(&state.edges[0]);
        let shared = reshape.copy_vertex(&state.shared[0], -1.0)?;
        let last = analysis.last_vertex(&state.edges[1]);
        let (first_copy, last_copy) = if previous.is_same(&last) {
            let copy = reshape.copy_vertex(&previous, -1.0)?;
            (copy.clone(), copy)
        } else {
            (
                reshape.copy_vertex(&previous, -1.0)?,
                reshape.copy_vertex(&last, -1.0)?,
            )
        };
        Ok((first_copy, shared, last_copy))
    }

    fn replace_adjacent_edges(
        &self,
        state: &CrossingState,
        reshape: &ShapeBuildReShape,
        first_vertex: &Vertex,
        shared_vertex: &Vertex,
        last_vertex: &Vertex,
    ) -> Result<(), Failure> {
        let first =
            ShapeBuildEdge.copy_replace_vertices(&state.edges[0], first_vertex, shared_vertex)?;
        let second =
            ShapeBuildEdge.copy_replace_vertices(&state.edges[1], shared_vertex, last_vertex)?;
        replace_context_edge(reshape, &state.edges[0], &first)?;
        replace_context_edge(reshape, &state.edges[1], &second)
    }

    fn reload_adjacent_edges(&self, state: &mut CrossingState) -> Result<(), Failure> {
        let wire = self.wire_data()?;
        state.edges = [
            wire.edge(state.wire_indices[0])?,
            wire.edge(state.wire_indices[1])?,
        ];
        let analysis = ShapeAnalysisEdge::new();
        state.shared = [
            analysis.last_vertex(&state.edges[0]),
            analysis.first_vertex(&state.edges[1]),
        ];
        state.point = BRepTool::pnt(&state.shared[0])?;
        state.tolerance = BRepTool::vertex_tolerance(&state.shared[0])?;
        Ok(())
    }

    fn raise_edge_tolerance(&self, edge: &Edge, tolerance: f64) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let tolerance = 1.000001 * tolerance;
        builder.update_edge_tolerance(edge, tolerance)?;
        let analysis = ShapeAnalysisEdge::new();
        builder.update_vertex_tolerance(&analysis.first_vertex(edge), tolerance)?;
        builder.update_vertex_tolerance(&analysis.last_vertex(edge), tolerance)
    }

    fn apply_intersection_action(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
    ) -> Result<(), Failure> {
        if self.topology_mode() {
            if action.improved {
                return Ok(());
            }
            return self.cut_adjacent_edges(state, action);
        }
        if action.tolerance > self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(());
        }
        if state.tolerance < action.tolerance {
            self.or_last_fix_status(ShapeExtendStatus::Done1);
            state.tolerance = action.tolerance.max(1.001 * action.radius);
        }
        Ok(())
    }

    fn cut_adjacent_edges(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
    ) -> Result<(), Failure> {
        let face = self.analyzer()?.face();
        let (cuts, cut_line) = cut_adjacent_pair(state, &face, (action.param1, action.param2))?;
        self.record_cut_result(state, cuts, action, cut_line)
    }

    fn record_cut_result(
        &self,
        state: &mut CrossingState,
        cuts: [bool; 2],
        action: &CrossingAction,
        cut_line: bool,
    ) -> Result<(), Failure> {
        self.record_cut_flags(state, cuts);
        self.record_cut_tolerance(state, action, cut_line, cuts_allow_edit(state, cuts))
    }

    fn record_cut_flags(&self, state: &CrossingState, cuts: [bool; 2]) {
        if cuts[0] || cuts[1] {
            self.or_last_fix_status(ShapeExtendStatus::Done7);
        }
        if !cuts[0] && same_vertex(&state.shared[0], &state.edges[0], false) {
            self.or_last_fix_status(ShapeExtendStatus::Done3);
        }
        if !cuts[1] && same_vertex(&state.shared[1], &state.edges[1], true) {
            self.or_last_fix_status(ShapeExtendStatus::Done4);
        }
    }

    fn record_cut_tolerance(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
        cut_line: bool,
        may_edit: bool,
    ) -> Result<(), Failure> {
        let same_parameter = BRepTool::same_parameter(&state.edges[0])?
            && BRepTool::same_parameter(&state.edges[1])?;
        if cut_line || may_edit && same_parameter {
            return self.record_cut_done2(state, action);
        }
        self.record_cut_tolerance_fallback(state, action)
    }

    fn record_cut_done2(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
    ) -> Result<(), Failure> {
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        state.point = action.point;
        if state.tolerance <= action.radius {
            self.or_last_fix_status(ShapeExtendStatus::Done1);
            state.tolerance = 1.001 * action.radius;
        }
        Ok(())
    }

    fn record_cut_tolerance_fallback(
        &self,
        state: &mut CrossingState,
        action: &CrossingAction,
    ) -> Result<(), Failure> {
        if state.tolerance < action.tolerance && action.tolerance <= self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Done1);
            state.tolerance = action.tolerance;
        } else if action.tolerance > self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn store_adjacent_shared_point(&self, state: &CrossingState) -> Result<(), Failure> {
        if !self.last_fix_status(ShapeExtendStatus::Done) {
            return Ok(());
        }
        let context = self.context();
        if let Some(reshape) = context.get()
            && !state.changed_edges
        {
            for vertex in distinct_shared_vertices(&state.shared) {
                reshape.copy_vertex_at(vertex, state.point, state.tolerance)?;
            }
            return Ok(());
        }
        for vertex in distinct_shared_vertices(&state.shared) {
            BRepBuilder::default().update_vertex_point(vertex, state.point, state.tolerance)?;
        }
        Ok(())
    }

    fn transfer_intersection_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }
}

impl CrossingState {
    fn load(
        analyzer: &ShapeAnalysisWire,
        index: i32,
        context: &Handle<crate::shape_build::ShapeBuildReShape>,
    ) -> Result<Self, Failure> {
        let (first_index, second_index) = adjacent_edge_indices(analyzer.nb_edges(), index);
        let edges = [
            apply_context_edge(analyzer.wire_data().edge(first_index)?, context)?,
            apply_context_edge(analyzer.wire_data().edge(second_index)?, context)?,
        ];
        let (shared, ranges, forwards, point, tolerance) = measure_crossing(analyzer, &edges)?;
        Ok(Self {
            ranges,
            forwards,
            point,
            tolerance,
            edges,
            shared,
            previous_ranges: [f64::MAX; 2],
            wire_indices: [first_index, second_index],
            changed_edges: false,
        })
    }

    fn remaining_ranges(&self, first: f64, second: f64) -> [f64; 2] {
        let first_start = if self.forwards[0] {
            self.ranges[0].0
        } else {
            self.ranges[0].1
        };
        let second_end = if self.forwards[1] {
            self.ranges[1].1
        } else {
            self.ranges[1].0
        };
        [(first_start - first).abs(), (second_end - second).abs()]
    }

    pub(super) fn accept_intersection(&mut self, parameters: (f64, f64)) -> bool {
        let ranges = self.remaining_ranges(parameters.0, parameters.1);
        if ranges[0] > self.previous_ranges[0] && ranges[1] > self.previous_ranges[1] {
            return false;
        }
        self.previous_ranges = ranges;
        true
    }
}

fn replace_context_edge(
    context: &ShapeBuildReShape,
    original: &Edge,
    replacement: &Edge,
) -> Result<(), Failure> {
    ReShape::replace(
        context,
        &original.clone().into_shape(),
        &replacement.clone().into_shape(),
    )
}

fn apply_context_edge(
    edge: Edge,
    context: &Handle<crate::shape_build::ShapeBuildReShape>,
) -> Result<Edge, Failure> {
    let Some(reshape) = context.get() else {
        return Ok(edge);
    };
    let applied = reshape.apply(
        &edge.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Edge,
    )?;
    topods::edge(&applied)
}

fn distinct_shared_vertices(vertices: &[Vertex; 2]) -> Vec<&Vertex> {
    if vertices[0].is_same(&vertices[1]) {
        return vec![&vertices[0]];
    }
    vec![&vertices[0], &vertices[1]]
}

fn adjacent_intersections(
    analyzer: &ShapeAnalysisWire,
    index: i32,
) -> Result<AdjacentIntersectionRecords, Failure> {
    let (mut uv, mut xyz, mut errors) = (
        Sequence::default(),
        Sequence::default(),
        Sequence::default(),
    );
    analyzer.check_adjacent_intersections_into(index, &mut uv, &mut xyz, &mut errors)?;
    Ok((uv, xyz, errors))
}

type CrossingMeasurements = ([Vertex; 2], [(f64, f64); 2], [bool; 2], Point3, f64);

fn adjacent_edge_indices(edge_count: i32, index: i32) -> (i32, i32) {
    let second = if index > 0 { index } else { edge_count };
    let first = if second > 1 { second - 1 } else { edge_count };
    (first, second)
}

fn measure_crossing(
    analyzer: &ShapeAnalysisWire,
    edges: &[Edge; 2],
) -> Result<CrossingMeasurements, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let shared = [
        analysis.last_vertex(&edges[0]),
        analysis.first_vertex(&edges[1]),
    ];
    let ranges = [
        BRepTool::range_on_face(&edges[0], &analyzer.face())?,
        BRepTool::range_on_face(&edges[1], &analyzer.face())?,
    ];
    let forwards = [
        edges[0].orientation() == rustycad_tkg3d::top_abs::Orientation::Forward,
        edges[1].orientation() == rustycad_tkg3d::top_abs::Orientation::Forward,
    ];
    let point = BRepTool::pnt(&shared[0])?;
    let tolerance = BRepTool::vertex_tolerance(&shared[0])?;
    Ok((shared, ranges, forwards, point, tolerance))
}

fn edge_local_tolerance(
    state: &CrossingState,
    face: &Face,
    action: &CrossingAction,
    edge_index: usize,
    parameter: f64,
    endpoint: f64,
) -> Result<f64, Failure> {
    let deviation = local_deviation(
        &state.edges[edge_index],
        face,
        action.point,
        state.point,
        parameter,
        endpoint,
    )?;
    Ok(action.radius + deviation)
}

fn local_deviation_ends(state: &CrossingState) -> (f64, f64) {
    let first_end = if state.forwards[0] {
        state.ranges[0].1
    } else {
        state.ranges[0].0
    };
    let second_start = if state.forwards[1] {
        state.ranges[1].0
    } else {
        state.ranges[1].1
    };
    (first_end, second_start)
}

fn cuts_allow_edit(state: &CrossingState, cuts: [bool; 2]) -> bool {
    let first_allowed = cuts[0] || same_vertex(&state.shared[0], &state.edges[0], false);
    let second_allowed = cuts[1] || same_vertex(&state.shared[1], &state.edges[1], true);
    first_allowed && second_allowed
}

#[cfg(test)]
#[path = "wire_intersection_fix_tests.rs"]
mod tests;
