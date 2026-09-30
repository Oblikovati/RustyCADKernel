// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_EdgeConnect.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_EdgeConnect`: shares the coincident end vertices of connected edges.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, FlatDataMap};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Xyz};

/// `ShapeFix_EdgeConnect`: records connections between the last vertex of an edge and
/// the first vertex of the next one, then replaces each group of connected vertices by
/// one shared vertex at the centre of the bounding box of the curve ends, with a
/// tolerance that holds them all.
///
/// ```
/// use rustycad_tkshhealing::shape_fix::ShapeFixEdgeConnect;
/// let mut tool = ShapeFixEdgeConnect::new();
/// tool.build().unwrap();
/// tool.clear();
/// ```
#[derive(Default)]
pub struct ShapeFixEdgeConnect {
    /// `myVertices`: each registered vertex and the shared vertex of its group.
    vertices: FlatDataMap<Shape, Shape, ShapeMapHasher>,
    /// `myLists`: for each shared vertex, the (vertex, edge) pairs of its group.
    lists: DataMap<Shape, Vec<Shape>, ShapeMapHasher>,
}

/// The static `EdgeEndUsage`: the ends of the FORWARD edge and which of them is the
/// vertex.
struct EdgeEndUsage {
    first: Vertex,
    last: Vertex,
    uses_first: bool,
    uses_last: bool,
}

/// The static `edgeEndUsage`.
fn edge_end_usage(edge: &Edge, vertex: &Vertex) -> EdgeEndUsage {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let (first, last) = topexp::vertices(&forward, false);
    let uses_first = vertex.is_same(&first);
    let uses_last = vertex.is_same(&last);
    EdgeEndUsage {
        first,
        last,
        uses_first,
        uses_last,
    }
}

/// The static `appendCurveEndPositions`: the used ends of every curve representation
/// (3D curves and curves on surfaces) of the edge.
fn append_curve_end_positions(
    edge: &Edge,
    usage: &EdgeEndUsage,
    positions: &mut Vec<Xyz>,
) -> Result<(), Failure> {
    let Ok(tedge) = edge_tshape(edge, "ShapeFix_EdgeConnect") else {
        return Ok(());
    };
    for representation in tedge.curves() {
        let handle = as_gcurve(&representation);
        let Some(curve) = handle.get() else { continue };
        let (first, last) = curve.range();
        if usage.uses_first {
            positions.push(curve.d0(first)?.xyz());
        }
        if usage.uses_last {
            positions.push(curve.d0(last)?.xyz());
        }
    }
    Ok(())
}

/// The static `positionAndTolerance`: the centre of the bounding box of the positions
/// (the only position when there is one, the origin when there is none), and the
/// largest distance to them times 1.0001, at least `Precision::Confusion()`.
fn position_and_tolerance(positions: &[Xyz]) -> (Xyz, f64) {
    let mut position = Xyz::new(0.0, 0.0, 0.0);
    let (mut lower, mut upper) = (position, position);
    for (index, current) in positions.iter().enumerate() {
        if index == 0 {
            (lower, upper) = (*current, *current);
        } else {
            lower = Xyz::new(
                lower.x().min(current.x()),
                lower.y().min(current.y()),
                lower.z().min(current.z()),
            );
            upper = Xyz::new(
                upper.x().max(current.x()),
                upper.y().max(current.y()),
                upper.z().max(current.z()),
            );
        }
        position = *current;
    }
    if positions.len() > 1 {
        position = (lower + upper) * 0.5;
    }
    let deviation = positions
        .iter()
        .fold(0.0_f64, |max, p| max.max((position - *p).modulus()));
    (position, (deviation * 1.0001).max(precision::confusion()))
}

/// The static `replaceVertex`: puts the shared vertex at the end of the (FORWARD) edge
/// that `vertex` bounds, at both ends of a closed edge. The edge is made free for the
/// builder and gets its free flag back.
fn replace_vertex(
    builder: &BRepBuilder,
    edge: &mut Edge,
    vertex: &Vertex,
    shared: &Vertex,
) -> Result<(), Failure> {
    // BRep_Builder modifies the oriented vertex list of this stored edge copy; keep the
    // legacy forward normalization before removing or adding vertices.
    edge.set_orientation(Orientation::Forward);
    let usage = edge_end_usage(edge, vertex);
    let old = if usage.uses_first {
        usage.first.clone()
    } else {
        usage.last.clone()
    };
    let orientation = if usage.uses_first {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    let new = topods::vertex(&shared.oriented(orientation))?;
    if old.is_same(&new) {
        return Ok(());
    }
    let was_free = edge.free()?;
    edge.set_free(true)?;
    builder.remove(edge, &old)?;
    builder.add(edge, &new)?;
    if usage.uses_first && usage.uses_last {
        builder.remove(edge, &old.oriented(Orientation::Reversed))?;
        builder.add(edge, &new.oriented(Orientation::Reversed))?;
    }
    edge.set_free(was_free)
}

impl ShapeFixEdgeConnect {
    /// `ShapeFix_EdgeConnect()`; `ShapeFixEdgeConnect::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Add(theFirstEdge, theSecondEdge)`: connects the oriented last vertex of the first
    /// edge with the oriented first vertex of the second one, merging their groups;
    /// `tool.add_edges(&e1, &e2)`.
    pub fn add_edges(&mut self, first_edge: &Edge, second_edge: &Edge) {
        let first_vertex: Shape = topexp::last_vertex(first_edge, true).into();
        let second_vertex: Shape = topexp::first_vertex(second_edge, true).into();
        let first_shared = self.vertices.seek(&first_vertex).cloned();
        let second_shared = self.vertices.seek(&second_vertex).cloned();
        match (first_shared, second_shared) {
            (None, None) => {
                self.vertices
                    .try_bind(first_vertex.clone(), first_vertex.clone());
                self.vertices
                    .try_bind(second_vertex.clone(), first_vertex.clone());
                let list = vec![
                    first_vertex.clone(),
                    first_edge.clone().into(),
                    second_vertex,
                    second_edge.clone().into(),
                ];
                if !self.lists.is_bound(&first_vertex) {
                    self.lists.bind(first_vertex, list);
                }
            }
            (Some(shared), None) => self.join(&shared, second_vertex, second_edge),
            (None, Some(shared)) => self.join(&shared, first_vertex, first_edge),
            (Some(first), Some(second)) => self.merge(&first, &second),
        }
    }

    /// Adds a (vertex, edge) pair to the group of `shared`.
    fn join(&mut self, shared: &Shape, vertex: Shape, edge: &Edge) {
        self.vertices.try_bind(vertex.clone(), shared.clone());
        if let Ok(list) = self.lists.change_find(shared) {
            list.push(vertex);
            list.push(edge.clone().into());
        }
    }

    /// Moves the group of `second` into the group of `first`.
    fn merge(&mut self, first: &Shape, second: &Shape) {
        if first.is_same(second) {
            return;
        }
        let moved = self.lists.find(second).cloned().unwrap_or_default();
        for vertex in moved.iter().step_by(2) {
            if let Some(slot) = self.vertices.change_seek(vertex) {
                *slot = first.clone();
            }
        }
        if let Ok(list) = self.lists.change_find(first) {
            list.extend(moved);
        }
        self.lists.unbind(second);
    }

    /// `Add(theShape)`: connects the consecutive edges of every wire of the shape (and the
    /// last edge with the first one when the wire has its Closed flag);
    /// `tool.add_shape(&shape)?`.
    pub fn add_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        for wire in Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let mut edges = Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)?;
            let Some(first) = Iterator::next(&mut edges) else {
                continue;
            };
            let first_edge = topods::edge(&first)?;
            let mut previous = first_edge.clone();
            for current in edges {
                let current = topods::edge(&current)?;
                self.add_edges(&previous, &current);
                previous = current;
            }
            if wire.closed()? {
                self.add_edges(&previous, &first_edge);
            }
        }
        Ok(())
    }

    /// `Build()`: for each group, moves the shared vertex to the centre of the connected
    /// curve ends with a tolerance that holds them, puts it on every edge of the group,
    /// then clears the registration; `tool.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let groups: Vec<(Shape, Vec<Shape>)> = self
            .lists
            .items()
            .map(|(shared, list)| (shared.clone(), list.clone()))
            .collect();
        for (shared_shape, list) in groups {
            let shared = topods::vertex(&shared_shape)?;
            let mut positions = Vec::new();
            for pair in list.chunks(2) {
                let (vertex, edge) = (topods::vertex(&pair[0])?, topods::edge(&pair[1])?);
                append_curve_end_positions(&edge, &edge_end_usage(&edge, &vertex), &mut positions)?;
            }
            let (position, tolerance) = position_and_tolerance(&positions);
            builder.update_vertex_point(&shared, Point3::from(position), tolerance)?;
            for pair in list.chunks(2) {
                let vertex = topods::vertex(&pair[0])?;
                let mut edge = topods::edge(&pair[1])?;
                replace_vertex(&builder, &mut edge, &vertex, &shared)?;
            }
        }
        self.clear();
        Ok(())
    }

    /// `Clear()`: forgets the connections without changing the edges; `tool.clear()`.
    pub fn clear(&mut self) {
        self.vertices.clear(true);
        self.lists.clear(true);
    }
}
