//! Native differential coverage for `BOPDS_DS::FaceInfoIn` and its cache refresh.
use rustycad_tkbo::bop_ds::{
    CommonBlock, DataStructure, InterfEF, InterfVF, VertexIndexSet, common_block_handle,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn face_info_in_matches_native_internal_vf_ef_and_common_block_paths() {
    let mut fixture = face_info_fixture();
    assert_direct_face_info(&mut fixture);
    assert_cached_face_info_refresh(&mut fixture);
}

fn assert_direct_face_info(fixture: &mut FaceFixture) {
    let (mut blocks, mut vertices) = (Default::default(), VertexIndexSet::default());
    fixture
        .ds
        .face_info_in(fixture.face_index, &mut blocks, &mut vertices)
        .unwrap();
    assert_eq!(sorted_vertices(&vertices), fixture.expected_vertices());
    assert_native_extents("in", blocks.extent(), vertices.extent());
    let block = blocks.find_key(1).unwrap();
    assert_eq!(
        fixture
            .ds
            .real_pave_block(block)
            .unwrap()
            .lock()
            .unwrap()
            .original_edge(),
        fixture.edge_index
    );
}

fn assert_cached_face_info_refresh(fixture: &mut FaceFixture) {
    fixture.ds.change_face_info(fixture.face_index).unwrap();
    fixture.ds.update_face_info_in(fixture.face_index).unwrap();
    assert_cached_vertices(fixture, fixture.expected_vertices(), "cached");
    fixture
        .ds
        .add_shape_sd(fixture.alias2_index, fixture.vf_index);
    fixture.ds.update_face_info_in(fixture.face_index).unwrap();
    assert_cached_vertices(
        fixture,
        vec![fixture.vf_index, fixture.ef_index],
        "refreshed",
    );
}

fn assert_cached_vertices(fixture: &FaceFixture, expected: Vec<i32>, stage: &str) {
    let info = fixture.ds.face_info(fixture.face_index).unwrap();
    assert_eq!(sorted_vertices(info.vertices_in()), expected);
    assert_native_extents(
        stage,
        info.pave_blocks_in().extent(),
        info.vertices_in().extent(),
    );
}

#[test]
fn face_info_in_appends_to_caller_maps_and_update_is_noop_without_face_record() {
    let mut fixture = face_info_fixture();
    append_vertex_face(&mut fixture.ds, fixture.vf_index, fixture.face_index);
    let mut vertices = VertexIndexSet::default();
    vertices.add(fixture.internal_index);
    let mut blocks = Default::default();
    fixture
        .ds
        .face_info_in(fixture.face_index, &mut blocks, &mut vertices)
        .unwrap();
    assert_eq!(vertices.extent(), 4);
    fixture.ds.update_face_info_in(fixture.face_index).unwrap();
    assert!(!fixture.ds.has_face_info(fixture.face_index).unwrap());
    assert!(fixture.ds.face_info_pool().is_empty());
}

struct FaceFixture {
    ds: DataStructure,
    face_index: i32,
    edge_index: i32,
    internal_index: i32,
    alias2_index: i32,
    vf_index: i32,
    ef_index: i32,
}

impl FaceFixture {
    fn expected_vertices(&self) -> Vec<i32> {
        let mut values = vec![self.alias2_index, self.vf_index, self.ef_index];
        values.sort_unstable();
        values
    }
}

fn face_info_fixture() -> FaceFixture {
    let (face, internal) = face_with_internal_vertex();
    let (alias1, alias2, vf_vertex, ef_vertex) = fixture_vertices();
    let builder = BRepBuilder::default();
    let edge = probe_edge();
    let argument = argument_compound(
        &builder,
        [&face, &alias1, &alias2, &vf_vertex, &ef_vertex, &edge],
    );
    let mut ds = initialized_structure(&argument);
    let (face_index, edge_index) = (ds.index(&face), ds.index(&edge));
    let (internal_index, alias1_index, alias2_index) =
        (ds.index(&internal), ds.index(&alias1), ds.index(&alias2));
    let (vf_index, ef_index) = (ds.index(&vf_vertex), ds.index(&ef_vertex));
    ds.add_shape_sd(internal_index, alias1_index);
    ds.add_shape_sd(alias1_index, alias2_index);
    append_vertex_face(&mut ds, vf_index, face_index);
    append_edge_face_new(&mut ds, edge_index, face_index, ef_index);
    append_edge_face_common(&mut ds, edge_index, face_index);
    FaceFixture {
        ds,
        face_index,
        edge_index,
        internal_index,
        alias2_index,
        vf_index,
        ef_index,
    }
}

fn face_with_internal_vertex() -> (Shape, Shape) {
    let mut face =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2.0, 2.0, -2.0, 2.0))
            .unwrap()
            .face()
            .unwrap()
            .into_shape();
    let internal = vertex_at(0.0, 0.0, 0.0);
    BRepBuilder::default().add(&mut face, &internal).unwrap();
    (face, internal)
}

fn fixture_vertices() -> (Shape, Shape, Shape, Shape) {
    (
        vertex_at(0.0, 0.0, 0.0),
        vertex_at(0.0, 0.0, 0.0),
        vertex_at(0.5, 0.0, 0.0),
        vertex_at(-0.5, 0.0, 0.0),
    )
}

fn probe_edge() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn argument_compound(builder: &BRepBuilder, children: [&Shape; 6]) -> Shape {
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    for child in children {
        builder.add(&mut shape, child).unwrap();
    }
    shape
}

fn initialized_structure(argument: &Shape) -> DataStructure {
    let mut ds = DataStructure::default();
    ds.set_arguments(std::slice::from_ref(argument));
    ds.init(1.0e-7).unwrap();
    ds
}

fn vertex_at(x: f64, y: f64, z: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, y, z), 1.0e-7)
        .unwrap();
    vertex.into_shape()
}

fn append_vertex_face(ds: &mut DataStructure, vertex: i32, face: i32) {
    let mut interference = InterfVF::default();
    interference.set_indices(vertex, face);
    ds.interf_vf().append(interference);
}

fn append_edge_face_new(ds: &mut DataStructure, edge: i32, face: i32, vertex: i32) {
    let mut interference = InterfEF::default();
    interference.set_indices(edge, face);
    interference.set_index_new(vertex);
    ds.interf_ef().append(interference);
}

fn append_edge_face_common(ds: &mut DataStructure, edge: i32, face: i32) {
    let block = ds.change_pave_blocks(edge).unwrap()[0].clone();
    let mut common = CommonBlock::default();
    common.set_pave_blocks(std::slice::from_ref(&block));
    common.set_faces(&[face]);
    ds.set_common_block(&block, &common_block_handle(common));
    let mut interference = InterfEF::default();
    interference.set_indices(edge, face);
    ds.interf_ef().append(interference);
}

fn sorted_vertices(vertices: &VertexIndexSet) -> Vec<i32> {
    let mut values: Vec<_> = vertices.iter().copied().collect();
    values.sort_unstable();
    values
}

fn assert_native_extents(stage: &str, blocks: usize, vertices: usize) {
    let row = include_str!("../../../tools/oracle/tkbo_face_info_in.expected")
        .lines()
        .find(|line| line.starts_with(stage))
        .expect("native FaceInfoIn output must contain requested stage");
    let fields: Vec<_> = row.split_whitespace().collect();
    let count_offset = if stage == "in" { 4 } else { 1 };
    let native_blocks = fields[count_offset].parse::<usize>().unwrap();
    let native_vertices = fields[count_offset + 1].parse::<usize>().unwrap();
    assert_eq!((blocks, vertices), (native_blocks, native_vertices));
}
