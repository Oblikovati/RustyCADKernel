//! Discrete wire polylines exercise the native segment classifier and area filter.
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmesh::{
    brep_mesh_data::DiscreteModel,
    imesh_data::{FaceHandle, MeshModel},
};
pub fn checker_model(v: &[i32]) -> (Handle<dyn MeshModel>, FaceHandle) {
    let (wire, face, ordered) = crate::wire_fixture::order_fixture(0, 0, 0);
    let model: Handle<dyn MeshModel> = Handle::new(DiscreteModel::new(&Shape::default())).cast();
    let face = model.add_face(&face).unwrap();
    face.set_deflection([0.0, 0.001, 0.03, 0.3][v[3] as usize]);
    if v[0] < 0 {
        return (model, face);
    }
    let mut points = checker_points(v[0]);
    if v[5] != 0 {
        points.reverse();
    }
    if v[6] != 0 {
        points.insert(1, points[0]);
    }
    append_wire(
        &model,
        &face,
        &wire,
        &ordered.edge(1).unwrap(),
        points,
        v[1] != 0,
    );
    if v[2] != 0 {
        let points = if v[2] == 1 {
            vec![Point2::new(-1.0, 0.5), Point2::new(2.0, 0.5)]
        } else {
            checker_points(0)
        };
        append_wire(
            &model,
            &face,
            &wire,
            &ordered.edge(2).unwrap(),
            points,
            v[1] != 0,
        );
    }
    (model, face)
}
fn append_wire(
    model: &Handle<dyn MeshModel>,
    face: &FaceHandle,
    wire: &Wire,
    topology: &rustycad_tkbrep::topods::Edge,
    points: Vec<Point2>,
    separate: bool,
) {
    let wire = face.add_wire(wire, points.len() as i32);
    if !separate {
        append_edge(model, face, &wire, topology, &points);
        return;
    }
    for pair in points.windows(2) {
        append_edge(model, face, &wire, topology, pair);
    }
}
fn append_edge(
    model: &Handle<dyn MeshModel>,
    face: &FaceHandle,
    wire: &rustycad_tkmesh::imesh_data::WireHandle,
    topology: &rustycad_tkbrep::topods::Edge,
    points: &[Point2],
) {
    let edge = model.add_edge(topology).unwrap();
    wire.add_edge(edge.downgrade(), Orientation::Forward);
    let curve = edge.add_pcurve(face.downgrade(), Orientation::Forward);
    for (index, point) in points.iter().enumerate() {
        curve.add_point(*point, index as f64);
    }
}
pub fn checker_points(kind: i32) -> Vec<Point2> {
    let values: Vec<(f64, f64)> = match kind {
        0 => vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)],
        1 => vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0), (1.0, 0.0), (0.0, 0.0)],
        2 => vec![
            (0.0, 0.0),
            (0.01, 0.01),
            (0.0, 0.01),
            (0.01, 0.0),
            (0.0, 0.0),
        ],
        3 => vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)],
        4 => vec![(0.0, 0.0), (2.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
        5 => vec![(0.0, 0.0), (2.0, 0.0), (1.0, 0.0), (3.0, 0.0)],
        6 => vec![(0.0, 0.0), (1.0, 0.01), (0.0, 0.01), (1.0, 0.0)],
        7 => vec![(0.0, 0.0), (1.0, 0.01), (1.0, 0.0), (0.0, 0.01)],
        _ => Vec::new(),
    };
    values.into_iter().map(|(x, y)| Point2::new(x, y)).collect()
}
