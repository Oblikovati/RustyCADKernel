//! Corrects non-collapsed offset images of degenerated boundary edges.
mod spherical_patch;

use super::BRepOffsetOffsetFace;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve2d,
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomSphericalSurface, GeomSurface, GeomTrimmedCurve},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3, Vector3};
use rustycad_tktopalgo::brep_lib::same_parameter_edge;
use spherical_patch::build_spherical_patch;

pub(super) fn correct_singular_face(
    offset: &BRepOffsetOffsetFace,
    distance: f64,
) -> Result<Vec<Face>, Failure> {
    let mut corrected = Vec::new();
    for (source_edge, image_edge) in offset.edge_images() {
        append_corrected_edge(
            &mut corrected,
            offset.face(),
            source_edge,
            image_edge,
            distance,
        )?;
    }
    Ok(corrected)
}

fn append_corrected_edge(
    corrected: &mut Vec<Face>,
    face: &Face,
    source_edge: &Edge,
    image_edge: &Edge,
    distance: f64,
) -> Result<(), Failure> {
    if !BRepTool::degenerated(source_edge)? || !is_noncollapsed(face, image_edge)? {
        return Ok(());
    }
    let apex = first_vertex(source_edge);
    corrected.push(correct_singular_edge(image_edge, &apex, face, distance)?);
    Ok(())
}

fn first_vertex(edge: &Edge) -> Vertex {
    rustycad_tkbrep::topexp::vertices(edge, false).0
}

fn is_noncollapsed(face: &Face, edge: &Edge) -> Result<bool, Failure> {
    if !BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    let (first, mid, last, _, _) = edge_surface_points(face, edge)?;
    // CorrectConicalFaces uses TolApex=1.e-5 to distinguish a real collapsed pole.
    Ok(first.distance(&last) > 1.0e-5 || first.distance(&mid) > 1.0e-5)
}

fn correct_singular_edge(
    image_edge: &Edge,
    apex: &Vertex,
    face: &Face,
    distance: f64,
) -> Result<Face, Failure> {
    let points = edge_surface_points(face, image_edge)?;
    let old_vertices = topexp::vertices(image_edge, false);
    let sphere = make_apex_sphere(apex, points, distance)?;
    let corrected = rebuild_singular_edge(image_edge, &sphere, points)?;
    replace_adjacent_vertices(face, image_edge, &old_vertices, &corrected, points)?;
    build_spherical_patch(&corrected, &sphere)
}

type EdgeSurfacePoints = (Point3, Point3, Point3, Point2, Point2);

pub(super) type SphericalParameters = (f64, f64, f64, f64);

pub(super) struct CorrectedEdge {
    pub(super) edge: Edge,
    pub(super) first_vertex: Vertex,
    pub(super) last_vertex: Vertex,
    pub(super) parameters: SphericalParameters,
}

fn edge_surface_points(face: &Face, edge: &Edge) -> Result<EdgeSurfacePoints, Failure> {
    let curve = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let adaptor = curve.geom2d_adaptor();
    let first = adaptor.first_parameter()?;
    let last = adaptor.last_parameter()?;
    let pcurve_first = adaptor.value(first)?;
    let pcurve_last = adaptor.value(last)?;
    let pcurve_mid = adaptor.value((first + last) / 2.0)?;
    let surface_handle = BRepTool::surface(face)?;
    let surface = surface_handle.get().ok_or_else(null_surface)?;
    let first3d = surface.value(pcurve_first.x(), pcurve_first.y())?;
    let last3d = surface.value(pcurve_last.x(), pcurve_last.y())?;
    let mid3d = surface.value(pcurve_mid.x(), pcurve_mid.y())?;
    Ok((first3d, mid3d, last3d, pcurve_first, pcurve_last))
}

fn make_apex_sphere(
    apex: &Vertex,
    points: EdgeSurfacePoints,
    distance: f64,
) -> Result<Handle<GeomSphericalSurface>, Failure> {
    let origin = BRepTool::pnt(apex)?;
    let (first, mid, last, _, _) = points;
    let first_leg = Vector3::between(first, mid);
    let second_leg = Vector3::between(first, last);
    let normal = first_leg.crossed(second_leg);
    let apex_leg = Vector3::between(origin, first);
    let y_direction = normal.crossed(apex_leg);
    let x_direction = y_direction.crossed(normal);
    let frame = Axis3::new(origin, direction(&normal)?, direction(&x_direction)?)?;
    Ok(Handle::new(GeomSphericalSurface::new(frame, distance)?))
}

fn direction(vector: &Vector3) -> Result<Direction3, Failure> {
    Direction3::new(vector.x(), vector.y(), vector.z())
}

pub(super) fn null_surface() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "offset correction face has no surface",
    )
}

fn rebuild_singular_edge(
    edge: &Edge,
    sphere: &Handle<GeomSphericalSurface>,
    points: EdgeSurfacePoints,
) -> Result<CorrectedEdge, Failure> {
    let (first, _, last, _, _) = points;
    let parameters = sphere_parameters(sphere, first, last)?;
    let (first_vertex, last_vertex) = make_spherical_vertices(first, last)?;
    replace_edge_vertices(edge, &first_vertex, &last_vertex)?;
    update_spherical_edge(edge, sphere, parameters)?;
    Ok(CorrectedEdge {
        edge: edge.clone(),
        first_vertex,
        last_vertex,
        parameters,
    })
}

fn sphere_parameters(
    sphere: &Handle<GeomSphericalSurface>,
    first: Point3,
    last: Point3,
) -> Result<SphericalParameters, Failure> {
    let gp_sphere = sphere.sphere()?;
    let (u_first, v) = elslib::parameters(&gp_sphere, first);
    let (mut u_last, _) = elslib::parameters(&gp_sphere, last);
    if u_last.abs() <= precision::confusion() {
        u_last = std::f64::consts::TAU;
    }
    Ok((u_first, u_last, v, v))
}

fn make_spherical_vertices(first: Point3, last: Point3) -> Result<(Vertex, Vertex), Failure> {
    let mut first_vertex = Vertex::default();
    let mut last_vertex = Vertex::default();
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    builder.make_vertex_at(&mut first_vertex, first, precision::confusion())?;
    if first.distance(&last) <= precision::confusion() {
        return Ok((first_vertex.clone(), first_vertex));
    }
    builder.make_vertex_at(&mut last_vertex, last, precision::confusion())?;
    Ok((first_vertex, last_vertex))
}

fn replace_edge_vertices(edge: &Edge, first: &Vertex, last: &Vertex) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let (old_first, old_last) = topexp::vertices(edge, false);
    let mut edge_shape = edge.clone().into_shape();
    edge_shape.set_free(true)?;
    builder.remove(&mut edge_shape, &old_first.clone().into_shape())?;
    if !old_last.is_same(&old_first) {
        builder.remove(&mut edge_shape, &old_last.into_shape())?;
    }
    builder.add(&mut edge_shape, &first.oriented(Orientation::Forward))?;
    builder.add(&mut edge_shape, &last.oriented(Orientation::Reversed))
}

fn update_spherical_edge(
    edge: &Edge,
    sphere: &Handle<GeomSphericalSurface>,
    parameters: SphericalParameters,
) -> Result<(), Failure> {
    set_singular_edge_flags(edge)?;
    let (curve, pcurve, surface, location) = spherical_edge_geometry(sphere, parameters)?;
    set_spherical_curve(edge, &curve, parameters)?;
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    builder.update_edge_pcurve(
        edge,
        &pcurve,
        &surface,
        &location,
        precision::confusion(),
        None,
    )?;
    builder.range_on_surface(edge, &surface, &location, parameters.0, parameters.1)?;
    same_parameter_edge(edge, precision::confusion())
}

fn set_spherical_curve(
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
    parameters: SphericalParameters,
) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    builder.update_edge_curve(edge, curve, precision::confusion())?;
    builder.range(edge, parameters.0, parameters.1, true)
}

fn set_singular_edge_flags(edge: &Edge) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    builder.degenerated(edge, false)?;
    builder.same_range(edge, false)?;
    builder.same_parameter(edge, false)
}

fn spherical_edge_geometry(
    sphere: &Handle<GeomSphericalSurface>,
    parameters: SphericalParameters,
) -> Result<SphericalEdgeGeometry, Failure> {
    let (u_first, u_last, v, _) = parameters;
    let curve = sphere.v_iso(v)?;
    let curve = GeomTrimmedCurve::new(&curve, u_first, u_last, true, true)?;
    let line = Handle::new(Geom2dLine::from_point(Point2::new(0.0, v), Direction2::X))
        .cast::<dyn Geom2dCurve>();
    let pcurve = Geom2dTrimmedCurve::new(&line, u_first, u_last, true, false)?;
    Ok((
        Handle::new(curve).cast(),
        Handle::new(pcurve).cast(),
        sphere.clone().cast(),
        rustycad_tkmath::toploc::Location::default(),
    ))
}

type SphericalEdgeGeometry = (
    Handle<dyn GeomCurve>,
    Handle<dyn Geom2dCurve>,
    Handle<dyn GeomSurface>,
    rustycad_tkmath::toploc::Location,
);

fn replace_adjacent_vertices(
    face: &Face,
    singular_edge: &Edge,
    old_vertices: &(Vertex, Vertex),
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    let (old_first, old_last) = old_vertices;
    replace_edges_on_face(face, singular_edge, old_first, old_last, corrected, points)
}

fn replace_edges_on_face(
    face: &Face,
    singular_edge: &Edge,
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    let edges = face_edges(face)?;
    replace_adjacent_edges(
        face,
        edges,
        singular_edge,
        old_first,
        old_last,
        corrected,
        points,
    )
}

fn face_edges(face: &Face) -> Result<Explorer, Failure> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
}

fn replace_adjacent_edges(
    face: &Face,
    edges: Explorer,
    singular_edge: &Edge,
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    for edge in edges {
        replace_listed_adjacent(
            face,
            edge,
            singular_edge,
            old_first,
            old_last,
            corrected,
            points,
        )?;
    }
    Ok(())
}

fn replace_listed_adjacent(
    face: &Face,
    edge: Shape,
    singular_edge: &Edge,
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    replace_adjacent_if_needed(
        face,
        singular_edge,
        &Edge::unchecked(edge),
        old_first,
        old_last,
        corrected,
        points,
    )
}

fn replace_adjacent_if_needed(
    face: &Face,
    singular_edge: &Edge,
    edge: &Edge,
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    if edge.is_same(singular_edge) || BRepTool::degenerated(edge)? {
        return Ok(());
    }
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    replace_adjacent_edge_vertex(face, &forward, old_first, old_last, corrected, points)
}

fn replace_adjacent_edge_vertex(
    face: &Face,
    edge: &Edge,
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    let (edge_first, edge_last) = topexp::vertices(edge, false);
    let endpoints = [
        (&edge_first, Orientation::Forward),
        (&edge_last, Orientation::Reversed),
    ];
    replace_matching_endpoints(
        face, edge, endpoints, old_first, old_last, corrected, points,
    )
}

fn replace_matching_endpoints(
    face: &Face,
    edge: &Edge,
    endpoints: [(&Vertex, Orientation); 2],
    old_first: &Vertex,
    old_last: &Vertex,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    for (vertex, orientation) in endpoints {
        if vertex.is_same(old_first) || vertex.is_same(old_last) {
            replace_adjacent_endpoint(face, edge, vertex, orientation, corrected, points)?;
        }
    }
    Ok(())
}

fn replace_adjacent_endpoint(
    face: &Face,
    edge: &Edge,
    old_vertex: &Vertex,
    orientation: Orientation,
    corrected: &CorrectedEdge,
    points: EdgeSurfacePoints,
) -> Result<(), Failure> {
    let point = adjacent_endpoint_uv(face, edge, orientation)?;
    let target = corrected_vertex_at_uv(point, points, corrected);
    replace_vertex(edge, old_vertex, &target.oriented(orientation))
}

fn adjacent_endpoint_uv(
    face: &Face,
    edge: &Edge,
    orientation: Orientation,
) -> Result<Point2, Failure> {
    let adaptor = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
    let adaptor = adaptor.geom2d_adaptor();
    let parameter = if orientation == Orientation::Forward {
        adaptor.first_parameter()?
    } else {
        adaptor.last_parameter()?
    };
    adaptor.value(parameter)
}

fn corrected_vertex_at_uv(
    point: Point2,
    points: EdgeSurfacePoints,
    corrected: &CorrectedEdge,
) -> &Vertex {
    let (_, _, _, first_pcurve, _) = points;
    if point.distance(first_pcurve) <= precision::confusion() {
        &corrected.first_vertex
    } else {
        &corrected.last_vertex
    }
}

pub(super) fn replace_vertex(edge: &Edge, old: &Vertex, new: &Shape) -> Result<(), Failure> {
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let mut edge_shape = edge.clone().oriented(Orientation::Forward);
    edge_shape.set_free(true)?;
    builder.remove(&mut edge_shape, &old.clone().into_shape())?;
    builder.add(&mut edge_shape, new)
}
