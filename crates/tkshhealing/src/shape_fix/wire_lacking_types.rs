//! Pair geometry shared by the `FixLacking` decision and mutation paths.
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3, Vector2};

pub(super) struct LackingPair {
    pub(super) wire: Handle<crate::shape_extend::ShapeExtendWireData>,
    pub(super) previous: Edge,
    pub(super) current: Edge,
    pub(super) first_vertex: Vertex,
    pub(super) last_vertex: Vertex,
    pub(super) face: rustycad_tkbrep::topods::Face,
    pub(super) first: Point2,
    pub(super) last: Point2,
    pub(super) previous_index: i32,
    pub(super) current_index: i32,
}

pub(super) struct LongGap {
    pub(super) start: Point3,
    pub(super) end: Point3,
    pub(super) start_tolerance: f64,
    pub(super) end_tolerance: f64,
}

impl LongGap {
    pub(super) fn measure(
        pair: &LackingPair,
        analyzer: &ShapeAnalysisWire,
        start: Point3,
        end: Point3,
    ) -> Result<Self, Failure> {
        let (start_gap, end_gap) = endpoint_surface_gaps(pair, analyzer, start, end)?;
        Ok(Self {
            start,
            end,
            start_tolerance: BRepTool::edge_tolerance(&pair.previous)?.max(start_gap),
            end_tolerance: BRepTool::edge_tolerance(&pair.current)?.max(end_gap),
        })
    }

    pub(super) fn is_candidate(
        &self,
        precision: f64,
        maximum: f64,
        analyzer: &ShapeAnalysisWire,
    ) -> bool {
        let distance = self.start.square_distance(&self.end);
        let allowed = self.start_tolerance + self.end_tolerance;
        !analyzer.last_check_status(ShapeExtendStatus::Done2)
            && distance > 1.25 * allowed * allowed
            && (distance > precision * precision || analyzer.max_distance_3d() > maximum)
    }
}

impl LackingPair {
    pub(super) fn load(
        analyzer: &ShapeAnalysisWire,
        index: i32,
        first: Point2,
        last: Point2,
    ) -> Result<Self, Failure> {
        let wire = analyzer.wire_data();
        let (previous_index, current_index, previous, current) = pair_edges(&wire, index)?;
        let (first_vertex, last_vertex) = pair_vertices(&previous, &current);
        Ok(Self {
            wire,
            previous,
            current,
            first_vertex,
            last_vertex,
            face: analyzer.face(),
            first,
            last,
            previous_index,
            current_index,
        })
    }

    pub(super) fn both_regular(&self) -> Result<bool, Failure> {
        Ok(!BRepTool::degenerated(&self.previous)? && !BRepTool::degenerated(&self.current)?)
    }

    pub(super) fn vertex_tolerance(&self) -> Result<f64, Failure> {
        let first = BRepTool::vertex_tolerance(&self.first_vertex)?;
        let last = BRepTool::vertex_tolerance(&self.last_vertex)?;
        Ok(first.max(last))
    }
}

pub(super) fn pair_edges(
    wire: &Handle<crate::shape_extend::ShapeExtendWireData>,
    index: i32,
) -> Result<(i32, i32, Edge, Edge), Failure> {
    let current = if index > 0 { index } else { wire.nb_edges() };
    let previous = if current > 1 {
        current - 1
    } else {
        wire.nb_edges()
    };
    Ok((previous, current, wire.edge(previous)?, wire.edge(current)?))
}

fn pair_vertices(previous: &Edge, current: &Edge) -> (Vertex, Vertex) {
    let analysis = ShapeAnalysisEdge::new();
    (
        analysis.last_vertex(previous),
        analysis.first_vertex(current),
    )
}

pub(super) fn average_vertex_point(first: &Vertex, last: &Vertex) -> Result<Point3, Failure> {
    let first = BRepTool::pnt(first)?;
    let last = BRepTool::pnt(last)?;
    Ok(Point3::new(
        (first.x() + last.x()) * 0.5,
        (first.y() + last.y()) * 0.5,
        (first.z() + last.z()) * 0.5,
    ))
}

pub(super) fn pair_surface_midpoint(pair: &LackingPair) -> Result<Point3, Failure> {
    let midpoint = midpoint_2d(pair.first, pair.last);
    let (surface, location) = BRepTool::surface_with_location(&pair.face)?;
    let surface = surface
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire face has no support surface"))?;
    Ok(surface
        .value(midpoint.x(), midpoint.y())?
        .transformed(location.transformation()))
}

pub(super) fn lacking_curve_endpoints(
    pair: &LackingPair,
) -> Result<Option<(Point3, Point3)>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (first, _, first_last) = analysis.curve_3d(&pair.previous, true)?;
    let (last, last_first, _) = analysis.curve_3d(&pair.current, true)?;
    let (Some(first), Some(last)) = (first.get(), last.get()) else {
        return Ok(None);
    };
    Ok(Some((first.value(first_last)?, last.value(last_first)?)))
}

pub(super) fn missing_line(
    first: Point2,
    last: Point2,
) -> Result<(Handle<dyn Geom2dCurve>, f64), Failure> {
    let vector = Vector2::new(last.x() - first.x(), last.y() - first.y());
    let direction = Direction2::new(vector.x(), vector.y())?;
    let curve = Handle::new(Geom2dLine::new(Axis2d::new(first, direction))).cast();
    Ok((curve, vector.magnitude()))
}

fn endpoint_surface_gaps(
    pair: &LackingPair,
    analyzer: &ShapeAnalysisWire,
    start: Point3,
    end: Point3,
) -> Result<(f64, f64), Failure> {
    let surface = analyzer.surface();
    let first = surface.value(pair.first.x(), pair.first.y())?;
    let last = surface.value(pair.last.x(), pair.last.y())?;
    Ok((start.distance(&first), end.distance(&last)))
}

fn midpoint_2d(first: Point2, last: Point2) -> Point2 {
    Point2::new((first.x() + last.x()) * 0.5, (first.y() + last.y()) * 0.5)
}
