//! Explicit discrete input for polygon publication, paired with the native oracle.
use crate::wire_fixture;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Point2, Point3},
    poly::{Triangle, Triangulation},
};
use rustycad_tkmesh::{
    brep_mesh::ShapeTool,
    brep_mesh_data::DiscreteModel,
    imesh_data::{MeshModel, Status},
};
pub fn post_model(v: &[i32]) -> Handle<dyn MeshModel> {
    if v[0] < 0 {
        return Handle::default();
    }
    let (_, _, ordered) = wire_fixture::order_fixture(0, 0, 0);
    let model: Handle<dyn MeshModel> = Handle::new(DiscreteModel::new(&Shape::default())).cast();
    let edge = model.add_edge(&ordered.edge(1).unwrap()).unwrap();
    edge.set_deflection(0.125);
    edge.status().set_status(Status(v[1]));
    for index in 0..v[3] {
        edge.get_curve().add_point(
            Point3::new(f64::from(index), 0.0, 0.0),
            f64::from(index) * 0.5,
        );
    }
    if v[0] == 0 {
        return model;
    }
    populate_post_faces(&model, &edge, v);
    model
}
fn populate_post_faces(
    model: &Handle<dyn MeshModel>,
    edge: &rustycad_tkmesh::imesh_data::EdgeHandle,
    v: &[i32],
) {
    for _ in 0..v[4] {
        let (_, face, _) = wire_fixture::order_fixture(0, 0, 0);
        let discrete = model.add_face(&face).unwrap();
        discrete.status().set_status(Status(v[2]));
        if v[5] != 0 {
            ShapeTool::add_in_face(&face, &post_triangulation()).unwrap();
        }
        for index in 0..v[0] {
            add_post_pcurve(edge, &discrete, index);
        }
    }
}
fn add_post_pcurve(
    edge: &rustycad_tkmesh::imesh_data::EdgeHandle,
    face: &rustycad_tkmesh::imesh_data::FaceHandle,
    offset: i32,
) {
    let orientation = if offset == 1 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let curve = edge.add_pcurve(face.downgrade(), orientation);
    for index in 0..2 {
        curve.add_point(
            Point2::new(f64::from(index), 0.0),
            f64::from(index) + f64::from(offset) * 0.1,
        );
        curve.set_index(index, index + 1 + offset % 2).unwrap();
    }
}
fn post_triangulation() -> Handle<Triangulation> {
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, true, false).unwrap());
    for (index, (x, y)) in [(0.0, 0.0), (2.0, 0.0), (0.0, 3.0)].into_iter().enumerate() {
        mesh.set_node(index as i32 + 1, Point3::new(x, y, 0.3))
            .unwrap();
        mesh.set_uv_node(index as i32 + 1, Point2::new(x, y))
            .unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh.set_deflection(7.0);
    mesh
}
