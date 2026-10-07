//! Native replay of BRepAlgoAPI_BuilderAlgo::SimplifyResult; see
//! tools/oracle/tkbo_simplify_result.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::brep_algo_api::{
    BRepAlgoAPIBooleanOperation, BRepAlgoAPIFuse, BRepAlgoAPISection,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPISewing,
};
use shape_dump::dump_result;
use std::fmt::Write;

/// Sorted by index like the oracle: the merged history images follow TShape-address order.
fn list<'a>(
    tag: &str,
    shapes: impl Iterator<Item = &'a Shape>,
    all: &IndexedMapOfShape,
    rows: &mut String,
) {
    let mut items: Vec<usize> = shapes.map(|shape| all.find_index(shape)).collect();
    items.sort_unstable();
    write!(rows, " {tag} {}:", items.len()).unwrap();
    for item in items {
        write!(rows, " {item}").unwrap();
    }
}

fn dump(label: &str, algo: &BRepAlgoAPIBooleanOperation, input: &Shape, rows: &mut String) {
    writeln!(rows, "step {label} errors {}", i32::from(algo.has_errors())).unwrap();
    let all = dump_result(algo.shape(), rows);
    let mut explorer = Explorer::new(input, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        let face = explorer.current();
        write!(
            rows,
            "face deleted {}",
            i32::from(algo.is_deleted(face).unwrap())
        )
        .unwrap();
        list("modified", algo.modified(face).unwrap().iter(), &all, rows);
        list(
            "generated",
            algo.generated(face).unwrap().iter(),
            &all,
            rows,
        );
        writeln!(rows).unwrap();
        explorer.next().unwrap();
    }
    list(
        "section-edges",
        algo.section_edges().unwrap().iter(),
        &all,
        rows,
    );
    writeln!(rows).unwrap();
}

fn plane_face(plane: Plane3, bounds: (f64, f64, f64, f64)) -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, bounds)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn fuse_case(rows: &mut String) {
    let box1 = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 15.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let box2 = BRepPrimAPIMakeBox::at_point(Point3::new(3.0, 7.0, 0.0), 10.0, 10.0, 15.0);
    let box2 = box2.unwrap().shape().unwrap().clone();
    let mut fuse = BRepAlgoAPIFuse::from_shapes(&box1, &box2).unwrap();
    dump("fuse", &fuse, &box1, rows);
    fuse.simplify_result(true, true, precision::angular())
        .unwrap();
    dump("fuse-simplified", &fuse, &box1, rows);
    fuse.simplify_result(false, false, precision::angular())
        .unwrap();
    dump("fuse-no-unify", &fuse, &box1, rows);
}

fn section_case(rows: &mut String) {
    let plane = Plane3::new(Point3::new(0.0, 0.0, 0.0), gp::dz()).unwrap();
    let mut sewing = BRepBuilderAPISewing::default();
    for bounds in [
        (-10.0, 10.0, -10.0, 10.0),
        (10.0, 30.0, -10.0, 10.0),
        (30.0, 50.0, -10.0, 10.0),
    ] {
        sewing.add(&plane_face(plane, bounds)).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    let shell = sewing.sewed_shape().clone();
    let cutting_plane = Plane3::new(Point3::new(0.0, 5.0, 0.0), gp::dy()).unwrap();
    let cutting = plane_face(cutting_plane, (-100.0, 100.0, -100.0, 100.0));
    let mut section = BRepAlgoAPISection::from_shapes(&shell, &cutting, None, true).unwrap();
    dump("section", &section, &shell, rows);
    section
        .simplify_result(true, false, precision::angular())
        .unwrap();
    dump("section-simplified", &section, &shell, rows);
}

#[test]
fn simplify_result_matches_native_shapes_and_history() {
    let mut rows = String::new();
    fuse_case(&mut rows);
    section_case(&mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_simplify_result.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
