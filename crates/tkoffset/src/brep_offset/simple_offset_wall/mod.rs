// Translation of BuildWallFace at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod loft;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace, build_curves3d_default};

/// Build a connecting face while sharing wall edges at common source vertices.
/// `build_simple_offset_wall(&original, &offset, &mut vertex_edges)?`.
pub fn build_simple_offset_wall(
    original: &Edge,
    offset: &Edge,
    vertex_edges: &mut DataMap<Vertex, Edge>,
) -> Result<Face, Failure> {
    let original = Edge::unchecked(original.oriented(Orientation::Forward));
    let offset = Edge::unchecked(offset.oriented(Orientation::Reversed));
    let Some((wall1, wall2)) = connecting_edges(&original, &offset, vertex_edges)? else {
        return Ok(Face::default());
    };
    let wire = wall_wire([&original, &wall1, &offset, &wall2])?;
    if let Ok(maker) = BRepLibMakeFace::from_wire(&wire, true)
        && maker.is_done()
    {
        return maker.face();
    }
    loft::loft_wall(&original, &offset, &wall1, &wall2, &wire)
}

fn connecting_edges(
    original: &Edge,
    offset: &Edge,
    vertex_edges: &mut DataMap<Vertex, Edge>,
) -> Result<Option<(Edge, Edge)>, Failure> {
    let (first, last) = topexp::vertices(original, false);
    let (new_first, new_last) = topexp::vertices(offset, false);
    let Some(wall1) = connecting_edge(&last, &new_last, vertex_edges)? else {
        return Ok(None);
    };
    let Some(wall2) = connecting_edge(&first, &new_first, vertex_edges)? else {
        return Ok(None);
    };
    Ok(Some((
        wall1,
        Edge::unchecked(wall2.oriented(Orientation::Reversed)),
    )))
}

fn connecting_edge(
    original: &Vertex,
    offset: &Vertex,
    vertex_edges: &mut DataMap<Vertex, Edge>,
) -> Result<Option<Edge>, Failure> {
    let original = Vertex::unchecked(original.oriented(Orientation::Forward));
    if vertex_edges.is_bound(&original) {
        return Ok(Some(vertex_edges.find(&original)?.clone()));
    }
    let offset = Vertex::unchecked(offset.oriented(Orientation::Reversed));
    let maker = BRepLibMakeEdge::from_vertices(&original, &offset)?;
    if !maker.is_done() {
        return Ok(None);
    }
    let edge = maker.edge()?;
    vertex_edges.bind(original, edge.clone());
    Ok(Some(edge))
}

fn wall_wire(edges: [&Edge; 4]) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge)?;
    }
    build_curves3d_default(&wire)?;
    Ok(wire)
}
