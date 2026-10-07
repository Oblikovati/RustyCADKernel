//! TKMesh GTest that builds its fixture with the TKBO Boolean fuse: translation of
//! BRepMesh_IncrementalMeshTest.BooleanTrimmedTorus_CrossingSeamProducesDenseMesh from
//! src/ModelingAlgorithms/TKMesh/GTests/BRepMesh_IncrementalMesh_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::brep_mesh::{BRepMeshDiscretRoot, BRepMeshIncrementalMesh};
use rustycad_tkmesh::imesh_tools::MeshParameters;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeRevol};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::PI;

/// The ring revolved from a circular profile of radius 17.5 that crosses the box end.
fn trimmed_torus_ring() -> Shape {
    let frame = Axis2::new(
        Point3::new(200.0, -20.0, 0.0),
        Direction3::new(0.0, -1.0, 0.0).unwrap(),
    )
    .unwrap();
    let edge_builder =
        BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, 17.5).unwrap()).unwrap();
    assert!(edge_builder.is_done());
    let wire_builder = BRepBuilderAPIMakeWire::from_edge(&edge_builder.edge().unwrap()).unwrap();
    assert!(wire_builder.is_done());
    let face_builder =
        BRepBuilderAPIMakeFace::from_wire(&wire_builder.wire().unwrap(), true).unwrap();
    assert!(face_builder.is_done());
    let axis = Axis1::new(
        Point3::new(0.0, -20.0, 0.0),
        Direction3::new(0.0, 0.0, -1.0).unwrap(),
    );
    let profile = face_builder.face().unwrap().into_shape();
    let mut ring_builder = BRepPrimAPIMakeRevol::new(&profile, axis, 2.0 * PI, false).unwrap();
    let ring = ring_builder.shape().unwrap().clone();
    assert!(ring_builder.is_done());
    ring
}

/// The face, node and triangle counts over the toroidal faces of the meshed shape.
fn torus_mesh_counts(shape: &Shape) -> (usize, i32, i32) {
    let (mut faces, mut nodes, mut triangles) = (0, 0, 0);
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        let face = topods::face(explorer.current()).unwrap();
        explorer.next().unwrap();
        if BRepAdaptorSurface::with_face(&face, true)
            .unwrap()
            .get_type()
            .unwrap()
            != SurfaceType::Torus
        {
            continue;
        }
        faces += 1;
        let (triangulation, _) = BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
        let triangulation = triangulation.get().expect("the toroidal face is meshed");
        nodes += triangulation.nb_nodes();
        triangles += triangulation.nb_triangles();
    }
    (faces, nodes, triangles)
}

#[test]
fn BRepMesh_IncrementalMeshTest__BooleanTrimmedTorus_CrossingSeamProducesDenseMesh() {
    let mut box_builder = BRepPrimAPIMakeBox::from_points(
        Point3::new(-187.0, -20.0, -67.5),
        Point3::new(187.0, 0.0, 67.5),
    )
    .unwrap();
    let box_shape = box_builder.shape().unwrap().clone();
    assert!(box_builder.is_done());
    let mut fuse = BRepAlgoAPIFuse::from_shapes(&box_shape, &trimmed_torus_ring()).unwrap();
    fuse.build().unwrap();
    assert!(fuse.is_done());
    let fused = fuse.shape().clone();
    assert!(!fused.is_null());
    assert!(
        BRepCheckAnalyzer::new(&fused, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let parameters = MeshParameters {
        deflection: 0.1,
        angle: 0.4,
        ..Default::default()
    };
    let mesher =
        BRepMeshIncrementalMesh::with_parameters(&fused, &parameters, &ProgressRange::default())
            .unwrap();
    assert!(mesher.is_done());
    let (faces, nodes, triangles) = torus_mesh_counts(&fused);
    assert!(faces > 0);
    // The regression produced fewer than 450 triangles for the whole fused shape.
    assert!(nodes > 1000, "{nodes} torus nodes");
    assert!(triangles > 1000, "{triangles} torus triangles");
}
