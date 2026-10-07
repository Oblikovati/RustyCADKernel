//! `Path` (BOPAlgo_WireSplitter_1.cxx): the walk from one leaving edge along the smallest
//! clockwise turns, closing a loop each time it comes back to a visited vertex.
use super::WireSplitter;
use super::angles::{clockwise_angle, coord_2d, coord_2d_forward, tolerance_2d, uv_tolerance_2d};
use super::split_block::{ClosedVertices, SmartMap};
use crate::bop_tools::ConnexityBlock;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, real::epsilon};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// An edge info in the smart map: the vertex index and the position in its list.
pub(super) type InfoRef = (usize, usize);

/// The fixed inputs of the walk.
pub(super) struct PathWalk<'a> {
    pub(super) surface: &'a BRepAdaptorSurface,
    pub(super) face: &'a Face,
    pub(super) closed: &'a ClosedVertices,
}

/// The edges walked so far with their start vertices, 2D start points and infos
/// (`aLS`, `aVertVa`, `aCoordVa`, `anInfoSeq`).
#[derive(Default)]
struct Trail {
    edges: Vec<Shape>,
    vertices: Vec<Shape>,
    points: Vec<Point2>,
    infos: Vec<InfoRef>,
}

impl Trail {
    fn truncate(&mut self, length: usize) {
        self.edges.truncate(length);
        self.vertices.truncate(length);
        self.points.truncate(length);
        self.infos.truncate(length);
    }
}

/// The end of the current edge: the next vertex, its 2D point, its edge infos and its 2D
/// tolerance (squared).
struct Arrival {
    vertex: Vertex,
    point: Point2,
    index: usize,
    tolerance_sq: f64,
    closed: bool,
}

/// Walks from `start_edge` leaving `start_vertex`, appending the loops it closes to `block`.
pub(super) fn path(
    walk: &PathWalk<'_>,
    start_vertex: &Vertex,
    start_edge: &Edge,
    start_info: InfoRef,
    block: &mut ConnexityBlock,
    smart: &mut SmartMap,
) -> Result<(), Failure> {
    let mut trail = Trail::default();
    let (mut vertex, mut edge, mut info) = (start_vertex.clone(), start_edge.clone(), start_info);
    loop {
        // Do not escape through the edge from which you enter.
        if trail.edges.len() == 1 && trail.edges[0].is_same(&edge) {
            return Ok(());
        }
        let arrival = advance(walk, &mut trail, &vertex, &edge, info, smart)?;
        match close_loops(walk, &mut trail, &arrival, block)? {
            LoopClosure::Finished => return Ok(()),
            LoopClosure::Rewound => {
                edge = Edge::unchecked(trail.edges.last().cloned().unwrap_or_default());
                info = *trail.infos.last().unwrap_or(&info);
            }
            LoopClosure::Open => {}
        }
        let Some(next) = next_edge(walk, &edge, info, &arrival, smart)? else {
            // No way to go (error).
            return Ok(());
        };
        vertex = arrival.vertex;
        edge = smart.find_from_index(next.0)?[next.1].edge().clone();
        info = next;
    }
}

/// Appends the current edge to the trail and finds the vertex it arrives at.
fn advance(
    walk: &PathWalk<'_>,
    trail: &mut Trail,
    vertex: &Vertex,
    edge: &Edge,
    info: InfoRef,
    smart: &mut SmartMap,
) -> Result<Arrival, Failure> {
    smart.change_from_index(info.0)?[info.1].set_passed(true);
    trail.edges.push(edge.clone().into_shape());
    trail.vertices.push(vertex.clone().into_shape());
    trail.infos.push(info);
    let forward = Vertex::unchecked(vertex.oriented(Orientation::Forward));
    trail.points.push(coord_2d(&forward, edge, walk.face)?);
    let next = next_vertex(&forward, edge);
    let point = coord_2d(&next, edge, walk.face)?;
    let index = smart.find_index(&next);
    if index == 0 {
        return Err(missing_vertex(&next));
    }
    let tolerance = 2. * tolerance_2d(&next, walk.surface)?;
    let closed = *walk.closed.find(&next)?;
    Ok(Arrival {
        vertex: next,
        point,
        index,
        tolerance_sq: tolerance * tolerance,
        closed,
    })
}

/// The outcome of looking for a loop closed by the arrival vertex.
enum LoopClosure {
    /// The arrival vertex was not visited.
    Open,
    /// A loop was closed and the whole trail consumed.
    Finished,
    /// A loop was closed; the trail was cut back to the edges before it.
    Rewound,
}

/// Closes the loop ending at the arrival vertex when it was already visited (at the same
/// 2D place on closed vertices), cutting the trail back.
fn close_loops(
    walk: &PathWalk<'_>,
    trail: &mut Trail,
    arrival: &Arrival,
    block: &mut ConnexityBlock,
) -> Result<LoopClosure, Failure> {
    let mut buffer: List<Shape> = List::default();
    let mut has_edge = false;
    for position in (0..trail.edges.len()).rev() {
        let edge = trail.edges[position].clone();
        buffer.append(edge.clone());
        if !has_edge {
            has_edge = !BRepTool::degenerated(&Edge::unchecked(edge))?;
            // Do not create a wire from degenerated edges only.
            if !has_edge {
                continue;
            }
        }
        if !same_place(
            walk,
            &trail.vertices[position],
            trail.points[position],
            arrival,
        )? {
            continue;
        }
        if buffer.len() != 2 || !first_last(&buffer).0.is_same(first_last(&buffer).1) {
            let wire = WireSplitter::make_wire(&buffer)?;
            block.change_loops().append(wire.into_shape());
        }
        if position == 0 {
            trail.truncate(0);
            return Ok(LoopClosure::Finished);
        }
        trail.truncate(position);
        return Ok(LoopClosure::Rewound);
    }
    Ok(LoopClosure::Open)
}

fn first_last(list: &List<Shape>) -> (&Shape, &Shape) {
    let mut items = list.iter();
    let first = items.next().expect("non-empty buffer");
    (first, items.last().unwrap_or(first))
}

/// Whether a visited vertex is the arrival vertex, at the same 2D place when the arrival
/// vertex closes an edge.
fn same_place(
    walk: &PathWalk<'_>,
    visited: &Shape,
    visited_point: Point2,
    arrival: &Arrival,
) -> Result<bool, Failure> {
    if !visited.is_same(&arrival.vertex) {
        return Ok(false);
    }
    if !arrival.closed {
        return Ok(true);
    }
    if visited_point.square_distance(arrival.point) >= arrival.tolerance_sq {
        return Ok(false);
    }
    let (u_tolerance, v_tolerance) = uv_tolerance_2d(&arrival.vertex, walk.surface)?;
    let u_distance = (visited_point.x() - arrival.point.x()).abs();
    let v_distance = (visited_point.y() - arrival.point.y()).abs();
    Ok(u_distance <= 2. * u_tolerance && v_distance <= 2. * v_tolerance)
}

/// Chooses the edge leaving the arrival vertex: the only way out, else the smallest
/// clockwise turn, preferring the single inner way when walking a boundary edge.
fn next_edge(
    walk: &PathWalk<'_>,
    edge: &Edge,
    info: InfoRef,
    arrival: &Arrival,
    smart: &SmartMap,
) -> Result<Option<InfoRef>, Failure> {
    let infos = smart.find_from_index(arrival.index)?;
    let angle_in = angle_in(edge, infos);
    let ways_out = infos
        .iter()
        .filter(|candidate| !candidate.is_in() && !candidate.passed())
        .count();
    let is_boundary = !smart.find_from_index(info.0)?[info.1].is_inside();
    let (mut best, mut min_angle) = (None, 100.);
    let (mut inner_ways, mut only_inner) = (0, None);
    for (position, candidate) in infos.iter().enumerate() {
        if candidate.is_in() || candidate.passed() {
            continue;
        }
        match ways_out {
            0 => return Ok(None),
            1 => return Ok(Some((arrival.index, position))),
            _ => {}
        }
        let angle = if candidate.edge().is_same(edge) {
            PI + PI
        } else {
            if arrival.closed {
                let point = coord_2d_forward(candidate.edge(), walk.face)?;
                if point.square_distance(arrival.point) > arrival.tolerance_sq {
                    continue;
                }
            }
            clockwise_angle(angle_in, candidate.angle())
        };
        if is_boundary && candidate.is_inside() {
            inner_ways += 1;
            only_inner = Some((arrival.index, position));
        }
        if angle < min_angle - epsilon(1.) {
            min_angle = angle;
            best = Some((arrival.index, position));
        }
    }
    Ok(if inner_ways == 1 { only_inner } else { best })
}

/// The angle of the edge entering the vertex (`AngleIn`), 0 when not found.
fn angle_in(edge: &Edge, infos: &[super::EdgeInfo]) -> f64 {
    infos
        .iter()
        .find(|info| info.is_in() && info.edge().is_equal(edge))
        .map_or(0., super::EdgeInfo::angle)
}

/// The other vertex of `edge` than `vertex` (`GetNextVertex`); `vertex` itself when none.
fn next_vertex(vertex: &Vertex, edge: &Edge) -> Vertex {
    ShapeIterator::new(edge, true, true)
        .find(|other| !other.is_equal(vertex))
        .map_or_else(|| vertex.clone(), Vertex::unchecked)
}

fn missing_vertex(vertex: &Vertex) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "BOPAlgo_WireSplitter Path: vertex {vertex:?} is not in the vertex/edge map; \
             expected a vertex of the block"
        ),
        "",
    )
}
