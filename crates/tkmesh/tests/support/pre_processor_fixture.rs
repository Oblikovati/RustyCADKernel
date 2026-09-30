//! Existing polygons and real conical topology supplied to native preprocessing.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{message::ProgressRange, standard::Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::poly::{Triangle, TriangulationParameters, mesh_purpose};
use rustycad_tkmesh::{
    brep_mesh::{BRepMeshEdgeDiscret, BRepMeshModelBuilder, BRepMeshModelPostProcessor},
    imesh_data::{MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo, ModelBuilder},
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCone;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
pub fn existing_model(v: &[i32]) -> Handle<dyn MeshModel> {
    let model = super::post_fixture::post_model(&[v[0], 0, 0, 3, 2, 1, 0]);
    if model.is_null() {
        return model;
    }
    let parameters = MeshParameters {
        min_size: 0.001,
        deflection: 0.1,
        ..Default::default()
    };
    BRepMeshModelPostProcessor::new().perform(&model, &parameters, &ProgressRange::default());
    model.get_edge(0).unwrap().status().set_status(Status(v[1]));
    for index in 0..model.faces_nb() {
        prepare_face(&model, index, v);
    }
    model
}
fn prepare_face(model: &Handle<dyn MeshModel>, index: i32, v: &[i32]) {
    let face = model.get_face(index).unwrap();
    let (wire, _, _) = crate::wire_fixture::order_fixture(0, 0, 0);
    face.add_wire(&wire, 1)
        .add_edge(model.get_edge(0).unwrap().downgrade(), Orientation::Forward);
    face.status().set_status(Status(v[2]));
    face.set_deflection([0.01, 0.1, 10.0][v[3] as usize]);
    let (mesh, _) = BRepTool::triangulation(&face.get_face().unwrap(), mesh_purpose::NONE).unwrap();
    mesh.set_deflection(0.1);
    mesh.set_parameters(source_parameters(v[4]));
    mesh.set_triangle(1, Triangle::new([1, 0, 4][v[5] as usize], 2, 3))
        .unwrap();
}
fn source_parameters(kind: i32) -> Handle<TriangulationParameters> {
    if kind == 0 {
        return Handle::default();
    }
    Handle::new(TriangulationParameters::new(
        if kind == 1 { 0.01 } else { -1.0 },
        0.5,
        0.001,
    ))
}

pub fn cone_model(kind: i32, parameters: &MeshParameters) -> Handle<dyn MeshModel> {
    let mut builder =
        BRepPrimAPIMakeCone::new(5.0, if kind == 0 { 0.0 } else { 2.0 }, 10.0).unwrap();
    let model = BRepMeshModelBuilder::new().perform(builder.shape().unwrap(), parameters);
    BRepMeshEdgeDiscret::new().perform(&model, parameters, &ProgressRange::default());
    model
}
pub fn alter_cone(model: &Handle<dyn MeshModel>, mode: i32) {
    use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
    for index in 0..model.faces_nb() {
        let face = model.get_face(index).unwrap();
        if face.get_surface().get_type().unwrap() != rustycad_tkmath::geom_abs::SurfaceType::Cone {
            continue;
        }
        if mode == 5 {
            face.status().set_status(Status::Failure);
        }
        let wire = face.get_wire(0).unwrap();
        for index in 0..wire.edges_nb() {
            let edge = wire.get_edge(index).unwrap().upgrade();
            if edge
                .get_pcurve_for(&face.downgrade(), Orientation::Forward)
                .unwrap()
                == edge
                    .get_pcurve_for(&face.downgrade(), Orientation::Reversed)
                    .unwrap()
            {
                continue;
            }
            alter_seam(&edge, &face, mode);
            return;
        }
    }
}
fn alter_seam(
    edge: &rustycad_tkmesh::imesh_data::EdgeHandle,
    face: &rustycad_tkmesh::imesh_data::FaceHandle,
    mode: i32,
) {
    if mode == 1 {
        reverse_seam(edge);
    }
    if mode == 2 {
        rustycad_tkbrep::brep::BRepBuilder::default()
            .update_edge_pcurve_on_face(
                &edge.get_edge().unwrap(),
                &Handle::default(),
                &face.get_face().unwrap(),
                1e-7,
            )
            .unwrap();
    }
    if mode == 3 {
        let p = edge.get_pcurve(0).unwrap();
        let first = p.get_point(0).unwrap();
        p.set_point(p.parameters_nb() - 1, first).unwrap();
    }
    if mode == 4 {
        let c = edge.get_curve();
        c.insert_point(1, c.get_point(0).unwrap(), c.get_parameter(0).unwrap())
            .unwrap();
    }
}
fn reverse_seam(edge: &rustycad_tkmesh::imesh_data::EdgeHandle) {
    let curve = edge.get_curve();
    let points: Vec<_> = (0..curve.parameters_nb())
        .map(|i| (curve.get_point(i).unwrap(), curve.get_parameter(i).unwrap()))
        .collect();
    for (index, (point, parameter)) in points.into_iter().rev().enumerate() {
        curve.set_point(index as i32, point).unwrap();
        curve.set_parameter(index as i32, parameter).unwrap();
    }
    for index in 0..edge.pcurves_nb() {
        let pcurve = edge.get_pcurve(index).unwrap();
        let points: Vec<_> = (0..pcurve.parameters_nb())
            .map(|i| {
                (
                    pcurve.get_point(i).unwrap(),
                    pcurve.get_parameter(i).unwrap(),
                )
            })
            .collect();
        for (index, (point, parameter)) in points.into_iter().rev().enumerate() {
            pcurve.set_point(index as i32, point).unwrap();
            pcurve.set_parameter(index as i32, parameter).unwrap();
        }
    }
}
