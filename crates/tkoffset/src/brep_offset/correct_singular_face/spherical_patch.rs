use super::{CorrectedEdge, SphericalParameters, null_surface, replace_vertex};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomSphericalSurface, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

pub(super) fn build_spherical_patch(
    corrected: &[CorrectedEdge],
    sphere: &Handle<GeomSphericalSurface>,
) -> Result<Face, Failure> {
    let parameters = patch_parameters(corrected)?;
    let face = bounded_sphere_face(parameters, sphere)?;
    let old_edge = opposite_pole_edge(&face, parameters)?;
    let old_vertices = topexp::vertices(&old_edge, false);
    replace_patch_adjacent_vertices(&face, &old_edge, &old_vertices, corrected)?;
    replace_patch_wire_edge(&face, &old_edge, corrected)?;
    Ok(face)
}

fn patch_parameters(corrected: &[CorrectedEdge]) -> Result<SphericalParameters, Failure> {
    let first = corrected.first().ok_or_else(null_surface)?;
    let last = corrected.last().ok_or_else(null_surface)?;
    let u_first = corrected
        .iter()
        .map(|edge| edge.parameters.0)
        .fold(f64::INFINITY, f64::min);
    let u_last = corrected
        .iter()
        .map(|edge| edge.parameters.1)
        .fold(f64::NEG_INFINITY, f64::max);
    Ok((u_first, u_last, first.parameters.2, last.parameters.2))
}

fn bounded_sphere_face(
    parameters: SphericalParameters,
    sphere: &Handle<GeomSphericalSurface>,
) -> Result<Face, Failure> {
    let (u_first, u_last, v, _) = parameters;
    let v_bounds = sphere_v_bounds(v);
    let surface: Handle<dyn GeomSurface> = sphere.clone().cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (u_first, u_last, v_bounds.0, v_bounds.1),
        precision::confusion(),
    )?
    .face()
}

fn sphere_v_bounds(v: f64) -> (f64, f64) {
    if v > 0.0 {
        (v, std::f64::consts::FRAC_PI_2)
    } else {
        (-std::f64::consts::FRAC_PI_2, v)
    }
}

fn opposite_pole_edge(face: &Face, parameters: SphericalParameters) -> Result<Edge, Failure> {
    let (u_first, u_last, _, _) = parameters;
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let edge = Edge::unchecked(edges.current().clone());
        if is_equator_edge(&edge, u_first, u_last)? {
            return Ok(edge);
        }
        edges.next()?;
    }
    Err(null_surface())
}

fn is_equator_edge(edge: &Edge, u_first: f64, u_last: f64) -> Result<bool, Failure> {
    if BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    let (first, last) = BRepTool::range(edge)?;
    Ok((first - u_first).abs() <= precision::confusion()
        && (last - u_last).abs() <= precision::confusion())
}

fn replace_patch_adjacent_vertices(
    face: &Face,
    old_edge: &Edge,
    old_vertices: &(Vertex, Vertex),
    corrected: &[CorrectedEdge],
) -> Result<(), Failure> {
    let first = corrected.first().ok_or_else(null_surface)?;
    let last = corrected.last().ok_or_else(null_surface)?;
    for edge in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )? {
        replace_edge_vertices_if_adjacent(
            &Edge::unchecked(edge),
            old_edge,
            old_vertices,
            first,
            last,
        )?;
    }
    Ok(())
}

fn replace_edge_vertices_if_adjacent(
    edge: &Edge,
    old_edge: &Edge,
    old_vertices: &(Vertex, Vertex),
    corrected_first: &CorrectedEdge,
    corrected_last: &CorrectedEdge,
) -> Result<(), Failure> {
    if edge.is_same(old_edge) {
        return Ok(());
    }
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let (first, last) = topexp::vertices(&forward, false);
    replace_patch_edge_endpoints(
        &forward,
        (&first, &last),
        old_vertices,
        corrected_first,
        corrected_last,
    )
}

fn replace_patch_edge_endpoints(
    edge: &Edge,
    endpoints: (&Vertex, &Vertex),
    old_vertices: &(Vertex, Vertex),
    corrected_first: &CorrectedEdge,
    corrected_last: &CorrectedEdge,
) -> Result<(), Failure> {
    let (first, last) = endpoints;
    replace_patch_endpoint(
        edge,
        first,
        Orientation::Forward,
        old_vertices,
        corrected_first,
        corrected_last,
    )?;
    replace_patch_endpoint(
        edge,
        last,
        Orientation::Reversed,
        old_vertices,
        corrected_first,
        corrected_last,
    )
}

fn replace_patch_endpoint(
    edge: &Edge,
    vertex: &Vertex,
    orientation: Orientation,
    old_vertices: &(Vertex, Vertex),
    corrected_first: &CorrectedEdge,
    corrected_last: &CorrectedEdge,
) -> Result<(), Failure> {
    let Some(replacement) =
        patch_vertex_image(vertex, old_vertices, corrected_first, corrected_last)
    else {
        return Ok(());
    };
    replace_vertex(edge, vertex, &replacement.oriented(orientation))
}

fn patch_vertex_image<'a>(
    vertex: &Vertex,
    old_vertices: &(Vertex, Vertex),
    corrected_first: &'a CorrectedEdge,
    corrected_last: &'a CorrectedEdge,
) -> Option<&'a Vertex> {
    if vertex.is_same(&old_vertices.0) {
        return Some(&corrected_first.first_vertex);
    }
    if vertex.is_same(&old_vertices.1) {
        return Some(&corrected_last.last_vertex);
    }
    None
}

fn replace_patch_wire_edge(
    face: &Face,
    old_edge: &Edge,
    corrected: &[CorrectedEdge],
) -> Result<(), Failure> {
    for wire in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )? {
        replace_edge_in_wire(&wire, old_edge, corrected)?;
    }
    Ok(())
}

fn replace_edge_in_wire(
    wire: &Shape,
    old_edge: &Edge,
    corrected: &[CorrectedEdge],
) -> Result<(), Failure> {
    let mut wire = wire.clone();
    wire.set_free(true)?;
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    builder.remove(&mut wire, &old_edge.clone().into_shape())?;
    let orientation = old_edge.orientation();
    for edge in corrected {
        builder.add(&mut wire, &edge.edge.clone().oriented(orientation))?;
    }
    Ok(())
}
