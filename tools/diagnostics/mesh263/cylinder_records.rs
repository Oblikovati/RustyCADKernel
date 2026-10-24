//! Capture every cylinder mesh field without supplying expected results.
use martensite_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Shape},
};
use martensite_tkernel::message::ProgressRange;
use martensite_tkg3d::top_abs::ShapeEnum;
use martensite_tkmath::poly::{Triangulation, mesh_purpose};
use martensite_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use martensite_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use martensite_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn main() {
    let shape = meshed_cylinder();
    let mut faces = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    for index in 0..3 {
        let face = topods::face(faces.current()).unwrap();
        let mesh = BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0;
        print_face(index, &mesh);
        faces.next().unwrap();
    }
    assert!(!faces.more());
}

fn meshed_cylinder() -> Shape {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 10.0);
    let shape = maker.shape().unwrap().clone();
    BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 0.5,
            ..Default::default()
        },
        &ProgressRange::default(),
    )
    .unwrap();
    shape
}

fn print_face(index: usize, mesh: &Triangulation) {
    println!("face {index} {} {}", mesh.nb_nodes(), mesh.nb_triangles());
    let nodes = print_nodes(mesh);
    let triangles = print_triangles(mesh);
    println!("hash {index} {nodes:016x} {triangles:016x}");
}

fn print_nodes(mesh: &Triangulation) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for index in 1..=mesh.nb_nodes() {
        let point = mesh.node(index).unwrap();
        let uv = mesh.uv_node(index).unwrap();
        for value in [point.x(), point.y(), point.z(), uv.x(), uv.y()] {
            let word = value.to_bits();
            print!("{word:016x} ");
            hash = hash_word(hash, word);
        }
        println!();
    }
    hash
}

fn print_triangles(mesh: &Triangulation) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for index in 1..=mesh.nb_triangles() {
        let (a, b, c) = mesh.triangle(index).unwrap().get();
        println!("t {a} {b} {c}");
        for node in [a, b, c] {
            hash = hash_word(hash, node as u64);
        }
    }
    hash
}

fn hash_word(hash: u64, word: u64) -> u64 {
    (hash ^ word).wrapping_mul(0x100_0000_01b3)
}
