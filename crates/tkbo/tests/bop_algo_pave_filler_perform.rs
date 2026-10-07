//! Native replay of the whole BOPAlgo_PaveFiller::Perform pipeline on solid pairs.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbo::bop_ds::{DataStructure, PaveBlockHandle, PaveBlockIndexMap, VertexIndexSet};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::fmt::Write;

fn h(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn write_block(ds: &DataStructure, block: &PaveBlockHandle, rows: &mut String) {
    let (indices, range, edge, original) = {
        let locked = block.lock().unwrap();
        (
            locked.indices(),
            locked.range(),
            locked.edge(),
            locked.original_edge(),
        )
    };
    write!(
        rows,
        " [{} {} {} {} {edge} {original}",
        indices.0,
        indices.1,
        h(range.0),
        h(range.1)
    )
    .unwrap();
    if let Some(common) = ds.common_block(block) {
        let common = common.lock().unwrap();
        write!(
            rows,
            " cb{}:{}:{}",
            common.pave_blocks().len(),
            common.faces().len(),
            h(common.tolerance())
        )
        .unwrap();
    }
    rows.push(']');
}

fn write_index_map(tag: &str, set: &VertexIndexSet, rows: &mut String) {
    write!(rows, " {tag}").unwrap();
    for vertex in set.iter() {
        write!(rows, " {vertex}").unwrap();
    }
}

fn write_block_map(map: &PaveBlockIndexMap, rows: &mut String) {
    rows.push_str(" |");
    for block in map.iter() {
        write!(rows, " {}", block.lock().unwrap().edge()).unwrap();
    }
}

fn write_shape(ds: &DataStructure, index: i32, rows: &mut String) {
    let info = ds.shape_info(index).unwrap();
    match info.shape_type() {
        ShapeEnum::Vertex => {
            let tolerance =
                BRepTool::vertex_tolerance(&Vertex::unchecked(info.shape().clone())).unwrap();
            writeln!(
                rows,
                "v {index} {} {}",
                h(tolerance),
                ds.same_domain_index(index)
            )
            .unwrap();
        }
        ShapeEnum::Edge => {
            let tolerance =
                BRepTool::edge_tolerance(&Edge::unchecked(info.shape().clone())).unwrap();
            write!(rows, "e {index} {} {}", h(tolerance), info.reference()).unwrap();
            for block in ds.pave_blocks(index).unwrap() {
                write_block(ds, block, rows);
            }
            rows.push('\n');
        }
        ShapeEnum::Face if ds.has_face_info(index).unwrap() => {
            let face = ds.face_info(index).unwrap();
            write!(rows, "f {index}").unwrap();
            write_block_map(face.pave_blocks_on(), rows);
            write_block_map(face.pave_blocks_in(), rows);
            write_block_map(face.pave_blocks_section(), rows);
            write_index_map("on", face.vertices_on(), rows);
            write_index_map("in", face.vertices_in(), rows);
            write_index_map("sc", face.vertices_section(), rows);
            rows.push('\n');
        }
        _ => {}
    }
}

fn dump(filler: &mut BOPAlgoPaveFiller, rows: &mut String) {
    let errors = filler.options().has_errors();
    let warnings = filler.options().has_warnings();
    writeln!(
        rows,
        "errors {} warnings {}",
        i32::from(errors),
        i32::from(warnings)
    )
    .unwrap();
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "shapes {} {}", ds.nb_source_shapes(), ds.nb_shapes()).unwrap();
    let counts = [
        ds.interf_vv_records().len(),
        ds.interf_ve_records().len(),
        ds.interf_ee_records().len(),
        ds.interf_vf_records().len(),
        ds.interf_ef_records().len(),
        ds.interf_ff_records().len(),
    ];
    writeln!(
        rows,
        "interf {} {} {} {} {} {}",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5]
    )
    .unwrap();
    for index in 0..ds.nb_shapes() {
        write_shape(ds, index, rows);
    }
    for record in ds.interf_ff_records().iter() {
        let (first, second) = record.indices();
        let tangent = i32::from(record.tangent_faces());
        writeln!(
            rows,
            "ff {first} {second} {tangent} {} {}",
            record.curves().len(),
            record.points().len()
        )
        .unwrap();
        for curve in record.curves().iter() {
            write!(rows, " c {}", h(curve.tolerance())).unwrap();
            for block in curve.pave_blocks() {
                write_block(ds, block, rows);
            }
            rows.push('\n');
        }
    }
}

fn run(label: &str, first: Shape, second: Shape, rows: &mut String) {
    writeln!(rows, "case {label}").unwrap();
    let mut filler = BOPAlgoPaveFiller::new();
    let arguments: List<Shape> = [first, second].into_iter().collect();
    filler.set_arguments(&arguments);
    filler.perform();
    dump(&mut filler, rows);
}

fn solid_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn pave_filler_perform_matches_native_oracle() {
    let mut rows = String::new();
    let unit = solid_box(Point3::default(), 10.);
    run(
        "box-box",
        unit.clone(),
        solid_box(Point3::new(5., 5., 5.), 10.),
        &mut rows,
    );
    run(
        "box-box-face",
        unit.clone(),
        BRepPrimAPIMakeBox::at_point(Point3::new(10., 2., 2.), 5., 5., 5.)
            .unwrap()
            .shape()
            .unwrap()
            .clone(),
        &mut rows,
    );
    let axes = Axis2::new(
        Point3::new(5., 5., -5.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 20.)
        .shape()
        .unwrap()
        .clone();
    run("box-cylinder", unit.clone(), cylinder, &mut rows);
    let sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(10., 10., 10.), 4.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    run("box-sphere", unit, sphere, &mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_pave_filler_perform.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, expected);
}
