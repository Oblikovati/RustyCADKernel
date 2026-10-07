//! Native replay of BOPDS_DS pave-block updates, common-block regrouping and face queries.
use rustycad_tkbo::bop_ds::{
    CommonBlock, DataStructure, Pave, PaveBlockHandle, common_block_handle,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use std::fmt::Write;
use std::sync::Arc;

fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn edge(x1: f64, y1: f64, x2: f64, y2: f64) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(x1, y1, 0.), Point3::new(x2, y2, 0.))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn vertex(x: f64, y: f64, tolerance: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, y, 0.), tolerance)
        .unwrap();
    vertex.into_shape()
}

fn initialized(arguments: &[Shape], fuzz: f64) -> DataStructure {
    let mut ds = DataStructure::default();
    ds.set_arguments(arguments);
    ds.init(fuzz).unwrap();
    ds
}

fn first_block(ds: &mut DataStructure, edge: i32) -> PaveBlockHandle {
    Arc::clone(&ds.change_pave_blocks(edge).unwrap()[0])
}

fn write_block(ds: &DataStructure, block: &PaveBlockHandle, rows: &mut String) {
    let (first, last, range) = {
        let locked = block.lock().unwrap();
        let (first, last) = locked.indices();
        (first, last, locked.range())
    };
    write!(rows, " [{first} {last} {} {}", h(range.0), h(range.1)).unwrap();
    if let Some(common) = ds.common_block(block) {
        let real = ds.real_pave_block(block).unwrap();
        let common = common.lock().unwrap();
        let first_is_self = Arc::ptr_eq(common.pave_block1().unwrap(), block);
        write!(
            rows,
            " cb {} {} {} {}",
            common.pave_blocks().len(),
            i32::from(first_is_self),
            common.faces().len(),
            real.lock().unwrap().original_edge()
        )
        .unwrap();
    }
    rows.push(']');
}

fn write_blocks(ds: &DataStructure, edge: i32, rows: &mut String) {
    write!(
        rows,
        "edge {edge} ref {}",
        ds.shape_info(edge).unwrap().reference()
    )
    .unwrap();
    for block in ds.pave_blocks(edge).unwrap() {
        write_block(ds, block, rows);
    }
    write!(rows, "\npaves {edge}").unwrap();
    for pave in ds.paves(edge).unwrap() {
        write!(rows, " {}:{}", pave.index(), h(pave.parameter())).unwrap();
    }
    rows.push('\n');
}

fn plain_updates(rows: &mut String) {
    let arguments = [
        edge(0., 0., 3., 0.),
        edge(0., 1., 3., 1.),
        vertex(1., 0., 1e-7),
        vertex(2., 0., 1e-7),
    ];
    let mut ds = initialized(&arguments, 0.);
    let block = first_block(&mut ds, 0);
    block.lock().unwrap().append_ext_pave(Pave::new(7, 2.));
    block.lock().unwrap().append_ext_pave(Pave::new(6, 1.));
    ds.update_pave_blocks();
    write_blocks(&ds, 0, rows);
    let last = Arc::clone(ds.change_pave_blocks(0).unwrap().last().unwrap());
    last.lock().unwrap().append_ext_pave(Pave::new(6, 2.5));
    ds.update_pave_block(&last).unwrap();
    write_blocks(&ds, 0, rows);
    ds.change_pave_blocks(3).unwrap();
    ds.release_pave_blocks().unwrap();
    write_blocks(&ds, 0, rows);
    write_blocks(&ds, 3, rows);
    plain_queries(&mut ds, rows);
}

fn plain_queries(ds: &mut DataStructure, rows: &mut String) {
    let sub = (
        ds.is_sub_shape(1, 0).unwrap(),
        ds.is_sub_shape(4, 0).unwrap(),
    );
    writeln!(rows, "sub {} {}", i32::from(sub.0), i32::from(sub.1)).unwrap();
    ds.add_interference(6, 1);
    let flags = [
        ds.has_interference_shape_subshapes(6, 0).unwrap(),
        ds.has_interference_shape_subshapes_mode(6, 0, false)
            .unwrap(),
        ds.has_interference_sub_shapes(0, 0).unwrap(),
        ds.has_interference_sub_shapes(3, 0).unwrap(),
    ];
    let flags: Vec<String> = flags
        .iter()
        .map(|flag| i32::from(*flag).to_string())
        .collect();
    writeln!(rows, "interf {}", flags.join(" ")).unwrap();
    ds.dump(rows).unwrap();
}

fn common_blocks(offset: f64, fuzz: f64, use_sd: bool, rows: &mut String) {
    let arguments = [
        edge(0., 0., 10., 0.),
        edge(0., offset, 10., offset),
        vertex(4., 0., 1e-7),
    ];
    let mut ds = initialized(&arguments, fuzz);
    if use_sd {
        ds.add_shape_sd(4, 1);
        ds.add_shape_sd(5, 2);
        ds.update_pave_blocks_with_sd_vertices();
    }
    let (first, second) = (first_block(&mut ds, 0), first_block(&mut ds, 3));
    let mut common = CommonBlock::new();
    common.add_pave_block(Arc::clone(&first));
    common.add_pave_block(Arc::clone(&second));
    common.add_face(7);
    let common = common_block_handle(common);
    ds.set_common_block(&first, &common);
    ds.set_common_block(&second, &common);
    first.lock().unwrap().append_ext_pave(Pave::new(6, 4.));
    second.lock().unwrap().append_ext_pave(Pave::new(6, 4.));
    second.lock().unwrap().append_ext_pave(Pave::new(6, 7.));
    ds.update_common_block(&common, fuzz).unwrap();
    writeln!(rows, "case {} {} {}", h(offset), h(fuzz), i32::from(use_sd)).unwrap();
    write_blocks(&ds, 0, rows);
    write_blocks(&ds, 3, rows);
    ds.release_pave_blocks().unwrap();
    write_blocks(&ds, 0, rows);
}

fn box_structure() -> (DataStructure, Vec<i32>) {
    let solid = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut ds = initialized(&[solid], 0.);
    let mut faces = Vec::new();
    for index in 0..ds.nb_source_shapes() {
        let kind = ds.shape_info(index).unwrap().shape_type();
        if kind == ShapeEnum::Edge {
            for block in ds.change_pave_blocks(index).unwrap() {
                block.lock().unwrap().set_edge(index);
            }
        }
        if kind == ShapeEnum::Face {
            faces.push(index);
        }
    }
    for face in &faces {
        ds.change_face_info(*face).unwrap();
    }
    (ds, faces)
}

fn shared_edge_rows(ds: &DataStructure, faces: &[i32], rows: &mut String) {
    for first in faces {
        for second in faces {
            write!(rows, "shared {first} {second}").unwrap();
            for edge in ds.shared_edges(*first, *second).unwrap() {
                write!(rows, " {edge}").unwrap();
            }
            rows.push('\n');
        }
    }
}

fn seed_first_face(ds: &mut DataStructure, face: i32) {
    let info = ds.change_face_info(face).unwrap();
    let on: Vec<PaveBlockHandle> = info.pave_blocks_on().iter().cloned().collect();
    for block in on.iter().step_by(2) {
        info.change_pave_blocks_in().add(Arc::clone(block));
    }
    info.change_vertices_in().add(99);
    info.change_vertices_in().add(-3);
    info.change_vertices_section().add(98);
}

fn on_in_rows(ds: &DataStructure, faces: &[i32], rows: &mut String) {
    let mut shared = rustycad_tkbo::bop_ds::SubShapesOnIn::default();
    ds.sub_shapes_on_in(faces[0], faces[2], &mut shared)
        .unwrap();
    write!(
        rows,
        "onin {} {} {} {} :",
        shared.vertices_on_in.extent(),
        shared.vertices_common.extent(),
        shared.pave_blocks_on_in.extent(),
        shared.common_pave_blocks.extent()
    )
    .unwrap();
    for vertex in shared.vertices_on_in.iter() {
        write!(rows, " {vertex}").unwrap();
    }
    rows.push_str(" :");
    for vertex in shared.vertices_common.iter() {
        write!(rows, " {vertex}").unwrap();
    }
    for block in shared.pave_blocks_on_in.iter() {
        write!(rows, " {}", block.lock().unwrap().original_edge()).unwrap();
    }
    rows.push('\n');
}

fn box_queries(rows: &mut String) {
    let (mut ds, faces) = box_structure();
    shared_edge_rows(&ds, &faces, rows);
    seed_first_face(&mut ds, faces[0]);
    let alone: Vec<String> = ds
        .alone_vertices(faces[0])
        .unwrap()
        .iter()
        .map(i32::to_string)
        .collect();
    writeln!(rows, "alone {}", alone.join(" ")).unwrap();
    let before = ds.face_info(faces[0]).unwrap().pave_blocks_in().extent();
    writeln!(rows, "in-before {before}").unwrap();
    on_in_rows(&ds, &faces, rows);
    ds.refine_face_info_in().unwrap();
    let after = ds.face_info(faces[0]).unwrap().pave_blocks_in().extent();
    writeln!(rows, "in-after {after}").unwrap();
    let solid = (0..ds.nb_source_shapes())
        .rfind(|index| ds.shape_info(*index).unwrap().shape_type() == ShapeEnum::Solid)
        .unwrap_or(-1);
    let mut bounds = BoundingBox3::default();
    ds.build_bnd_box_solid(solid, &mut bounds, true).unwrap();
    let flags = (i32::from(bounds.is_void()), i32::from(bounds.is_whole()));
    writeln!(rows, "box {solid} {} {}", flags.0, flags.1).unwrap();
}

#[test]
fn ds_updates_match_native_oracle() {
    let mut rows = String::new();
    plain_updates(&mut rows);
    common_blocks(0., 0., true, &mut rows);
    common_blocks(1e-6, 1e-5, true, &mut rows);
    common_blocks(0.5, 0., true, &mut rows);
    common_blocks(0., 0., false, &mut rows);
    box_queries(&mut rows);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_ds_updates.expected")
    );
}
