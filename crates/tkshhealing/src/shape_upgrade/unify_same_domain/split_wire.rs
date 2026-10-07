//! `SplitWire` of ShapeUpgrade_UnifySameDomain.cxx: a wire through splitting vertices cut
//! into the loops starting at each of them, turning right at the branchings.
use super::geometry::null_pcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector2;

type VertexEdges = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// The forward and reversed edges of the wire (each once) by their first vertex.
fn edges_by_first_vertex(wire: &Wire) -> Result<VertexEdges, Failure> {
    let mut map = VertexEdges::default();
    let mut seen = MapOfShape::default();
    for shape in ShapeIterator::new(wire, true, true) {
        if !seen.add(shape.clone()) {
            continue;
        }
        if !matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            continue;
        }
        let vertex = topexp::first_vertex(&topods::edge(&shape)?, true).into_shape();
        match map.change_seek(&vertex) {
            Some(list) => list.push(shape),
            None => {
                map.bind(vertex, vec![shape]);
            }
        }
    }
    Ok(map)
}

/// The tangent of the p-curve at the end (or start) of the edge, in the edge direction.
fn edge_direction(edge: &Edge, face: &Face, at_end: bool) -> Result<Option<Vector2>, Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(curve) = found.pcurve.get() else {
        return Ok(None);
    };
    let forward = edge.orientation() == Orientation::Forward;
    let parameter = if forward == at_end {
        found.last
    } else {
        found.first
    };
    let mut direction = curve.eval_d1(parameter)?.d1;
    direction.normalize()?;
    if edge.orientation() == Orientation::Reversed {
        direction.reverse();
    }
    Ok(Some(direction))
}

/// The branch with the smallest angle to the current direction ("the rightest"), the
/// first one when none has a p-curve; removed from the list.
fn rightest_branch(list: &mut Vec<Shape>, current: Vector2, face: &Face) -> Result<Shape, Failure> {
    let mut min_angle = f64::MAX;
    let mut next: Option<Shape> = None;
    for shape in list.iter() {
        let Some(direction) = edge_direction(&topods::edge(shape)?, face, false)? else {
            continue;
        };
        let angle = current.angle(direction)?;
        if angle < min_angle {
            min_angle = angle;
            next = Some(shape.clone());
        }
    }
    let next = next.unwrap_or_else(|| list[0].clone());
    if let Some(position) = list.iter().position(|s| next.is_same(s)) {
        list.remove(position);
    }
    Ok(next)
}

/// The edge following `current` at its last vertex, removed from the map; None at the
/// origin or a dead end.
fn next_branch(
    current: &Edge,
    origin: &Shape,
    map: &mut VertexEdges,
    face: &Face,
) -> Result<Option<Shape>, Failure> {
    let vertex = topexp::last_vertex(current, true).into_shape();
    if vertex.is_same(origin) {
        return Ok(None);
    }
    let Some(list) = map.change_seek(&vertex) else {
        return Ok(None);
    };
    if list.is_empty() {
        return Ok(None);
    }
    let has_pcurve = !BRepTool::curve_on_face(current, face)?.pcurve.is_null();
    if list.len() == 1 || !has_pcurve {
        return Ok(Some(list.remove(0)));
    }
    let direction = edge_direction(current, face, true)?.ok_or_else(null_pcurve)?;
    Ok(Some(rightest_branch(list, direction, face)?))
}

/// `SplitWire(theWire, theFace, theVmap, theWireSeq)`: from each splitting vertex, each
/// branch followed until it comes back to the vertex or stops, as a new wire.
pub(super) fn split_wire(
    wire: &Wire,
    face: &Face,
    vertices: &IndexedMapOfShape,
    wires: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut map = edges_by_first_vertex(wire)?;
    let builder = BRepBuilder::default();
    for index in 1..=vertices.extent() {
        let origin = vertices.find_key(index)?.clone();
        loop {
            let branches = map
                .change_seek(&origin)
                .ok_or_else(|| no_branches(&origin))?;
            if branches.is_empty() {
                break;
            }
            let mut current = topods::edge(&branches.remove(0))?;
            let mut new_wire = Wire::default();
            builder.make_wire(&mut new_wire);
            loop {
                builder.add(&mut new_wire, &current)?;
                let Some(next) = next_branch(&current, &origin, &mut map, face)? else {
                    break;
                };
                current = topods::edge(&next)?;
            }
            wires.push(new_wire.into_shape());
        }
    }
    Ok(())
}

fn no_branches(vertex: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "ShapeUpgrade_UnifySameDomain::SplitWire: vertex {:?} starts no edge of the wire; expected a splitting vertex of it",
            vertex.shape_type().ok()
        ),
        "",
    )
}
