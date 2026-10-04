//! Native lazy initialization and refresh behavior of face classification pools.
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::fmt::Write;

fn box_structure() -> DataStructure {
    let mut maker = BRepPrimAPIMakeBox::at_point(Point3::default(), 1.0, 1.0, 1.0).unwrap();
    let mut ds = DataStructure::default();
    ds.set_arguments(&[maker.shape().unwrap().clone()]);
    ds.init(1e-7).unwrap();
    ds
}

fn face_indices(ds: &DataStructure) -> Vec<i32> {
    (0..ds.nb_source_shapes())
        .filter(|index| ds.shape_info(*index).unwrap().shape_type() == ShapeEnum::Face)
        .collect()
}

fn append_faces(ds: &DataStructure, stage: i32, rows: &mut String) {
    for index in face_indices(ds) {
        let info = ds.face_info(index).unwrap();
        write!(
            rows,
            "{stage} {index} {} {} {} {} {}",
            i32::from(ds.has_face_info(index).unwrap()),
            info.index(),
            info.pave_blocks_on().extent(),
            info.vertices_on().extent(),
            info.vertices_in().extent()
        )
        .unwrap();
        for vertex in info.vertices_on().iter() {
            write!(rows, " {vertex}").unwrap();
        }
        rows.push('\n');
    }
}

fn initialize_faces(ds: &mut DataStructure) {
    for index in face_indices(ds) {
        ds.change_face_info(index).unwrap();
    }
}

fn update_faces(ds: &mut DataStructure) {
    for index in face_indices(ds) {
        ds.update_face_info_on(index).unwrap();
    }
}

fn initialize_edges(ds: &mut DataStructure) {
    for index in 0..ds.nb_source_shapes() {
        if ds.shape_info(index).unwrap().shape_type() == ShapeEnum::Edge {
            ds.change_pave_blocks(index).unwrap();
        }
    }
}

#[test]
fn face_classification_matches_native_lazy_pool_and_refresh() {
    let mut ds = box_structure();
    let mut rows = String::new();
    append_faces(&ds, 0, &mut rows);
    update_faces(&mut ds);
    append_faces(&ds, 1, &mut rows);
    initialize_faces(&mut ds);
    append_faces(&ds, 2, &mut rows);
    initialize_edges(&mut ds);
    append_faces(&ds, 3, &mut rows);
    update_faces(&mut ds);
    append_faces(&ds, 4, &mut rows);
    initialize_faces(&mut ds);
    append_faces(&ds, 5, &mut rows);
    assert_eq!(rows.lines().count(), 36);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_face_info.expected")
    );
}

#[test]
fn empty_face_record_is_shared_and_reads_do_not_allocate() {
    let mut ds = box_structure();
    let faces = face_indices(&ds);
    assert!(std::ptr::eq(
        ds.face_info(faces[0]).unwrap(),
        ds.face_info(faces[1]).unwrap()
    ));
    ds.init_face_info_in(faces[0]).unwrap();
    assert_eq!(ds.face_info_pool().len(), 0);
    ds.init_face_info(faces[0]).unwrap();
    assert_eq!(ds.face_info_pool().len(), 1);
    ds.init_face_info(faces[0]).unwrap();
    assert_eq!(ds.face_info_pool().len(), 2);
}
