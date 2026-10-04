//! Native map-overload and `RefineFaceInfoOn` behavior.
use rustycad_tkbo::bop_ds::{CommonBlock, DataStructure, InterfEF, InterfVF, common_block_handle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn face_info_map_overloads_and_refine_match_native() {
    let mut fixture = face_map_fixture();
    fixture.ds.update_face_info_on_map(&fixture.faces).unwrap();
    assert_native_row("on", fixture.on_row());
    assert_eq!(
        fixture
            .ds
            .face_info(fixture.face)
            .unwrap()
            .vertices_in()
            .extent(),
        0
    );
    fixture.ds.update_face_info_in_map(&fixture.faces).unwrap();
    assert_native_row("in", fixture.in_row());
    assert_in_selection(&fixture);
    fixture.ds.refine_face_info_on().unwrap();
    assert_native_row("refined", fixture.refined_row());
    assert_refined_blocks(&fixture);
}

fn assert_in_selection(fixture: &FaceMapFixture) {
    let info = fixture.ds.face_info(fixture.face).unwrap();
    assert!(info.vertices_in().contains(fixture.first_vertex));
    assert!(info.vertices_in().contains(fixture.new_vertex));
    assert!(!info.vertices_in().contains(fixture.ignored_vertex));
    assert_eq!(info.pave_blocks_in().extent(), 1);
}

fn assert_refined_blocks(fixture: &FaceMapFixture) {
    let info = fixture.ds.face_info(fixture.face).unwrap();
    assert_eq!(info.pave_blocks_on().extent(), 1);
    assert_eq!(info.vertices_on().extent(), 4);
    assert_eq!(info.vertices_in().extent(), 2);
}

struct FaceMapFixture {
    ds: DataStructure,
    faces: Map<i32>,
    face: i32,
    first_vertex: i32,
    ignored_vertex: i32,
    new_vertex: i32,
}

impl FaceMapFixture {
    fn on_row(&self) -> String {
        let info = self.ds.face_info(self.face).unwrap();
        format!(
            "on {} {} {}",
            info.pave_blocks_on().extent(),
            info.vertices_on().extent(),
            info.vertices_in().extent()
        )
    }

    fn in_row(&self) -> String {
        let info = self.ds.face_info(self.face).unwrap();
        format!(
            "in {} {} {} {} {}",
            info.pave_blocks_in().extent(),
            info.vertices_in().extent(),
            info.vertices_in().contains(self.first_vertex) as u8,
            info.vertices_in().contains(self.new_vertex) as u8,
            info.vertices_in().contains(self.ignored_vertex) as u8
        )
    }

    fn refined_row(&self) -> String {
        let info = self.ds.face_info(self.face).unwrap();
        format!(
            "refined {} {}",
            info.pave_blocks_on().extent(),
            info.vertices_on().extent()
        )
    }
}

fn face_map_fixture() -> FaceMapFixture {
    let shape = box_shape();
    let mut ds = initialized_ds(&shape);
    let (face, edge_indices) = first_face_and_edges(&ds);
    let vertex_indices = source_vertices(&ds);
    initialize_face_edge_blocks(&mut ds, &edge_indices);
    retain_one_result_edge(&mut ds, edge_indices[0]);
    add_face_interferences(
        &mut ds,
        face,
        edge_indices[0],
        vertex_indices[0],
        vertex_indices[1],
        vertex_indices[2],
    );
    let mut faces = Map::default();
    faces.add(face);
    FaceMapFixture {
        ds,
        faces,
        face,
        first_vertex: vertex_indices[0],
        ignored_vertex: vertex_indices[1],
        new_vertex: vertex_indices[2],
    }
}

fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::at_point(Point3::default(), 1.0, 2.0, 3.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn initialized_ds(shape: &Shape) -> DataStructure {
    let mut ds = DataStructure::default();
    ds.set_arguments(std::slice::from_ref(shape));
    ds.init(1.0e-7).unwrap();
    ds
}

fn first_face_and_edges(ds: &DataStructure) -> (i32, Vec<i32>) {
    let face = (0..ds.nb_source_shapes())
        .find(|index| ds.shape_info(*index).unwrap().shape_type() == ShapeEnum::Face)
        .unwrap();
    let edges = ds
        .shape_info(face)
        .unwrap()
        .sub_shapes()
        .iter()
        .copied()
        .filter(|index| ds.shape_info(*index).unwrap().shape_type() == ShapeEnum::Edge)
        .collect();
    (face, edges)
}

fn source_vertices(ds: &DataStructure) -> Vec<i32> {
    (0..ds.nb_source_shapes())
        .filter(|index| ds.shape_info(*index).unwrap().shape_type() == ShapeEnum::Vertex)
        .collect()
}

fn initialize_face_edge_blocks(ds: &mut DataStructure, edges: &[i32]) {
    for edge in edges {
        ds.change_pave_blocks(*edge).unwrap();
    }
}

fn retain_one_result_edge(ds: &mut DataStructure, edge: i32) {
    let block = ds.change_pave_blocks(edge).unwrap()[0].clone();
    block.lock().unwrap().set_edge(edge);
}

fn add_face_interferences(
    ds: &mut DataStructure,
    face: i32,
    edge: i32,
    first_vertex: i32,
    ignored_vertex: i32,
    new_vertex: i32,
) {
    append_vf(ds, face, ignored_vertex);
    append_vf(ds, first_vertex, face);
    append_ef(ds, face, edge, Some(ignored_vertex));
    append_ef(ds, edge, face, Some(new_vertex));
    append_ef(ds, edge, face, None);
    append_common_block(ds, face, edge);
}

fn append_vf(ds: &mut DataStructure, first: i32, second: i32) {
    let mut record = InterfVF::default();
    record.set_indices(first, second);
    ds.interf_vf().append(record);
}

fn append_ef(ds: &mut DataStructure, first: i32, second: i32, new_index: Option<i32>) {
    let mut record = InterfEF::default();
    record.set_indices(first, second);
    if let Some(index) = new_index {
        record.set_index_new(index);
    }
    ds.interf_ef().append(record);
}

fn append_common_block(ds: &mut DataStructure, face: i32, edge: i32) {
    let block = ds.pave_blocks(edge).unwrap()[0].clone();
    let mut common = CommonBlock::default();
    common.set_pave_blocks(std::slice::from_ref(&block));
    common.set_faces(&[face]);
    ds.set_common_block(&block, &common_block_handle(common));
}

fn native_row(stage: &str) -> String {
    include_str!("../../../tools/oracle/tkbo_face_info_map.expected")
        .lines()
        .find(|line| line.starts_with(stage))
        .expect("native map fixture row is missing")
        .to_owned()
}

fn assert_native_row(stage: &str, actual: String) {
    assert_eq!(actual, native_row(stage));
}
