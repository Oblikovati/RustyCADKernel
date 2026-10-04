use super::wire_intersection::intersection_distance;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp,
    topods::{Edge, Face, Shape, Vertex},
    toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape},
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::{
    geom::GeomSurface,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::Point3;
pub(super) struct EdgeSample {
    pub edge: Edge,
    pub parameter: f64,
    pub closed: bool,
    pub adaptor: Geom2dAdaptorCurve,
    pub range: (f64, f64),
}
pub(super) fn correct_wires(face: &Face, avoided: &IndexedMapOfShape) -> Result<(), Failure> {
    let mut forward = face.clone();
    forward.set_orientation(Orientation::Forward);
    let surface = BRepTool::surface(face)?;
    let mut adjacent = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_unique_ancestors(
        &forward,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut adjacent,
        true,
    )?;
    let mut lengths = Vec::new();
    for index in 1..=adjacent.extent() {
        let vertex = Vertex::unchecked(adjacent.find_key(index)?.clone());
        correct_vertex(
            &vertex,
            &forward,
            &surface,
            adjacent.find_from_index(index)?,
            avoided,
            &mut lengths,
        )?;
    }
    Ok(())
}
fn correct_vertex(
    vertex: &Vertex,
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    edges: &List<Shape>,
    avoided: &IndexedMapOfShape,
    lengths: &mut Vec<(Shape, f64)>,
) -> Result<(), Failure> {
    let point = BRepTool::pnt(vertex)?;
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    let (samples, mut maximum) = sample_edges(vertex, face, surface, edges, point)?;
    for (index, first) in samples.iter().enumerate() {
        for second in &samples[index + 1..] {
            let distance = intersection_distance(vertex, surface, first, second, lengths)?;
            if distance > maximum {
                maximum = distance;
            }
        }
    }
    if maximum > tolerance * tolerance && !avoided.contains(vertex) {
        BRepBuilder::default().update_vertex_tolerance(vertex, 1.01 * maximum.sqrt())?;
    }
    Ok(())
}
fn sample_edges(
    vertex: &Vertex,
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    edges: &List<Shape>,
    point: Point3,
) -> Result<(Vec<EdgeSample>, f64), Failure> {
    let mut samples = Vec::new();
    let mut maximum = -1.;
    for shape in edges.iter() {
        let edge = Edge::unchecked(shape.clone());
        let found = BRepTool::curve_on_face(&edge, face)?;
        if found.pcurve.is_null() {
            continue;
        }
        let parameter = BRepTool::parameter(vertex, &edge)?;
        let (first, last) = topexp::vertices(&edge, false);
        let uv = found.pcurve.value(parameter)?;
        let distance = point.square_distance(&surface.value(uv.x(), uv.y())?);
        if distance > maximum {
            maximum = distance;
        }
        samples.push(EdgeSample {
            edge,
            parameter,
            closed: first.is_same(&last),
            adaptor: Geom2dAdaptorCurve::with_curve(&found.pcurve)?,
            range: (found.first, found.last),
        });
    }
    Ok((samples, maximum))
}
