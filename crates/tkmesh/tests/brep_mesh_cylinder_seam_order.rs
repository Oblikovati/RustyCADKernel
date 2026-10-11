//! Port-only probe: the seam node copy of a meshed cylinder keeps the pinned native order.
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
#[path = "support/triangulation_hash.rs"]
mod triangulation_hash;
use triangulation_hash::cylinder_node_hash;

#[test]
fn cylinder_seam_copy_matches_pinned_native_order() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 10.0);
    let shape = maker.shape().unwrap().clone();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.5,
            ..Default::default()
        },
        &ProgressRange::default(),
    );
    result.unwrap();
    let faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let face = rustycad_tkbrep::topods::face(faces.current()).unwrap();
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert_eq!(triangulation.nb_nodes(), 54);
    assert_eq!(triangulation.nb_triangles(), 52);
    let seam_uv = triangulation.uv_node(29).unwrap();
    assert_eq!(seam_uv.x().to_bits(), std::f64::consts::TAU.to_bits());
    assert_eq!(seam_uv.y().to_bits(), 0.0_f64.to_bits());
    // Run 38118858849: host libm changes coordinate bits; node order remains native-exact.
    let expected = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        0x5c0c_e88c_ea86_3cc7
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        // Run 38120792912: independently captured native MSVC /fp:precise coordinates.
        0x8051_bf23_619e_245b
    } else {
        0x72a7_2694_b373_88ef
    };
    assert_eq!(cylinder_node_hash(&triangulation), expected);
}
