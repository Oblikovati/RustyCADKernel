//! The vertex-sphere branch of `BRepOffset_Offset::Init`.
use rustycad_tkbrep::brep_tools::update_face;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::map_shapes_of_type,
    topods::{Edge, Face, Vertex, Wire},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomSphericalSurface, GeomTrimmedCurve},
    top_abs::Orientation,
};
use rustycad_tkgeombase::{geom_proj_lib, gprop::GProps};
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Vector2, Vector3,
};
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::{
    brep_gprop::BRepGProp,
    brep_lib::{BRepLibMakeWire, build_curves3d_default},
};
use std::f64::consts::PI;

/// Builds the spherical vertex-join face from offset edges; `build_vertex_arc_join(&v, &edges, 2.0)?`.
pub fn build_vertex_arc_join(
    vertex: &Vertex,
    edges: &[Edge],
    offset: f64,
) -> Result<Face, Failure> {
    let wire = fixed_wire(wire_of_edges(edges)?)?;
    let sphere = sphere_for_wire(vertex, &wire, offset)?;
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    let surface = sphere.cast();
    builder.make_face_on_surface(&mut face, &surface, BRepTool::vertex_tolerance(vertex)?)?;
    add_projected_edges(&mut face, &sphere, vertex, edges, offset)?;
    update_face(&face)?;
    Ok(face)
}

fn sphere_for_wire(
    vertex: &Vertex,
    wire: &Wire,
    offset: f64,
) -> Result<Handle<GeomSphericalSurface>, Failure> {
    let origin = BRepTool::pnt(vertex)?;
    let mut properties = GProps::new();
    BRepGProp::linear_properties(&wire.clone().into_shape(), &mut properties, false, false)?;
    let center = properties.centre_of_mass();
    let farthest = farthest_vertex(wire)?;
    let normal = frame_normal(origin, center, farthest)?;
    let x_direction = Direction3::new(
        origin.x() - center.x(),
        origin.y() - center.y(),
        origin.z() - center.z(),
    )?;
    let frame = Axis3::new(origin, normal, x_direction)?;
    Ok(Handle::new(GeomSphericalSurface::new(frame, offset.abs())?))
}

fn fixed_wire(wire: Wire) -> Result<Wire, Failure> {
    let fixer = ShapeFixShape::new(wire.into_shape());
    fixer.perform()?;
    let shape = fixer.shape().clone();
    Ok(Wire::unchecked(shape))
}

fn wire_of_edges(edges: &[Edge]) -> Result<Wire, Failure> {
    let mut edge_shapes = List::default();
    for edge in edges {
        edge_shapes.append(edge.clone().into_shape());
    }
    let mut maker = BRepLibMakeWire::new();
    maker.add_list(&edge_shapes)?;
    maker.wire()
}

fn farthest_vertex(wire: &Wire) -> Result<Point3, Failure> {
    let mut vertices = IndexedMapOfShape::default();
    map_shapes_of_type(&wire.clone().into_shape(), ShapeEnum::Vertex, &mut vertices)?;
    let mut points = Vec::new();
    for index in 1..=vertices.extent() {
        let vertex = Vertex::unchecked(vertices.find_key(index)?.clone());
        let point = BRepTool::pnt(&vertex)?;
        points.push(point);
    }
    farthest_pair_first(&points)
}

fn farthest_pair_first(points: &[Point3]) -> Result<Point3, Failure> {
    let mut result = Point3::default();
    let mut max_squared = 0.0;
    for first in points {
        for second in points {
            let squared = first.square_distance(second);
            if squared > max_squared {
                max_squared = squared;
                result = *first;
            }
        }
    }
    Ok(result)
}

fn frame_normal(origin: Point3, center: Point3, farthest: Point3) -> Result<Direction3, Failure> {
    let first = Vector3::between(origin, center);
    let second = Vector3::between(origin, farthest);
    let normal = first.crossed(second);
    Direction3::new(normal.x(), normal.y(), normal.z())
}

fn add_projected_edges(
    face: &mut Face,
    sphere: &Handle<GeomSphericalSurface>,
    vertex: &Vertex,
    edges: &[Edge],
    offset: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for source in edges {
        add_projected_edge(face, sphere, vertex, source, &mut wire)?;
    }
    attach_join_wire(face, wire, offset)
}

fn add_projected_edge(
    face: &Face,
    sphere: &Handle<GeomSphericalSurface>,
    vertex: &Vertex,
    edge: &Edge,
    wire: &mut Wire,
) -> Result<(), Failure> {
    let (curve, first, last) = edge_curve(edge)?;
    let trimmed = trimmed_curve(curve, first, last)?;
    let pcurve = geom_proj_lib::curve2d(&trimmed, &sphere.clone().cast())?;
    adjust_sphere_pcurve(&pcurve, first)?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, &pcurve, face, BRepTool::vertex_tolerance(vertex)?)?;
    builder.range_on_face(edge, face, first, last)?;
    builder.add(wire, &edge.clone().into_shape())?;
    Ok(())
}

fn attach_join_wire(face: &mut Face, wire: Wire, offset: f64) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let orientation = if offset < 0.0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let wire_shape = wire.oriented(orientation);
    builder.add(face, &wire_shape)?;
    if offset < 0.0 {
        face.reverse();
    }
    Ok(())
}

fn edge_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    if found.curve.is_null() {
        build_curves3d_default(&edge.clone().into_shape())?;
    }
    let found = BRepTool::curve(edge)?;
    if found.0.is_null() {
        return Err(not_implemented(
            "vertex join edge has no 3D curve after BuildCurve3d",
        ));
    }
    Ok(found)
}

fn trimmed_curve(
    curve: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast())
}

fn adjust_sphere_pcurve(
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    first: f64,
) -> Result<(), Failure> {
    let point: Point2 = curve.value(first)?;
    if point.y() < -PI / 2.0 {
        curve.mirror_axis(Axis2d::new(Point2::new(0.0, -PI / 2.0), Direction2::X))?;
        curve.translate(Vector2::new(if point.x() > PI { -PI } else { PI }, 0.0))?;
    } else if point.y() > PI / 2.0 {
        curve.mirror_axis(Axis2d::new(Point2::new(0.0, PI / 2.0), Direction2::X))?;
        curve.translate(Vector2::new(if point.x() > PI { -PI } else { PI }, 0.0))?;
    }
    Ok(())
}

fn not_implemented(detail: &str) -> Failure {
    Failure::with_type("Standard_NotImplemented", detail, "BRepOffset_Offset")
}
