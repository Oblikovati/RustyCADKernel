// Rust translation of ShapeFix/ShapeFix_Wire.cxx reorder and gap entry points at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Ordered-wire state, reorder application and the pinned periodic p-curve repair.
#[path = "wire_closed.rs"]
mod wire_closed;
#[path = "wire_connected.rs"]
mod wire_connected;
#[path = "wire_degenerated.rs"]
mod wire_degenerated;
#[path = "wire_edge_curves.rs"]
mod wire_edge_curves;
#[path = "wire_intersection_fix.rs"]
mod wire_intersection_fix;
#[path = "wire_intersection_geometry.rs"]
mod wire_intersection_geometry;
#[path = "wire_lacking.rs"]
mod wire_lacking;
#[path = "wire_nonadjacent_intersection_fix.rs"]
mod wire_nonadjacent_intersection_fix;
#[path = "wire_seam.rs"]
mod wire_seam;
#[path = "wire_self_intersection_fix.rs"]
mod wire_self_intersection_fix;
#[path = "wire_self_intersection_remove.rs"]
mod wire_self_intersection_remove;
#[path = "wire_self_intersection_split.rs"]
mod wire_self_intersection_split;
#[path = "wire_shifted.rs"]
mod wire_shifted;
#[path = "wire_small.rs"]
mod wire_small;
#[path = "wire_split_tool.rs"]
mod wire_split_tool;
#[path = "wire_state.rs"]
mod wire_state;

use super::{FixRoot, FixRootData, ShapeFixRoot};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire, ShapeAnalysisWireOrder};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::ShapeFixEdge;
use crate::shape_fix::ShapeFixShapeTolerance;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeWire;

#[derive(Default)]
struct WireFixState {
    analyzer: Handle<ShapeAnalysisWire>,
    reorder_status: i32,
    gaps_3d_status: i32,
    gaps_2d_status: i32,
    degenerated_status: i32,
    last_fix_status: i32,
    small_status: i32,
    connected_status: i32,
    closed_status: i32,
    intersections_status: i32,
    lacking_status: i32,
    closed_mode: bool,
    topology_mode: bool,
    geometry_mode: bool,
    remove_loop_mode: i32,
    edge_curves_mode: i32,
    reversed_2d_mode: i32,
    remove_pcurve_mode: i32,
    add_pcurve_mode: i32,
    remove_curve_3d_mode: i32,
    add_curve_3d_mode: i32,
    seam_mode: i32,
    shifted_mode: i32,
    same_parameter_mode: i32,
    vertex_tolerance_mode: i32,
    edge_curves_status: i32,
}

/// The implemented `ShapeFix_Wire` reorder, no-gap and periodic-cone p-curve paths.
pub struct ShapeFixWire {
    parent: ShapeFixRoot,
    state: SharedState<WireFixState>,
    edge_fixer: Handle<ShapeFixEdge>,
}

impl_transient!(ShapeFixWire, "ShapeFix_Wire", ShapeFixRoot, parent, interfaces: [dyn FixRoot]);

impl Default for ShapeFixWire {
    fn default() -> Self {
        Self {
            parent: ShapeFixRoot::new(),
            state: SharedState::new(WireFixState {
                analyzer: Handle::new(ShapeAnalysisWire::new()),
                reorder_status: 0,
                gaps_3d_status: 0,
                gaps_2d_status: 0,
                degenerated_status: 0,
                last_fix_status: 0,
                small_status: 0,
                connected_status: 0,
                closed_status: 0,
                intersections_status: 0,
                lacking_status: 0,
                closed_mode: true,
                topology_mode: false,
                geometry_mode: true,
                remove_loop_mode: -1,
                edge_curves_mode: -1,
                reversed_2d_mode: -1,
                remove_pcurve_mode: -1,
                add_pcurve_mode: -1,
                remove_curve_3d_mode: -1,
                add_curve_3d_mode: -1,
                seam_mode: -1,
                shifted_mode: -1,
                same_parameter_mode: -1,
                vertex_tolerance_mode: -1,
                edge_curves_status: 0,
            }),
            edge_fixer: Handle::new(ShapeFixEdge::new()),
        }
    }
}

impl ShapeFixWire {
    /// `ShapeFix_Wire()`: an unloaded fixer with closed-wire mode enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(analyzer)`: share an existing ShapeAnalysis_Wire instance.
    pub fn init(&self, analyzer: &Handle<ShapeAnalysisWire>) {
        self.clear_statuses();
        self.state.write().analyzer = analyzer.clone();
        self.fix_root_data()
            .with_mut(|fields| fields.shape = Shape::default());
    }

    /// `Load(wire)`: replace the analyzer edge list and clear prior statuses.
    pub fn load(&self, wire: &Wire) -> Result<(), Failure> {
        self.clear_statuses();
        let working_wire = self.context_wire(wire)?;
        let analyzer = self.ensure_analyzer();
        analyzer.load_wire(&working_wire)?;
        self.fix_root_data()
            .with_mut(|fields| fields.shape = wire.clone().into_shape());
        Ok(())
    }

    /// `Load(wireData)`: share data and apply context replacements in place;
    /// `fixer.load_wire_data(&data)?`.
    pub fn load_wire_data(&self, wire_data: &Handle<ShapeExtendWireData>) -> Result<(), Failure> {
        self.clear_statuses();
        self.ensure_analyzer().load_wire_data(wire_data);
        self.update_wire()?;
        self.fix_root_data()
            .with_mut(|fields| fields.shape = Shape::default());
        Ok(())
    }

    /// Applies the current reshape context to every working edge, splicing replacement edges.
    /// `fixer.update_wire()?`.
    pub fn update_wire(&self) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        let analyzer = self.ensure_analyzer();
        self.replace_context_edges(&analyzer.wire_data(), &context)
    }

    /// Current analysis data; `let data = fixer.wire_data()?`.
    pub fn wire_data(&self) -> Result<Handle<ShapeExtendWireData>, Failure> {
        Ok(self.analyzer()?.wire_data())
    }

    /// Whether the analyzer has at least one working edge; `fixer.is_loaded()`.
    pub fn is_loaded(&self) -> bool {
        self.state.read().analyzer.is_loaded()
    }

    /// Whether both working edges and a supporting face are available; `fixer.is_ready()`.
    pub fn is_ready(&self) -> bool {
        self.state.read().analyzer.is_ready()
    }

    /// Builds a wire from the current working edge sequence; `fixer.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        self.ensure_analyzer().wire_data().wire()
    }

    /// Builds a wire using the BRep API maker; `fixer.wire_api_make()?`.
    pub fn wire_api_make(&self) -> Result<Wire, Failure> {
        self.ensure_analyzer().wire_data().wire_api_make()
    }

    /// `SetFace(face)`: update the shared analyzer support face.
    pub fn set_face(&self, face: &Face) -> Result<(), Failure> {
        self.ensure_analyzer().set_face(face)
    }

    /// Set whether `FixLacking` may add edges; `fixer.set_topology_mode(true)`.
    pub fn set_topology_mode(&self, enabled: bool) {
        self.state.write().topology_mode = enabled;
    }

    /// Set whether `FixLacking` may bend existing p-curves; `fixer.set_geometry_mode(false)`.
    pub fn set_geometry_mode(&self, enabled: bool) {
        self.state.write().geometry_mode = enabled;
    }

    /// Select `ModifyRemoveLoopMode()` behavior; `fixer.set_remove_loop_mode(1)`.
    pub fn set_remove_loop_mode(&self, mode: i32) {
        self.state.write().remove_loop_mode = mode;
    }

    /// Query the aggregate flags from the last `FixLacking()` call.
    pub fn status_lacking(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().lacking_status, status)
    }

    /// Queries accumulated small-edge repair status; `fixer.status_small(ShapeExtendStatus::Done)`.
    pub fn status_small(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().small_status, status)
    }

    /// Query aggregate connection repair flags; `fixer.status_connected(ShapeExtendStatus::Done)`.
    pub fn status_connected(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().connected_status, status)
    }

    /// `FixReorder()`: analyze the loaded 3D endpoints, then apply the proposed order.
    pub fn fix_reorder(&self) -> Result<bool, Failure> {
        let mut order = ShapeAnalysisWireOrder::new();
        self.ensure_analyzer()
            .check_order_with(&mut order, self.closed_mode(), true, false)?;
        self.fix_reorder_with(&order)
    }

    /// `FixReorder(order)`: reorder existing wire data using its signed edge indices.
    pub fn fix_reorder_with(&self, order: &ShapeAnalysisWireOrder) -> Result<bool, Failure> {
        self.set_reorder_status(ShapeExtendStatus::Ok);
        let wire_data = self.wire_data()?;
        let count = wire_data.nb_edges();
        if count == 0 || order.status() == 0 {
            return Ok(false);
        }
        if order.status() <= -10 {
            self.set_reorder_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        if count != order.nb_edges() {
            self.set_reorder_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        self.apply_order(&wire_data, order, count)?;
        self.record_reorder_status(order.status());
        Ok(true)
    }

    /// `FixGaps3d()`: run the native adjacent-curve analysis when no gap needs repair.
    pub fn fix_gaps_3d(&self) -> Result<bool, Failure> {
        self.set_gap_status(true, ShapeExtendStatus::Ok);
        self.fix_gap_pass(true)
    }

    /// `FixGaps2d()`: analyze adjacent p-curves and repair the pinned closed periodic gap.
    pub fn fix_gaps_2d(&self) -> Result<bool, Failure> {
        self.set_gap_status(false, ShapeExtendStatus::Ok);
        self.fix_gap_pass(false)
    }

    /// Splits one strict interior crossing between nonadjacent line p-curves.
    pub fn fix_line_self_intersections(&self) -> Result<bool, Failure> {
        self.state.write().intersections_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        let face = analyzer.face();
        if face.is_null() {
            return Ok(false);
        }
        let Some((first_index, second_index, hit)) = self.first_line_crossing(&analyzer)? else {
            return Ok(false);
        };
        self.split_crossing_pair(&analyzer, &face, first_index, second_index, hit)?;
        self.state.write().intersections_status =
            ShapeExtend::encode_status(ShapeExtendStatus::Done1)
                | ShapeExtend::encode_status(ShapeExtendStatus::Done5);
        let _ = self.fix_reorder()?;
        Ok(true)
    }

    /// `StatusSelfIntersection(status)`: query the implemented wire-intersection flags.
    pub fn status_self_intersection(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().intersections_status, status)
    }

    /// `StatusReorder(status)`: query reorder flags.
    pub fn status_reorder(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().reorder_status, status)
    }

    /// `StatusGaps3d(status)`: query 3D gap-fixing flags.
    pub fn status_gaps_3d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().gaps_3d_status, status)
    }

    /// `StatusGaps2d(status)`: query 2D gap-fixing flags.
    pub fn status_gaps_2d(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().gaps_2d_status, status)
    }

    fn ensure_analyzer(&self) -> Handle<ShapeAnalysisWire> {
        let mut state = self.state.write();
        if state.analyzer.is_null() {
            state.analyzer = Handle::new(ShapeAnalysisWire::new());
        }
        state.analyzer.clone()
    }

    fn analyzer(&self) -> Result<Handle<ShapeAnalysisWire>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() {
            return Err(Failure::new(
                "ShapeFix_Wire requires Init(analyzer) or Load(wire)",
            ));
        }
        Ok(analyzer)
    }

    fn context_wire(&self, wire: &Wire) -> Result<Wire, Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(wire.clone());
        }
        let context = context
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire context is null"))?;
        let replacement = ReShape::apply(context, &wire.clone().into_shape(), ShapeEnum::Shape)?;
        topods::wire(&replacement)
    }

    fn replace_context_edges(
        &self,
        wire_data: &ShapeExtendWireData,
        context: &Handle<ShapeBuildReShape>,
    ) -> Result<(), Failure> {
        let context = context
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire context is null"))?;
        let mut index = 1;
        while index <= wire_data.nb_edges() {
            let edge = wire_data.edge(index)?;
            let replacement =
                ReShape::apply(context, &edge.clone().into_shape(), ShapeEnum::Shape)?;
            if replacement == edge.clone().into_shape() {
                index += 1;
                continue;
            }
            let replacements = replacement_edges(&replacement)?;
            self.splice_context_edges(wire_data, index, &replacements)?;
            index += replacements.len() as i32;
        }
        Ok(())
    }

    fn splice_context_edges(
        &self,
        wire_data: &ShapeExtendWireData,
        index: i32,
        replacements: &[Edge],
    ) -> Result<(), Failure> {
        for (offset, edge) in replacements.iter().enumerate() {
            wire_data.add_edge(edge, index + offset as i32)?;
        }
        wire_data.remove(index + replacements.len() as i32)
    }

    /// Reads native `ClosedMode`; `assert!(fixer.closed_mode())`.
    pub fn closed_mode(&self) -> bool {
        self.state.read().closed_mode
    }

    /// Reads native `TopoMode`; `assert!(!fixer.topology_mode())`.
    pub fn topology_mode(&self) -> bool {
        self.state.read().topology_mode
    }

    fn geometry_mode(&self) -> bool {
        self.state.read().geometry_mode
    }

    fn remove_loop_mode(&self) -> i32 {
        self.state.read().remove_loop_mode
    }

    fn apply_order(
        &self,
        wire_data: &Handle<ShapeExtendWireData>,
        order: &ShapeAnalysisWireOrder,
        count: i32,
    ) -> Result<(), Failure> {
        let old_edges = (1..=count)
            .map(|index| wire_data.edge(index))
            .collect::<Result<Vec<Edge>, _>>()?;
        for rank in 1..=count {
            let Some(edge) = self.reordered_edge(&old_edges, order, rank, count)? else {
                return Ok(());
            };
            wire_data.set(&edge, rank)?;
        }
        Ok(())
    }

    fn reordered_edge(
        &self,
        edges: &[Edge],
        order: &ShapeAnalysisWireOrder,
        rank: i32,
        count: i32,
    ) -> Result<Option<Edge>, Failure> {
        let signed_index = order.ordered(rank)?;
        if signed_index == 0 {
            self.set_reorder_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        let old_index = signed_index.abs();
        if old_index > count {
            return Err(invalid_order_index(signed_index, count));
        }
        let mut edge = edges[(old_index - 1) as usize].clone();
        if signed_index < 0 {
            edge.reverse();
        }
        Ok(Some(edge))
    }

    fn record_reorder_status(&self, order_status: i32) {
        self.set_reorder_status(ShapeExtendStatus::Done1);
        if matches!(order_status, 2 | -2) {
            self.or_reorder_status(ShapeExtendStatus::Done2);
        }
        if order_status < 0 {
            self.or_reorder_status(ShapeExtendStatus::Done3);
        }
        if order_status == 3 {
            self.or_reorder_status(ShapeExtendStatus::Done5);
        }
    }

    fn fix_gap_pass(&self, is_3d: bool) -> Result<bool, Failure> {
        let analyzer = self.analyzer()?;
        if !analyzer.is_loaded() {
            return Ok(false);
        }
        if !is_3d && !analyzer.is_ready() {
            self.set_gap_status(false, ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        let start = if self.closed_mode() { 1 } else { 2 };
        for index in start..=analyzer.nb_edges() {
            self.fix_gap_pair(&analyzer, is_3d, index)?;
        }
        Ok(self.gap_status(is_3d, ShapeExtendStatus::Done))
    }

    fn first_line_crossing(
        &self,
        analyzer: &ShapeAnalysisWire,
    ) -> Result<Option<(i32, i32, super::PcurveCrossing)>, Failure> {
        let data = analyzer.wire_data();
        for first_index in 1..=(data.nb_edges() - 2).max(0) {
            for second_index in (first_index + 2)..=data.nb_edges() {
                if first_index == 1 && second_index == data.nb_edges() {
                    continue;
                }
                let first_edge = data.edge(first_index)?;
                let second_edge = data.edge(second_index)?;
                let first_pc = ShapeAnalysisEdge::new().pcurve_on_face(
                    &first_edge,
                    &analyzer.face(),
                    false,
                )?;
                let second_pc = ShapeAnalysisEdge::new().pcurve_on_face(
                    &second_edge,
                    &analyzer.face(),
                    false,
                )?;
                let Some(hit) = super::line_pcurve_crossing(
                    &first_pc.0,
                    (first_pc.1, first_pc.2),
                    &second_pc.0,
                    (second_pc.1, second_pc.2),
                    rustycad_tkernel::precision::p_confusion(),
                ) else {
                    continue;
                };
                return Ok(Some((first_index, second_index, hit)));
            }
        }
        Ok(None)
    }

    fn split_crossing_pair(
        &self,
        analyzer: &ShapeAnalysisWire,
        face: &Face,
        first_index: i32,
        second_index: i32,
        hit: super::PcurveCrossing,
    ) -> Result<(), Failure> {
        let data = analyzer.wire_data();
        let vertex = crossing_vertex(face, hit.point)?;
        self.split_wire_edge(&data, face, second_index, hit.second_parameter, &vertex)?;
        self.split_wire_edge(&data, face, first_index, hit.first_parameter, &vertex)
    }

    fn split_wire_edge(
        &self,
        data: &ShapeExtendWireData,
        face: &Face,
        index: i32,
        parameter: f64,
        vertex: &Vertex,
    ) -> Result<(), Failure> {
        let edge = data.edge(index)?;
        let (first, second) = ShapeBuildEdge.split_at_parameter(&edge, face, parameter, vertex)?;
        let replacement = BRepBuilderAPIMakeWire::from_two_edges(&first, &second)?.wire()?;
        let mut context = self.context();
        if context.is_null() {
            context = Handle::new(ShapeBuildReShape::new());
            self.set_context(&context);
        }
        let context = context
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire context is null"))?;
        ReShape::replace(
            context,
            &edge.clone().into_shape(),
            &replacement.into_shape(),
        )?;
        data.set(&first, index)?;
        data.add_edge(&second, index + 1)
    }

    fn fix_gap_pair(
        &self,
        analyzer: &ShapeAnalysisWire,
        is_3d: bool,
        index: i32,
    ) -> Result<(), Failure> {
        let changed = if is_3d {
            analyzer.check_gap_3d_at(index)?
        } else {
            analyzer.check_gap_2d_at(index)?
        };
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_gap_status(is_3d, ShapeExtendStatus::Fail1);
            return Ok(());
        }
        if changed && !is_3d && self.fix_closed_periodic_pcurve_gap(analyzer, index)? {
            return Ok(());
        }
        if changed {
            return Err(unsupported_gap(index, is_3d, analyzer));
        }
        Ok(())
    }

    /// Repairs the same-edge periodic p-curve gap used by the pinned conical loop.
    fn fix_closed_periodic_pcurve_gap(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
    ) -> Result<bool, Failure> {
        if analyzer.nb_edges() != 1 || index != 1 {
            return Ok(false);
        }
        let edge = analyzer.wire_data().edge(1)?;
        let face = analyzer.face();
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(&edge, &face, false)?;
        let Some(midpoint) = periodic_gap_midpoint(&curve, first, last, &face)? else {
            return Ok(false);
        };
        let replacement =
            ShapeBuildEdge.copy_bspline_pcurve_with_endpoints(&curve, midpoint, midpoint)?;
        if !self.replace_closed_edge(&edge, &face, &replacement, (first, last), index)? {
            return Ok(false);
        }
        self.set_gap_status(false, ShapeExtendStatus::Done1);
        Ok(true)
    }

    /// Records the copied closed edge and vertex in the shared reshape context.
    fn replace_closed_edge(
        &self,
        edge: &Edge,
        face: &Face,
        pcurve: &Handle<dyn Geom2dCurve>,
        range: (f64, f64),
        index: i32,
    ) -> Result<bool, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first = analysis.first_vertex(edge);
        let last = analysis.last_vertex(edge);
        if !first.is_same(&last) {
            return Ok(false);
        }
        let (new_first, new_last) = replacement_vertices(&last)?;
        let fixed = ShapeBuildEdge.copy_replace_vertices_with_pcurve_range(
            edge, &new_first, &new_last, pcurve, face, range,
        )?;
        reset_replacement_tolerances(&new_last, &fixed)?;
        self.record_replacement(edge, &last, &new_last, &fixed)?;
        self.wire_data()?.set(&fixed, index)?;
        Ok(true)
    }

    /// Makes a context when needed and records the native vertex and edge replacements.
    fn record_replacement(
        &self,
        edge: &Edge,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
        replacement: &Edge,
    ) -> Result<(), Failure> {
        let mut context = self.context();
        if context.is_null() {
            context = Handle::new(ShapeBuildReShape::new());
            self.set_context(&context);
        }
        let context = context.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeFix_Wire context became null; expected an initialized reshape context",
            )
        })?;
        ReShape::replace(
            context,
            &old_vertex.clone().into_shape(),
            &new_vertex.clone().into_shape(),
        )?;
        ReShape::replace(
            context,
            &edge.clone().into_shape(),
            &replacement.clone().into_shape(),
        )
    }

    fn clear_statuses(&self) {
        let mut state = self.state.write();
        state.reorder_status = 0;
        state.gaps_3d_status = 0;
        state.gaps_2d_status = 0;
        state.degenerated_status = 0;
        state.intersections_status = 0;
        state.lacking_status = 0;
        state.edge_curves_status = 0;
        state.last_fix_status = 0;
        state.small_status = 0;
        state.connected_status = 0;
        state.closed_status = 0;
    }

    fn set_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status = ShapeExtend::encode_status(status);
    }

    fn or_reorder_status(&self, status: ShapeExtendStatus) {
        self.state.write().reorder_status |= ShapeExtend::encode_status(status);
    }

    fn set_gap_status(&self, is_3d: bool, status: ShapeExtendStatus) {
        let mut state = self.state.write();
        let flags = if is_3d {
            &mut state.gaps_3d_status
        } else {
            &mut state.gaps_2d_status
        };
        *flags |= ShapeExtend::encode_status(status);
    }

    fn gap_status(&self, is_3d: bool, status: ShapeExtendStatus) -> bool {
        let state = self.state.read();
        let flags = if is_3d {
            state.gaps_3d_status
        } else {
            state.gaps_2d_status
        };
        ShapeExtend::decode_status(flags, status)
    }
}

/// Finds the midpoint after moving periodic U values to their nearest representation.
fn periodic_gap_midpoint(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    face: &Face,
) -> Result<Option<Point2>, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let Some(surface) = surface.get() else {
        return Ok(None);
    };
    if !surface.is_u_periodic() {
        return Ok(None);
    }
    let start = curve.value(first)?;
    let end = curve.value(last)?;
    let end_u = nearest_periodic_value(end.x(), start.x(), surface.u_period()?);
    Ok(Some(Point2::new(
        (start.x() + end_u) * 0.5,
        (start.y() + end.y()) * 0.5,
    )))
}

/// Shifts a periodic value by the nearest integer period to its target.
fn nearest_periodic_value(value: f64, target: f64, period: f64) -> f64 {
    value + ((target - value) / period).round() * period
}

fn crossing_vertex(face: &Face, point: Point2) -> Result<Vertex, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let surface = surface
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire face has no surface"))?;
    let position = surface
        .value(point.x(), point.y())?
        .transformed(location.transformation());
    let mut vertex = Vertex::default();
    rustycad_tkbrep::brep::BRepBuilder::default().make_vertex_at(
        &mut vertex,
        position,
        rustycad_tkernel::precision::confusion(),
    )?;
    Ok(vertex)
}

/// Copies the closed endpoint vertex and gives its two edge occurrences opposite senses.
fn replacement_vertices(vertex: &Vertex) -> Result<(Vertex, Vertex), Failure> {
    let copy = topods::vertex(&vertex.empty_copied()?)?;
    let reversed = topods::vertex(&copy.clone().into_shape().oriented(Orientation::Reversed))?;
    Ok((reversed, copy))
}

/// Sets the replacement vertex and edge tolerances to OCCT's confusion value.
fn reset_replacement_tolerances(vertex: &Vertex, edge: &Edge) -> Result<(), Failure> {
    let tolerance = ShapeFixShapeTolerance::new();
    tolerance.set_tolerance(
        &vertex.clone().into_shape(),
        rustycad_tkernel::precision::confusion(),
        ShapeEnum::Vertex,
    )?;
    tolerance.set_tolerance(
        &edge.clone().into_shape(),
        rustycad_tkernel::precision::confusion(),
        ShapeEnum::Edge,
    )
}

impl FixRoot for ShapeFixWire {
    fn set_precision(&self, precision: f64) {
        self.parent.set_precision(precision);
        self.state.read().analyzer.set_precision(precision);
    }

    fn fix_root_data(&self) -> &FixRootData {
        self.parent.fix_root_data()
    }
}

fn invalid_order_index(index: i32, count: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "ShapeFix_Wire::FixReorder: ordered index {index}; expected an index within 1..={count}"
        ),
    )
}

fn unsupported_gap(index: i32, is_3d: bool, analyzer: &ShapeAnalysisWire) -> Failure {
    let (space, distance) = if is_3d {
        ("3D", analyzer.max_distance_3d())
    } else {
        ("2D", analyzer.max_distance_2d())
    };
    Failure::with_message(
        "Standard_NotImplemented",
        &format!(
            "ShapeFix_Wire gap correction is not ported for {space} pair {index}, gap {distance}"
        ),
    )
}

fn replacement_edges(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    if shape.is_null() {
        return Ok(Vec::new());
    }
    if shape.shape_type()? == ShapeEnum::Edge {
        return Ok(vec![topods::edge(shape)?]);
    }
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut edges = Vec::new();
    while explorer.more() {
        edges.push(topods::edge(explorer.current())?);
        explorer.next()?;
    }
    Ok(edges)
}
