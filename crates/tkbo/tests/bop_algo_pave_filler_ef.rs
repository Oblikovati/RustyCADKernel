//! Native differential coverage for one interior line/plane edge-face crossing.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Shape, Vertex},
};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomSurface};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

#[test]
fn perform_ef_records_crossing_vertex_splits_block_and_updates_face_info() {
    let edge = edge_argument();
    let face = face_argument();
    let mut filler = configured_filler(&edge, &face, false, 1.0e-6);
    let before = filler.data_structure().unwrap();
    let edge_index = before.index(&edge);
    let face_index = before.index(&face);
    let mut rows = format!("before {} {edge_index} {face_index}\n", before.nb_shapes());
    filler.perform_ef().unwrap();
    append_result(&mut filler, edge_index, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef.expected")
    );
}

#[test]
fn perform_ef_endpoint_contact_keeps_the_source_block_unsplit() {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0))
            .unwrap()
            .edge()
            .unwrap()
            .into_shape();
    let face = face_argument();
    let mut filler = configured_filler(&edge, &face, false, 1.0e-6);
    let edge_index = filler.data_structure().unwrap().index(&edge);
    let face_index = filler.data_structure().unwrap().index(&face);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_endpoint_result(&mut filler, edge_index, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_endpoint.expected")
    );
}

#[test]
fn perform_ef_localizes_bezier_surface_and_records_native_vertex() {
    let edge = line_edge((0.5, 0.5, -1.0), (0.5, 0.5, 1.0));
    let face = bezier_face();
    let mut filler = configured_filler(&edge, &face, false, 1.0e-7);
    filler.perform_ef().unwrap();
    let ds = filler.data_structure_mut().unwrap();
    let mut rows = format!("after {} {}\n", ds.nb_shapes(), ds.interf_ef().len());
    append_coplanar_record(ds, &mut rows);
    append_bezier_vertices(ds, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_bezier.expected")
    );
}

fn append_bezier_vertices(ds: &rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    for index in 0..ds.nb_shapes() {
        let shape = ds.shape(index).unwrap();
        if shape.shape_type().unwrap() != rustycad_tkg3d::top_abs::ShapeEnum::Vertex {
            continue;
        }
        let point = BRepTool::pnt(&Vertex::unchecked(shape.clone())).unwrap();
        writeln!(rows, "v {:.9} {:.9} {:.9}", point.x(), point.y(), point.z()).unwrap();
    }
}

#[test]
fn perform_ef_records_coplanar_edge_without_a_generated_vertex() {
    let edge = coplanar_edge_argument();
    let face = face_argument();
    let mut filler = configured_filler(&edge, &face, false, 0.0);
    let edge_index = filler.data_structure().unwrap().index(&edge);
    let face_index = filler.data_structure().unwrap().index(&face);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_coplanar_result(&mut filler, edge_index, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_coplanar.expected")
    );
}

#[test]
fn perform_ef_common_edge_records_face_membership_and_tolerance() {
    let edge = coplanar_edge_argument();
    let face = face_argument();
    BRepBuilder::default()
        .update_face_tolerance(&Face::unchecked(face.clone()), 0.001)
        .unwrap();
    let mut filler = configured_filler(&edge, &face, false, 0.0);
    let (edge_index, face_index) = seed_face_endpoint_membership(&mut filler, &edge, &face);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_common_block_result(&mut filler, edge_index, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_commonblock.expected")
    );
}

fn seed_face_endpoint_membership(
    filler: &mut BOPAlgoPaveFiller,
    edge: &Shape,
    face: &Shape,
) -> (i32, i32) {
    let ds = filler.data_structure_mut().unwrap();
    let edge_index = ds.index(edge);
    let face_index = ds.index(face);
    let endpoints = ds.shape_info(edge_index).unwrap().sub_shapes().to_vec();
    let info = ds.change_face_info(face_index).unwrap();
    for endpoint in endpoints {
        info.change_vertices_on().add(endpoint);
    }
    (edge_index, face_index)
}

fn coplanar_edge_argument() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn fused_edge_arguments() -> [Shape; 3] {
    [
        line_edge((-1.0, 0.0, -1.0), (1.0, 0.0, 1.0)),
        line_edge((0.0, -1.0, -1.0), (0.0, 1.0, 1.0)),
        line_edge((-1.0, -1.0, -1.0), (1.0, 1.0, 1.0)),
    ]
}

fn near_edge_arguments() -> [Shape; 3] {
    [
        line_edge((-1.0, 0.0, -1.0), (1.0, 0.0, 1.0)),
        line_edge((-1.0, 0.0003, -1.0), (1.0, 0.0003, 1.0)),
        line_edge((-1.0, 0.0006, -1.0), (1.0, 0.0006, 1.0)),
    ]
}

fn line_edge(first: (f64, f64, f64), last: (f64, f64, f64)) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(
        Point3::new(first.0, first.1, first.2),
        Point3::new(last.0, last.1, last.2),
    )
    .unwrap()
    .edge()
    .unwrap()
    .into_shape()
}

fn bezier_face() -> Shape {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            *poles.change_value(u, v).unwrap() =
                Point3::new(f64::from(u - 1), f64::from(v - 1), 0.0);
        }
    }
    let surface = Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast::<dyn GeomSurface>();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0.0, 1.0, 0.0, 1.0), 1.0e-7)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

#[test]
fn perform_ef_repeated_calls_preserve_native_results_for_fuzzy_and_safe_modes() {
    let cases = [(false, 1.0e-6), (true, 1.0e-6), (false, 0.0)];
    let mut rows = String::new();
    for (mode, (non_destructive, fuzzy)) in cases.into_iter().enumerate() {
        append_repeated_case(mode as i32, non_destructive, fuzzy, &mut rows);
    }
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_repeat.expected")
    );
}

#[test]
fn perform_ef_fuses_three_intersections_and_shares_the_vertex_across_blocks() {
    let edges = fused_edge_arguments();
    let face = face_argument();
    let (mut filler, indices, face_index) = fused_filler(&edges, &face, 1.0e-6);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_fused_result(&mut filler, &indices, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_fused.expected")
    );
}

#[test]
fn perform_ef_transitively_fuses_nearby_crossings_using_native_center_and_tolerance() {
    let edges = near_edge_arguments();
    let face = face_argument();
    let (mut filler, indices, face_index) = fused_filler(&edges, &face, 4.0e-4);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_fused_result(&mut filler, &indices, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_near.expected")
    );
}

fn fused_filler(edges: &[Shape], face: &Shape, fuzzy: f64) -> (BOPAlgoPaveFiller, Vec<i32>, i32) {
    let mut filler = BOPAlgoPaveFiller::new();
    edges.iter().for_each(|edge| filler.add_argument(edge));
    filler.add_argument(face);
    filler.options_mut().set_fuzzy_value(fuzzy);
    filler.initialize().unwrap();
    let structure = filler.data_structure().unwrap();
    let edge_indices = edges.iter().map(|edge| structure.index(edge)).collect();
    let face_index = structure.index(face);
    (filler, edge_indices, face_index)
}

fn edge_argument() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-1.0, 0.0, -1.0), Point3::new(1.0, 0.0, 1.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn face_argument() -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2.0, 2.0, -2.0, 2.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn configured_filler(
    edge: &Shape,
    face: &Shape,
    non_destructive: bool,
    fuzzy: f64,
) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(edge);
    filler.add_argument(face);
    filler.set_non_destructive(non_destructive);
    filler.options_mut().set_fuzzy_value(fuzzy);
    filler.initialize().unwrap();
    filler
}

fn append_result(filler: &mut BOPAlgoPaveFiller, edge: i32, face: i32, rows: &mut String) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "after {} {} 0", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    append_ef_record(ds, rows);
    append_edge_blocks(ds, edge, rows);
    append_face_info(ds, face, rows);
    append_generated_vertex(ds, rows);
}

fn append_endpoint_result(filler: &mut BOPAlgoPaveFiller, edge: i32, face: i32, rows: &mut String) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(
        rows,
        "after {} {} {}",
        ds.nb_shapes(),
        ds.interf_ef().len(),
        ds.interf_vf().len()
    )
    .unwrap();
    append_edge_blocks(ds, edge, rows);
    append_face_info(ds, face, rows);
}

fn append_coplanar_result(filler: &mut BOPAlgoPaveFiller, edge: i32, face: i32, rows: &mut String) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "after {} {}", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    append_coplanar_record(ds, rows);
    append_coplanar_face(ds, face, rows);
    writeln!(rows, "edge-blocks {}", ds.pave_blocks(edge).unwrap().len()).unwrap();
}

fn append_common_block_result(
    filler: &mut BOPAlgoPaveFiller,
    edge: i32,
    face: i32,
    rows: &mut String,
) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "after {} {}", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    append_coplanar_record(ds, rows);
    writeln!(
        rows,
        "face {} {}",
        ds.has_face_info(face).unwrap() as u8,
        ds.face_info(face).unwrap().pave_blocks_on().extent()
    )
    .unwrap();
    let block = ds.pave_blocks(edge).unwrap()[0].clone();
    append_common_metadata(ds, edge, &block, rows);
}

fn append_common_metadata(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    block: &rustycad_tkbo::bop_ds::PaveBlockHandle,
    rows: &mut String,
) {
    writeln!(rows, "pb {edge} {}", ds.is_common_block(block) as u8).unwrap();
    if let Some(common) = ds.common_block(block) {
        let common = common.lock().unwrap();
        writeln!(
            rows,
            "cb {} {:.9}",
            common.faces().len(),
            common.tolerance()
        )
        .unwrap();
        for face in common.faces() {
            writeln!(rows, "cf {face}").unwrap();
        }
    }
    append_endpoint_tolerances(ds, edge, rows);
}

fn append_endpoint_tolerances(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    rows: &mut String,
) {
    for vertex in ds.shape_info(edge).unwrap().sub_shapes() {
        let shape = ds.shape(*vertex).unwrap().clone();
        let tolerance = BRepTool::vertex_tolerance(&Vertex::unchecked(shape)).unwrap();
        writeln!(rows, "v {vertex} {tolerance:.9}").unwrap();
    }
}

fn append_coplanar_record(ds: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let record = ds.interf_ef().value(0).unwrap();
    writeln!(
        rows,
        "ef {} {} {} {}",
        record.index1(),
        record.index2(),
        record.index_new(),
        record.common_part().shape_type() as i32
    )
    .unwrap();
}

fn append_coplanar_face(ds: &rustycad_tkbo::bop_ds::DataStructure, face: i32, rows: &mut String) {
    let info = ds.face_info(face).unwrap();
    writeln!(
        rows,
        "face {} {} {}",
        ds.has_face_info(face).unwrap() as u8,
        info.pave_blocks_on().extent(),
        info.vertices_in().extent()
    )
    .unwrap();
}

fn append_repeated_case(mode: i32, non_destructive: bool, fuzzy: f64, rows: &mut String) {
    let edge = edge_argument();
    let face = face_argument();
    let mut filler = configured_filler(&edge, &face, non_destructive, fuzzy);
    let structure = filler.data_structure().unwrap();
    let edge_index = structure.index(&edge);
    let face_index = structure.index(&face);
    for pass in 0..2 {
        filler.perform_ef().unwrap();
        append_repeat_snapshot(&mut filler, mode, pass, edge_index, face_index, rows);
    }
}

fn append_fused_result(
    filler: &mut BOPAlgoPaveFiller,
    edges: &[i32],
    face: i32,
    rows: &mut String,
) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "after {} {}", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    append_fused_records(ds, rows);
    for edge in edges {
        append_named_edge_blocks(ds, *edge, rows);
    }
    append_face_info(ds, face, rows);
    append_generated_vertex(ds, rows);
}

fn append_fused_records(ds: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    for record in ds.interf_ef().iter() {
        writeln!(
            rows,
            "ef {} {} {} {}",
            record.index1(),
            record.index2(),
            record.index_new(),
            record.common_part().shape_type() as i32
        )
        .unwrap();
    }
}

fn append_named_edge_blocks(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    rows: &mut String,
) {
    let blocks = ds.pave_blocks(edge).unwrap();
    writeln!(rows, "blocks {edge} {}", blocks.len()).unwrap();
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.range();
        let (first_index, last_index) = block.indices();
        writeln!(rows, "pb {first_index} {last_index} {first:.9} {last:.9}").unwrap();
    }
}

fn append_repeat_snapshot(
    filler: &mut BOPAlgoPaveFiller,
    mode: i32,
    pass: i32,
    edge: i32,
    face: i32,
    rows: &mut String,
) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(
        rows,
        "case {mode} {pass} {} {}",
        ds.nb_shapes(),
        ds.interf_ef().len()
    )
    .unwrap();
    append_repeat_record(ds, rows);
    append_edge_blocks(ds, edge, rows);
    append_face_info(ds, face, rows);
}

fn append_repeat_record(ds: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let record = ds.interf_ef().value(0).unwrap();
    writeln!(
        rows,
        "ef {} {} {} {}",
        record.index1(),
        record.index2(),
        record.index_new(),
        record.common_part().shape_type() as i32
    )
    .unwrap();
}

fn append_ef_record(ds: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let record = ds.interf_ef().value(0).unwrap();
    let common = record.common_part();
    writeln!(
        rows,
        "ef {} {} {} {} {:.9}",
        record.index1(),
        record.index2(),
        record.index_new(),
        common.shape_type() as i32,
        common.vertex_parameter1()
    )
    .unwrap();
}

fn append_edge_blocks(ds: &rustycad_tkbo::bop_ds::DataStructure, edge: i32, rows: &mut String) {
    let blocks = ds.pave_blocks(edge).unwrap();
    writeln!(rows, "blocks {}", blocks.len()).unwrap();
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.range();
        let (first_index, last_index) = block.indices();
        writeln!(rows, "pb {first_index} {last_index} {first:.9} {last:.9}").unwrap();
    }
}

fn append_face_info(ds: &rustycad_tkbo::bop_ds::DataStructure, face: i32, rows: &mut String) {
    let info = ds.face_info(face).unwrap();
    writeln!(
        rows,
        "face {} {}",
        ds.has_face_info(face).unwrap() as u8,
        info.vertices_in().extent()
    )
    .unwrap();
}

fn append_generated_vertex(ds: &rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let index = ds.nb_shapes() - 1;
    let vertex = Vertex::unchecked(ds.shape(index).unwrap().clone());
    let point = BRepTool::pnt(&vertex).unwrap();
    writeln!(
        rows,
        "v {index} {:.9} {:.9} {:.9} {:.9}",
        point.x(),
        point.y(),
        point.z(),
        BRepTool::vertex_tolerance(&vertex).unwrap()
    )
    .unwrap();
}
