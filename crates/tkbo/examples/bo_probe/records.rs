use super::{flag, integer, real};
use rustycad_tkbo::bop_ds::{
    BopdsCurve, CommonBlock, CoupleOfPaveBlocks, FaceInfo, IndexRange, InterfEE, InterfEF,
    InterfEZ, InterfFF, InterfFZ, InterfVE, InterfVF, InterfVV, InterfVZ, InterfZZ, Interference,
    Pair, Pave, PaveBlock, Point, ShapeInfo, common_block_handle, pave_block_handle,
};
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point2, Point3};
use std::sync::Arc;
fn index_state(value: &IndexRange, query: i32) {
    integer(value.first());
    integer(value.last());
    let (first, last) = value.indices();
    integer(first);
    integer(last);
    flag(value.contains(query));
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn index_case(v: &[f64]) {
    index_state(&IndexRange::default(), v[4] as i32);
    let mut value = IndexRange::new(v[0] as i32, v[1] as i32);
    index_state(&value, v[4] as i32);
    value.set_first(v[2] as i32);
    value.set_last(v[3] as i32);
    index_state(&value, v[4] as i32);
    value.set_indices(v[1] as i32, v[0] as i32);
    index_state(&value, v[4] as i32);
}
fn pave_state(value: &Pave) {
    integer(value.index());
    real(value.parameter());
    let (index, parameter) = value.contents();
    integer(index);
    real(parameter);
    print!("{} ", value.hash_code());
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn pave_case(v: &[f64]) {
    pave_state(&Pave::default());
    let mut a = Pave::new(v[0] as i32, v[1]);
    let b = Pave::new(v[2] as i32, v[3]);
    pave_state(&a);
    pave_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_index(v[2] as i32);
    a.set_parameter(v[3]);
    pave_state(&a);
    flag(a.is_equal(&b));
}
fn pair_state(value: &Pair) {
    let (a, b) = value.indices();
    integer(a);
    integer(b);
    print!("{} ", value.hash_code());
}
pub fn pair_case(v: &[f64]) {
    pair_state(&Pair::default());
    let mut a = Pair::new(v[0] as i32, v[1] as i32);
    let b = Pair::new(v[2] as i32, v[3] as i32);
    pair_state(&a);
    pair_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_indices(v[3] as i32, v[2] as i32);
    pair_state(&a);
    flag(a == b);
}
fn point_state(value: &Point) {
    for coordinate in [
        value.pnt().x(),
        value.pnt().y(),
        value.pnt().z(),
        value.pnt2d1().x(),
        value.pnt2d1().y(),
        value.pnt2d2().x(),
        value.pnt2d2().y(),
    ] {
        real(coordinate);
    }
    integer(value.index());
}
pub fn point_case(v: &[f64]) {
    let mut value = Point::default();
    point_state(&value);
    value.set_pnt(Point3::new(v[0], v[1], v[2]));
    value.set_pnt2d1(Point2::new(v[3], v[4]));
    value.set_pnt2d2(Point2::new(v[5], v[6]));
    value.set_index(v[7] as i32);
    point_state(&value);
}

/// The `BOPDS_PaveBlock` endpoint, extra-pave, update and shrunk-box oracle request.
pub fn pave_block_case(v: &[f64]) {
    let mut block = PaveBlock::default();
    let mut edge = 99;
    integer(block.edge());
    integer(block.original_edge());
    flag(block.has_edge());
    flag(block.has_edge_into(&mut edge));
    integer(edge);
    block.set_edge(v[0] as i32);
    block.set_original_edge(v[1] as i32);
    block.set_pave1(Pave::new(v[2] as i32, v[3]));
    block.set_pave2(Pave::new(v[4] as i32, v[5]));
    flag(block.is_split_edge());
    let (first, last) = block.range();
    real(first);
    real(last);
    let (i1, i2) = block.indices();
    integer(i1);
    integer(i2);
    append_extra_paves(&mut block, v);
    integer(block.ext_paves().len() as i32);
    let mut found = -7;
    flag(block.contains_parameter(v[12], v[13], &mut found));
    integer(found);
    flag(block.is_to_update());
    let mut splits = Vec::new();
    block.update(&mut splits, v[14] != 0.0);
    write_splits(&splits);
    write_shrunk_data(&mut block, v);
}

/// The `BOPDS_CommonBlock` native ordering, identity, face-list and dump oracle request.
pub fn common_block_case(v: &[f64]) {
    let mut group = CommonBlock::default();
    real(group.tolerance());
    flag(group.pave_block1().is_err());
    flag(group.edge().is_err());
    integer(group.pave_blocks().len() as i32);
    integer(group.faces().len() as i32);
    let blocks = [
        common_pave_block(v[0] as i32, v[1] as i32),
        common_pave_block(v[2] as i32, v[3] as i32),
        common_pave_block(v[4] as i32, v[5] as i32),
    ];
    for block in &blocks {
        group.add_pave_block(std::sync::Arc::clone(block));
    }
    integer(group.pave_blocks().len() as i32);
    for block in group.pave_blocks() {
        integer(block.lock().unwrap().original_edge());
    }
    flag(group.is_pave_block_on_edge(v[8] as i32));
    flag(group.contains_pave_block(&blocks[2]));
    flag(group.is_pave_block_on_face(v[9] as i32));
    group.set_edge(v[10] as i32);
    for block in group.pave_blocks() {
        integer(block.lock().unwrap().edge());
    }
    integer(group.edge().unwrap());
    group.set_real_pave_block(&blocks[2]);
    flag(std::sync::Arc::ptr_eq(
        group.pave_block1().unwrap(),
        &blocks[2],
    ));
    for block in group.pave_blocks() {
        integer(block.lock().unwrap().original_edge());
    }
    group.set_faces(&[v[6] as i32, v[7] as i32]);
    let mut appended = vec![v[11] as i32, v[12] as i32];
    group.append_faces(&mut appended);
    integer(appended.len() as i32);
    integer(group.faces().len() as i32);
    for face in group.faces() {
        integer(*face);
    }
    flag(group.contains_face(v[11] as i32));
    flag(group.is_pave_block_on_face(v[9] as i32));
    let selected = group.pave_block_on_edge(v[8] as i32).unwrap();
    flag(std::sync::Arc::ptr_eq(&selected, &blocks[2]));
    selected.lock().unwrap().set_edge(v[13] as i32);
    integer(blocks[2].lock().unwrap().edge());
    group.set_tolerance(v[14]);
    real(group.tolerance());
    let mut dump = String::new();
    group.dump(&mut dump).unwrap();
    print!("{}", dump.replace('\n', "\\n"));
}

fn common_pave_block(original_edge: i32, edge: i32) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_original_edge(original_edge);
    block.set_edge(edge);
    pave_block_handle(block)
}

/// The `BOPDS_CoupleOfPaveBlocks` nullable-handle and scalar oracle request.
pub fn couple_of_pave_blocks_case(v: &[f64]) {
    let empty = CoupleOfPaveBlocks::default();
    integer(empty.index());
    integer(empty.interference_index());
    real(empty.tolerance());
    flag(empty.pave_block1().is_none());
    flag(empty.pave_block2().is_none());
    let first = common_pave_block(v[0] as i32, -1);
    let second = common_pave_block(v[1] as i32, -1);
    let mut value = CoupleOfPaveBlocks::with_pave_blocks(
        Some(std::sync::Arc::clone(&first)),
        Some(std::sync::Arc::clone(&second)),
    );
    flag(std::sync::Arc::ptr_eq(value.pave_block1().unwrap(), &first));
    flag(std::sync::Arc::ptr_eq(
        value.pave_block2().unwrap(),
        &second,
    ));
    value.set_index(v[2] as i32);
    value.set_interference_index(v[3] as i32);
    value.set_tolerance(v[4]);
    integer(value.index());
    integer(value.interference_index());
    real(value.tolerance());
    let (mut output_first, mut output_second) = (None, None);
    value.pave_blocks_into(&mut output_first, &mut output_second);
    flag(std::sync::Arc::ptr_eq(
        output_first.as_ref().unwrap(),
        &first,
    ));
    flag(std::sync::Arc::ptr_eq(
        output_second.as_ref().unwrap(),
        &second,
    ));
    value.set_pave_block1(Some(std::sync::Arc::clone(&second)));
    value.set_pave_block2(None);
    flag(std::sync::Arc::ptr_eq(
        value.pave_block1().unwrap(),
        &second,
    ));
    flag(value.pave_block2().is_none());
    value.set_pave_blocks(Some(std::sync::Arc::clone(&first)), Some(second.clone()));
    flag(std::sync::Arc::ptr_eq(value.pave_block1().unwrap(), &first));
    flag(std::sync::Arc::ptr_eq(
        value.pave_block2().unwrap(),
        &second,
    ));
}

/// The `BOPDS_ShapeInfo` metadata and Dump oracle request.
pub fn shape_info_case(v: &[f64]) {
    let mut info = ShapeInfo::default();
    let mut flag_value = 99;
    integer(info.shape_type() as i32);
    flag(info.shape().is_null());
    flag(info.box_bounds().is_void());
    integer(info.sub_shapes().len() as i32);
    integer(info.reference());
    flag(info.has_reference());
    flag(info.has_brep());
    flag(info.is_interfering());
    flag(info.has_flag());
    flag(info.has_flag_into(&mut flag_value));
    integer(flag_value);
    info.set_shape(&rustycad_tkbrep::topods::Shape::default());
    info.set_shape_type(ShapeEnum::ALL[v[0] as usize]);
    info.set_box(&BoundingBox3::new(
        Point3::new(v[3], v[4], v[5]),
        Point3::new(v[6], v[7], v[8]),
    ));
    info.set_reference(v[1] as i32);
    info.set_flag(v[2] as i32);
    info.change_sub_shapes()
        .extend([v[9] as i32, v[10] as i32, v[11] as i32]);
    info.change_box().set_gap(v[12]);
    integer(info.shape_type() as i32);
    flag(info.shape().is_null());
    flag(info.box_bounds().is_void());
    integer(info.sub_shapes().len() as i32);
    for index in info.sub_shapes() {
        integer(*index);
    }
    flag(info.has_sub_shape(v[9] as i32));
    flag(info.has_sub_shape(-999));
    integer(info.reference());
    flag(info.has_reference());
    flag(info.has_brep());
    flag(info.is_interfering());
    flag(info.has_flag());
    flag(info.has_flag_into(&mut flag_value));
    integer(flag_value);
    for bound in info.box_bounds().get().unwrap() {
        real(bound);
    }
    let mut dump = String::new();
    info.dump(&mut dump).unwrap();
    print!("{dump}");
}

/// The `BOPDS_FaceInfo` map insertion and partial-clear oracle request.
pub fn face_info_case(_: &[f64]) {
    let mut info = FaceInfo::default();
    integer(info.index());
    integer(info.pave_blocks_in().extent() as i32);
    integer(info.vertices_in().extent() as i32);
    integer(info.pave_blocks_on().extent() as i32);
    integer(info.vertices_on().extent() as i32);
    integer(info.pave_blocks_section().extent() as i32);
    integer(info.vertices_section().extent() as i32);
    let a = common_pave_block(0, -1);
    let b = common_pave_block(0, -1);
    let c = common_pave_block(0, -1);
    integer(info.change_pave_blocks_in().add(Arc::clone(&a)) as i32);
    integer(info.change_pave_blocks_in().add(Arc::clone(&b)) as i32);
    integer(info.change_pave_blocks_in().add(Arc::clone(&a)) as i32);
    flag(info.change_vertices_in().add(4));
    flag(info.change_vertices_in().add(9));
    flag(info.change_vertices_in().add(4));
    integer(info.change_pave_blocks_on().add(Arc::clone(&b)) as i32);
    flag(info.change_vertices_on().add(9));
    integer(info.change_pave_blocks_section().add(Arc::clone(&c)) as i32);
    integer(info.change_pave_blocks_section().add(Arc::clone(&a)) as i32);
    flag(info.change_vertices_section().add(6));
    flag(info.change_vertices_section().add(7));
    info.set_index(12);
    write_face_info_counts(&info);
    flag(info.pave_blocks_in().contains(&a));
    flag(info.pave_blocks_in().contains(&c));
    flag(info.vertices_in().contains(9));
    info.clear();
    write_face_info_counts(&info);
    flag(info.pave_blocks_section().contains(&c));
    flag(info.pave_blocks_section().contains(&a));
    flag(info.vertices_section().contains(7));
}

/// The pre-Init shape and argument storage of `BOPDS_DS`.
pub fn data_structure_case(_: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    let (first, second) = data_structure_shapes();
    let mut structure = DataStructure::default();
    structure.set_arguments(&[first.clone(), second.clone()]);
    write_initial_ds_state(&structure);
    write_appended_ds_state(&mut structure, &first, &second);
    write_appended_metadata(&mut structure, &second);
    structure.clear();
    write_cleared_ds_state(&structure, &first);
}

pub fn data_structure_common_block_case(_: &[f64]) {
    let first = block_with_source_edge(8);
    let real = block_with_source_edge(3);
    let unrelated = block_with_source_edge(3);
    let mut common = CommonBlock::default();
    common.set_pave_blocks(&[first.clone(), real.clone()]);
    let common = common_block_handle(common);
    let mut structure = rustycad_tkbo::bop_ds::DataStructure::default();
    flag(structure.is_common_block(&first));
    flag(structure.common_block(&first).is_none());
    flag(Arc::ptr_eq(
        &structure.real_pave_block(&first).unwrap(),
        &first,
    ));
    flag(structure.is_common_block_on_edge(&first));
    structure.set_common_block(&first, &common);
    structure.set_common_block(&real, &common);
    flag(structure.is_common_block(&first));
    flag(structure.is_common_block(&real));
    flag(structure.is_common_block(&unrelated));
    flag(Arc::ptr_eq(
        &structure.common_block(&first).unwrap(),
        &common,
    ));
    flag(Arc::ptr_eq(
        &structure.real_pave_block(&first).unwrap(),
        &real,
    ));
    flag(structure.is_common_block_on_edge(&first));
    let mut replacement_group = CommonBlock::default();
    replacement_group.set_pave_blocks(std::slice::from_ref(&first));
    let replacement = common_block_handle(replacement_group);
    structure.set_common_block(&first, &replacement);
    flag(Arc::ptr_eq(
        &structure.common_block(&first).unwrap(),
        &replacement,
    ));
    flag(Arc::ptr_eq(
        &structure.real_pave_block(&first).unwrap(),
        &first,
    ));
    flag(structure.is_common_block_on_edge(&first));
    structure.clear();
    flag(structure.is_common_block(&first));
}

pub fn bopds_curve_case(_: &[f64]) {
    let mut value = BopdsCurve::default();
    real(value.tolerance());
    real(value.tangential_tolerance());
    flag(value.box_bounds().is_void());
    integer(value.pave_blocks().len() as i32);
    integer(value.techno_vertices().len() as i32);
    flag(value.has_edge());
    let error = value.change_pave_block1().unwrap_err();
    flag(error.exception_type() == "Standard_NoSuchObject");
    value.init_pave_block1();
    value.init_pave_block1();
    integer(value.pave_blocks().len() as i32);
    value
        .change_pave_block1()
        .unwrap()
        .lock()
        .unwrap()
        .set_edge(12);
    flag(value.has_edge());

    let mut curve = Curve::default();
    curve.set_tangential_tolerance(4.5e-5);
    value.set_curve(&curve);
    real(value.tangential_tolerance());
    let mut bounds = BoundingBox3::new(Point3::new(-1., -2., -3.), Point3::new(4., 5., 6.));
    bounds.set_gap(0.125);
    value.set_box(&bounds);
    bounds.set_gap(9.);
    real(value.box_bounds().gap());
    value.change_box().set_gap(0.25);
    real(value.box_bounds().gap());
    for coordinate in value.box_bounds().get().unwrap() {
        real(coordinate);
    }

    let first = pave_block_handle(PaveBlock::default());
    let edge = block_with_edge(7);
    value.set_pave_blocks(&[first.clone(), edge.clone(), edge.clone()]);
    integer(value.pave_blocks().len() as i32);
    flag(Arc::ptr_eq(&value.pave_blocks()[0], &first));
    flag(Arc::ptr_eq(&value.pave_blocks()[1], &edge));
    flag(Arc::ptr_eq(&value.pave_blocks()[2], &edge));
    flag(value.has_edge());
    value.change_pave_blocks().remove(0);
    integer(value.pave_blocks().len() as i32);
    value.change_techno_vertices().extend([8, 3, 8]);
    integer(value.techno_vertices().len() as i32);
    for index in value.techno_vertices() {
        integer(*index);
    }
    value.set_tolerance(-0.);
    real(value.tolerance());
}

pub fn bopds_interf_case(_: &[f64]) {
    let mut vv = InterfVV::default();
    integer(vv.index1());
    integer(vv.index2());
    integer(vv.index_new());
    flag(vv.has_index_new());
    let mut new_index = 77;
    flag(vv.has_index_new_into(&mut new_index));
    integer(new_index);
    optional_index(vv.get_index_new());
    vv.set_indices(3, 8);
    let (first, second) = vv.indices();
    integer(first);
    integer(second);
    vv.set_index1(-4);
    vv.set_index2(0);
    integer(vv.index1());
    integer(vv.index2());
    integer(vv.opposite_index(-4));
    integer(vv.opposite_index(0));
    integer(vv.opposite_index(19));
    flag(vv.contains(-4));
    flag(vv.contains(8));
    vv.set_index_new(-2);
    new_index = 77;
    integer(vv.index_new());
    flag(vv.has_index_new_into(&mut new_index));
    integer(new_index);
    flag(vv.has_index_new());
    optional_index(vv.get_index_new());
    vv.set_index_new(-1);
    flag(vv.has_index_new());
    optional_index(vv.get_index_new());

    let mut ve = InterfVE::default();
    real(ve.parameter());
    ve.set_parameter(-0.);
    real(ve.parameter());
    ve.set_indices(6, 9);
    integer(ve.opposite_index(9));
    let mut vf = InterfVF::default();
    let (u, v) = vf.uv();
    real(u);
    real(v);
    vf.set_uv(-1.25, 2.5);
    let (u, v) = vf.uv();
    real(u);
    real(v);
    vf.set_indices(11, 13);
    flag(vf.contains(13));
}

pub fn bopds_solid_interferences_case(_: &[f64]) {
    let mut vz = InterfVZ::default();
    solid_interference_state(&mut vz);
    let mut ez = InterfEZ::default();
    solid_interference_state(&mut ez);
    let mut fz = InterfFZ::default();
    solid_interference_state(&mut fz);
    let mut zz = InterfZZ::default();
    solid_interference_state(&mut zz);
}

fn solid_interference_state(value: &mut Interference) {
    integer(value.index1());
    integer(value.index2());
    integer(value.index_new());
    value.set_indices(4, 9);
    value.set_index_new(13);
    let (first, second) = value.indices();
    integer(first);
    integer(second);
    integer(value.opposite_index(4));
    integer(value.opposite_index(9));
    integer(value.opposite_index(27));
    flag(value.contains(4));
    flag(value.contains(9));
    flag(value.contains(27));
    integer(value.index_new());
    flag(value.has_index_new());
    let mut new_index = 77;
    flag(value.has_index_new_into(&mut new_index));
    integer(new_index);
    value.set_index_new(-1);
    flag(value.has_index_new());
}

pub fn bopds_interf_common_case(_: &[f64]) {
    let first = common_part(ShapeEnum::Edge, (1.0, 2.0), &[(3.0, 4.0)], true, 1.0);
    let mut edge_edge = InterfEE::default();
    integer(edge_edge.index1());
    integer(edge_edge.index2());
    write_common_part(edge_edge.common_part());
    edge_edge.set_indices(7, 9);
    edge_edge.set_common_part(&first);
    integer(edge_edge.index1());
    integer(edge_edge.index2());
    write_common_part(edge_edge.common_part());

    let second = common_part(ShapeEnum::Vertex, (-1.0, -2.0), &[(7.0, 8.0)], false, -1.0);
    edge_edge.set_common_part(&second);
    write_common_part(edge_edge.common_part());

    let mut edge_face = InterfEF::default();
    edge_face.set_indices(11, 19);
    edge_face.set_common_part(&first);
    integer(edge_face.index1());
    integer(edge_face.index2());
    write_common_part(edge_face.common_part());
}

pub fn bopds_interf_ff_case(_: &[f64]) {
    let mut value = InterfFF::default();
    let (first_index, second_index) = value.indices();
    integer(first_index);
    integer(second_index);
    flag(value.tangent_faces());
    write_dynamic_bounds(
        value.curves().lower(),
        value.curves().upper(),
        value.curves().length(),
    );
    write_dynamic_bounds(
        value.points().lower(),
        value.points().upper(),
        value.points().length(),
    );
    value.init(3, 2);
    integer(value.curves().length());
    integer(value.points().length());
    value.set_indices(7, 11);
    value.set_tangent_faces(true);
    append_probe_curve(&mut value, 0, 0.125);
    append_probe_curve(&mut value, 4, 0.875);
    append_probe_point(&mut value, 0, [1., 2., 3.], [4., 5.], [6., 7.], 10);
    append_probe_point(&mut value, 2, [-1., -2., -3.], [-4., -5.], [-6., -7.], 12);
    value.init(1, 1);
    write_interf_ff_results(&value);
}

fn append_probe_curve(record: &mut InterfFF, index: i32, tolerance: f64) {
    let mut curve = BopdsCurve::default();
    curve.set_tolerance(tolerance);
    record.change_curves().set_value(index, curve).unwrap();
}

fn append_probe_point(
    record: &mut InterfFF,
    index: i32,
    point: [f64; 3],
    first_uv: [f64; 2],
    second_uv: [f64; 2],
    vertex: i32,
) {
    let mut value = Point::default();
    value.set_pnt(Point3::new(point[0], point[1], point[2]));
    value.set_pnt2d1(Point2::new(first_uv[0], first_uv[1]));
    value.set_pnt2d2(Point2::new(second_uv[0], second_uv[1]));
    value.set_index(vertex);
    record.change_points().set_value(index, value).unwrap();
}

fn write_dynamic_bounds(lower: i32, upper: i32, length: i32) {
    integer(length);
    integer(lower);
    integer(upper);
}

fn write_interf_ff_results(value: &InterfFF) {
    let (first_index, second_index) = value.indices();
    integer(first_index);
    integer(second_index);
    flag(value.tangent_faces());
    write_dynamic_bounds(
        value.curves().lower(),
        value.curves().upper(),
        value.curves().length(),
    );
    for index in [0, 1, 4] {
        real(value.curves().value(index).unwrap().tolerance());
    }
    write_dynamic_bounds(
        value.points().lower(),
        value.points().upper(),
        value.points().length(),
    );
    write_probe_point(value.points().value(0).unwrap());
    write_probe_point(value.points().value(2).unwrap());
}

fn write_probe_point(value: &Point) {
    for coordinate in [
        value.pnt().x(),
        value.pnt().y(),
        value.pnt().z(),
        value.pnt2d1().x(),
        value.pnt2d1().y(),
        value.pnt2d2().x(),
        value.pnt2d2().y(),
    ] {
        real(coordinate);
    }
    integer(value.index());
}

fn common_part(
    kind: ShapeEnum,
    bounds: (f64, f64),
    other_bounds: &[(f64, f64)],
    all_null: bool,
    point_offset: f64,
) -> rustycad_tkbo::int_tools::CommonPrt {
    use rustycad_tkmath::gp::Point3;
    let mut part = rustycad_tkbo::int_tools::CommonPrt::default();
    part.set_type(kind);
    part.set_range1_bounds(bounds.0, bounds.1);
    part.set_vertex_parameter1(1.25);
    part.set_vertex_parameter2(-2.5);
    part.set_all_null_flag(all_null);
    for (first, last) in other_bounds {
        part.append_range2_bounds(*first, *last);
    }
    part.set_bounding_points(
        Point3::new(point_offset, 2.0 * point_offset, 3.0 * point_offset),
        Point3::new(4.0 * point_offset, 5.0 * point_offset, 6.0 * point_offset),
    );
    part
}

fn write_common_part(part: &rustycad_tkbo::int_tools::CommonPrt) {
    integer(part.shape_type() as i32);
    let (first, last) = part.range1_bounds();
    real(first);
    real(last);
    real(part.vertex_parameter1());
    real(part.vertex_parameter2());
    flag(part.all_null_flag());
    let (first, second) = part.bounding_points();
    for coordinate in first.coordinates().iter().chain(second.coordinates()) {
        real(*coordinate);
    }
    integer(part.ranges2().len() as i32);
    for range in part.ranges2() {
        let (first, last) = range.range();
        real(first);
        real(last);
    }
}

fn optional_index(index: Option<i32>) {
    flag(index.is_some());
    if let Some(index) = index {
        integer(index);
    }
}

fn block_with_edge(edge: i32) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_edge(edge);
    pave_block_handle(block)
}

fn block_with_source_edge(edge: i32) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let mut block = PaveBlock::default();
    block.set_original_edge(edge);
    pave_block_handle(block)
}

/// The recursive `BOPDS_DS::Init` order and vertex-bound preparation slice.
pub fn data_structure_init_case(v: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    let (root, first, second, third) = data_structure_init_shapes();
    let mut structure = DataStructure::default();
    structure.set_arguments(&[root.clone(), first.clone(), third.clone()]);
    structure.init(v[0]).expect("valid BRep source arguments");
    write_initialized_ranges(&structure);
    write_initialized_indices(&structure, &root, &first, &second, &third);
    write_initialized_records(&structure);
}

/// Writes edge initialization, shared-vertex links and endpoint pave blocks.
pub fn data_structure_edges_case(v: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    use rustycad_tkbrep::brep::BRepBuilder;
    use rustycad_tkbrep::topods::{Shape, Vertex, Wire};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let builder = BRepBuilder::default();
    let first = initialized_vertex(&builder, [-1.0, 0.0, 0.0], 1e-7);
    let shared = initialized_vertex(&builder, [0.0, 0.0, 0.0], 2e-7);
    let last = initialized_vertex(&builder, [1.0, 1.0, 0.0], 3e-7);
    let first_vertex = Vertex::unchecked(first.clone());
    let shared_vertex = Vertex::unchecked(shared.clone());
    let last_vertex = Vertex::unchecked(last.clone());
    let first_edge = BRepBuilderAPIMakeEdge::from_vertices(&first_vertex, &shared_vertex)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    let second_edge = BRepBuilderAPIMakeEdge::from_vertices(&shared_vertex, &last_vertex)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut root: Shape = wire.into_shape();
    builder.add(&mut root, &first_edge).unwrap();
    builder.add(&mut root, &second_edge).unwrap();
    let mut structure = DataStructure::default();
    structure.set_arguments(&[root]);
    structure.init(v[0]).unwrap();
    write_edge_source_state(
        &structure,
        &first,
        &shared,
        &last,
        &first_edge,
        &second_edge,
    );
    let shared_index = structure.index(&shared);
    structure.init_pave_blocks_for_vertex(shared_index).unwrap();
    write_edge_pave_state(
        &structure,
        &[structure.index(&first_edge), structure.index(&second_edge)],
    );
}

/// Writes native box face/edge flattening and bounds after a single-solid initialization.
pub fn data_structure_box_case(v: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    let solid = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape();
    let mut structure = DataStructure::default();
    structure.set_arguments(std::slice::from_ref(&solid));
    structure.init(v[0]).unwrap();
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    integer(structure.index(&solid));
    for index in 0..structure.nb_shapes() {
        let info = structure.shape_info(index).unwrap();
        integer(info.shape_type() as i32);
        integer(info.sub_shapes().len() as i32);
        for child in info.sub_shapes() {
            integer(*child);
        }
        flag(info.box_bounds().is_void());
        if !info.box_bounds().is_void() {
            real(info.box_bounds().gap());
            for coordinate in info.box_bounds().get().unwrap() {
                real(coordinate);
            }
        }
    }
}

/// Writes finite and infinite line-edge endpoint preparation and pave construction.
pub fn data_structure_infinite_case(v: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    use rustycad_tkbrep::topods::{Edge, Shape};
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::Line3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let infinity = rustycad_tkernel::precision::infinite();
    let (first, last) = line_edge_bounds(v[1] as i32, infinity);
    let edge: Shape = BRepBuilderAPIMakeEdge::from_lin_params(Line3::default(), first, last)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    let edge = if v[1] as i32 == 4 {
        Edge::unchecked(edge.oriented(Orientation::Internal)).into_shape()
    } else {
        edge
    };
    let mut structure = DataStructure::default();
    structure.set_arguments(std::slice::from_ref(&edge));
    structure.init(v[0]).unwrap();
    write_infinite_edge_shapes(&structure, &edge);
    write_infinite_edge_paves(&mut structure, &edge);
}

/// Writes closed-edge pcurve selection and duplicate endpoint paves.
pub fn data_structure_seam_case(v: &[f64]) {
    use rustycad_tkbo::bop_ds::DataStructure;
    use rustycad_tkbrep::topods::Edge;
    use std::f64::consts::TAU;

    let edge = make_seam_edge(v[1] as i32, TAU);
    let edge = if v[1] as i32 >= 2 {
        edge.reversed()
    } else {
        edge
    };
    let edge = Edge::unchecked(edge);
    let edge_shape = edge.clone().into_shape();
    let mut structure = DataStructure::default();
    structure.set_arguments(std::slice::from_ref(&edge_shape));
    structure.init(v[0]).unwrap();
    write_seam_shape_state(&structure, &edge);
    structure
        .change_pave_blocks(structure.index(&edge.clone().into_shape()))
        .unwrap();
    write_seam_pave_state(&structure, &edge);
}

fn make_seam_edge(kind: i32, period: f64) -> rustycad_tkbrep::topods::Shape {
    use rustycad_tkbrep::brep::BRepBuilder;
    use rustycad_tkbrep::topods::Vertex;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
    use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3, Circle3, Direction2, Point2, Point3};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    if kind % 2 == 0 {
        return BRepBuilderAPIMakeEdge::from_circ(Circle3::new(Axis2::default(), 1.0).unwrap())
            .unwrap()
            .edge()
            .unwrap()
            .into_shape();
    }
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::X,
    )))
    .cast();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_pcurve_vertices_params(
        &curve, &surface, &vertex, &vertex, 0.0, period,
    )
    .unwrap()
    .edge()
    .unwrap()
    .into_shape()
}

fn write_seam_shape_state(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: &rustycad_tkbrep::topods::Edge,
) {
    use rustycad_tkbrep::brep::BRepTool;
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    let edge_shape = edge.clone().into_shape();
    let edge_index = structure.index(&edge_shape);
    integer(edge_index);
    let info = structure.shape_info(edge_index).unwrap();
    integer(info.sub_shapes().len() as i32);
    for child in info.sub_shapes() {
        integer(*child);
        integer(structure.shape(*child).unwrap().orientation() as i32);
    }
    let (curve, _, _) = BRepTool::curve(edge).unwrap();
    flag(curve.is_null());
    let pcurve = BRepTool::first_curve_on_surface(edge).unwrap().pcurve;
    flag(pcurve.is_null());
}

fn write_seam_pave_state(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: &rustycad_tkbrep::topods::Edge,
) {
    integer(structure.pave_blocks_pool().len() as i32);
    let edge_index = structure.index(&edge.clone().into_shape());
    integer(structure.shape_info(edge_index).unwrap().reference());
    let blocks = structure.pave_blocks(edge_index).unwrap();
    integer(blocks.len() as i32);
    for handle in blocks {
        let block = handle.lock().unwrap();
        integer(block.original_edge());
        let (first, last) = block.indices();
        let (first_parameter, last_parameter) = block.range();
        integer(first);
        integer(last);
        real(first_parameter);
        real(last_parameter);
        integer(block.ext_paves().len() as i32);
    }
}

fn line_edge_bounds(kind: i32, infinity: f64) -> (f64, f64) {
    match kind {
        1 => (-infinity, 2.0),
        2 => (-2.0, infinity),
        3 | 4 => (-2.0, 3.0),
        _ => (-infinity, infinity),
    }
}

fn write_infinite_edge_shapes(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: &rustycad_tkbrep::topods::Shape,
) {
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    integer(structure.index(edge));
    for index in 0..structure.nb_shapes() {
        let info = structure.shape_info(index).unwrap();
        integer(info.shape_type() as i32);
        flag(info.has_flag());
        integer(info.flag());
        integer(info.sub_shapes().len() as i32);
        for child in info.sub_shapes() {
            integer(*child);
        }
        flag(info.box_bounds().is_void());
        if !info.box_bounds().is_void() {
            real(info.box_bounds().gap());
            for coordinate in info.box_bounds().get().unwrap() {
                real(coordinate);
            }
        }
    }
}

fn write_infinite_edge_paves(
    structure: &mut rustycad_tkbo::bop_ds::DataStructure,
    edge: &rustycad_tkbrep::topods::Shape,
) {
    let index = structure.index(edge);
    structure.change_pave_blocks(index).unwrap();
    integer(structure.pave_blocks_pool().len() as i32);
    integer(structure.shape_info(index).unwrap().reference());
    let blocks = structure.pave_blocks(index).unwrap();
    integer(blocks.len() as i32);
    for block in blocks {
        let block = block.lock().unwrap();
        integer(block.original_edge());
        let (first, last) = block.indices();
        let (first_parameter, last_parameter) = block.range();
        integer(first);
        integer(last);
        real(first_parameter);
        real(last_parameter);
    }
}

fn write_edge_source_state(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    first: &rustycad_tkbrep::topods::Shape,
    shared: &rustycad_tkbrep::topods::Shape,
    last: &rustycad_tkbrep::topods::Shape,
    first_edge: &rustycad_tkbrep::topods::Shape,
    second_edge: &rustycad_tkbrep::topods::Shape,
) {
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    for shape in [first, shared, last, first_edge, second_edge] {
        integer(structure.index(shape));
    }
    for index in 0..structure.nb_shapes() {
        let info = structure.shape_info(index).unwrap();
        integer(info.shape_type() as i32);
        integer(info.sub_shapes().len() as i32);
        for child in info.sub_shapes() {
            integer(*child);
        }
        flag(info.box_bounds().is_void());
        if !info.box_bounds().is_void() {
            real(info.box_bounds().gap());
            for coordinate in info.box_bounds().get().unwrap() {
                real(coordinate);
            }
        }
    }
}

fn write_edge_pave_state(structure: &rustycad_tkbo::bop_ds::DataStructure, edge_indices: &[i32]) {
    integer(structure.pave_blocks_pool().len() as i32);
    for index in edge_indices {
        flag(structure.has_pave_blocks(*index).unwrap());
        integer(structure.shape_info(*index).unwrap().reference());
        let blocks = structure.pave_blocks(*index).unwrap();
        integer(blocks.len() as i32);
        for block in blocks {
            let block = block.lock().unwrap();
            integer(block.original_edge());
            let (first, last) = block.indices();
            let (first_parameter, last_parameter) = block.range();
            integer(first);
            integer(last);
            real(first_parameter);
            real(last_parameter);
            integer(block.ext_paves().len() as i32);
            for pave in block.ext_paves() {
                integer(pave.index());
                real(pave.parameter());
            }
        }
    }
}

fn data_structure_init_shapes() -> (
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Shape,
) {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::Compound};
    let builder = BRepBuilder::default();
    let first = initialized_vertex(&builder, [1.0, 2.0, 3.0], 2e-5);
    let second = initialized_vertex(&builder, [4.0, 5.0, 6.0], 5e-7);
    let third = initialized_vertex(&builder, [7.0, 8.0, 9.0], 1e-4);
    let mut root = Compound::default();
    builder.make_compound(&mut root);
    builder.add(&mut root, &first).expect("unlocked compound");
    builder.add(&mut root, &second).expect("unlocked compound");
    builder.add(&mut root, &first).expect("unlocked compound");
    (root.into_shape(), first, second, third)
}

fn initialized_vertex(
    builder: &rustycad_tkbrep::brep::BRepBuilder,
    point: [f64; 3],
    tolerance: f64,
) -> rustycad_tkbrep::topods::Shape {
    use rustycad_tkbrep::topods::Vertex;
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(
            &mut vertex,
            rustycad_tkmath::gp::Point3::new(point[0], point[1], point[2]),
            tolerance,
        )
        .expect("finite point and positive tolerance");
    vertex.into_shape()
}

fn write_initialized_ranges(structure: &rustycad_tkbo::bop_ds::DataStructure) {
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    for index in 0..structure.nb_ranges() {
        let range = structure.range(index).unwrap();
        integer(range.first());
        integer(range.last());
    }
}

fn write_initialized_indices(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    root: &rustycad_tkbrep::topods::Shape,
    first: &rustycad_tkbrep::topods::Shape,
    second: &rustycad_tkbrep::topods::Shape,
    third: &rustycad_tkbrep::topods::Shape,
) {
    integer(structure.rank(0));
    integer(structure.rank(2));
    integer(structure.rank(3));
    integer(structure.index(root));
    integer(structure.index(first));
    integer(structure.index(second));
    integer(structure.index(third));
}

fn write_initialized_records(structure: &rustycad_tkbo::bop_ds::DataStructure) {
    use rustycad_tkg3d::top_abs::ShapeEnum;
    for index in 0..structure.nb_shapes() {
        let info = structure.shape_info(index).unwrap();
        integer(info.shape_type() as i32);
        integer(info.sub_shapes().len() as i32);
        for child in info.sub_shapes() {
            integer(*child);
        }
        flag(info.box_bounds().is_void());
        if info.shape_type() == ShapeEnum::Vertex {
            real(info.box_bounds().gap());
            for bound in info.box_bounds().get().unwrap() {
                real(bound);
            }
        }
    }
}

fn data_structure_shapes() -> (
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Shape,
) {
    use rustycad_tkbrep::brep::BRepBuilder;
    let builder = BRepBuilder::default();
    (make_vertex(&builder, 1.0), make_vertex(&builder, 2.0))
}

fn make_vertex(
    builder: &rustycad_tkbrep::brep::BRepBuilder,
    x: f64,
) -> rustycad_tkbrep::topods::Shape {
    use rustycad_tkbrep::topods::Vertex;
    use rustycad_tkmath::gp::Point3;
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1e-7)
        .expect("valid finite vertex point");
    vertex.into_shape()
}

fn write_initial_ds_state(structure: &rustycad_tkbo::bop_ds::DataStructure) {
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    integer(structure.arguments().len() as i32);
}

fn write_appended_ds_state(
    structure: &mut rustycad_tkbo::bop_ds::DataStructure,
    first: &rustycad_tkbrep::topods::Shape,
    second: &rustycad_tkbrep::topods::Shape,
) {
    integer(structure.append_shape(first));
    integer(structure.append_shape(second));
    let reversed = first.reversed();
    integer(structure.append_shape(&reversed));
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    integer(structure.rank(0));
    integer(structure.rank(1));
    flag(structure.is_new_shape(0));
    flag(structure.is_new_shape(-1));
    integer(structure.index(first));
    integer(structure.index(&reversed));
    integer(structure.index(second));
    flag(structure.shape(0).unwrap().is_same(first));
    flag(structure.shape_info(0).unwrap().shape().is_same(first));
}

fn write_appended_metadata(
    structure: &mut rustycad_tkbo::bop_ds::DataStructure,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    let mut info = ShapeInfo::default();
    info.set_shape(shape);
    info.set_flag(17);
    integer(structure.append_shape_info(&info));
    integer(structure.shape_info(3).unwrap().flag());
    structure.change_shape_info(3).unwrap().set_flag(21);
    integer(structure.shape_info(3).unwrap().flag());
}

fn write_cleared_ds_state(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    integer(structure.nb_shapes());
    integer(structure.nb_source_shapes());
    integer(structure.nb_ranges());
    integer(structure.arguments().len() as i32);
    integer(structure.index(shape));
}

/// Writes FaceInfo collections in native In, On and section order.
fn write_face_info_counts(info: &FaceInfo) {
    integer(info.index());
    integer(info.pave_blocks_in().extent() as i32);
    integer(info.vertices_in().extent() as i32);
    integer(info.pave_blocks_on().extent() as i32);
    integer(info.vertices_on().extent() as i32);
    integer(info.pave_blocks_section().extent() as i32);
    integer(info.vertices_section().extent() as i32);
}

/// Appends the three test paves; `append_extra_paves(&mut block, &request)`.
fn append_extra_paves(block: &mut PaveBlock, values: &[f64]) {
    for pair in [(6, 7), (8, 9), (10, 11)] {
        block.append_ext_pave(Pave::new(values[pair.0] as i32, values[pair.1]));
    }
}

/// Writes each generated interval in native field order.
fn write_splits(splits: &[PaveBlock]) {
    integer(splits.len() as i32);
    for split in splits {
        let (first, last) = split.range();
        let (i1, i2) = split.indices();
        real(first);
        real(last);
        integer(i1);
        integer(i2);
        integer(split.original_edge());
    }
}

/// Stores and prints shrunk bounds, matching `BOPDS_PaveBlock::ShrunkData`.
fn write_shrunk_data(block: &mut PaveBlock, values: &[f64]) {
    let bounds = BoundingBox3::new(
        rustycad_tkmath::gp::Point3::new(values[15], values[16], values[17]),
        rustycad_tkmath::gp::Point3::new(values[18], values[19], values[20]),
    );
    block.set_shrunk_data(values[21], values[22], &bounds, values[23] != 0.0);
    let (first, last, result, splittable) = block.shrunk_data();
    flag(block.has_shrunk_data());
    real(first);
    real(last);
    flag(splittable);
    for coordinate in result.get().unwrap() {
        real(coordinate);
    }
}
