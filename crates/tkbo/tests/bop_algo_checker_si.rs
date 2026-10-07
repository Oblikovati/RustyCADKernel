//! BOPAlgo_CheckerSI against tools/oracle/tkbo_checker_si.cxx: errors, per-kind record counts
//! and the interference table in native map order, for several shapes and levels of check.
use rustycad_tkbo::bop_algo::BOPAlgoCheckerSI;
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomSurface, GeomToroidalSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeVertex, BRepBuilderAPINurbsConvert,
};
use std::fmt::Write;

fn cube(x: f64, y: f64, z: f64, size: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), size, size, size).unwrap();
    maker.shape().unwrap().clone()
}

fn compound(first: &Shape, second: &Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut result = Compound::default();
    builder.make_compound(&mut result);
    builder.add(&mut result, first).unwrap();
    builder.add(&mut result, second).unwrap();
    result.into_shape()
}

fn record_counts(structure: &mut DataStructure) -> [usize; 10] {
    [
        structure.interf_vv_records().len(),
        structure.interf_ve_records().len(),
        structure.interf_ee_records().len(),
        structure.interf_vf_records().len(),
        structure.interf_ef_records().len(),
        structure.interf_ff_records().len(),
        structure.interf_vz().len(),
        structure.interf_ez().len(),
        structure.interf_fz().len(),
        structure.interf_zz().len(),
    ]
}

fn run(label: &str, shape: &Shape, level: i32, out: &mut String) {
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(&[shape.clone()].into_iter().collect::<List<Shape>>());
    checker.set_level_of_check(level);
    checker.options_mut().set_run_parallel(false);
    checker.perform();
    let (errors, warnings) = (
        checker.options().has_errors(),
        checker.options().has_warnings(),
    );
    writeln!(
        out,
        "case {label} level {level} errors {} warnings {}",
        i32::from(errors),
        i32::from(warnings)
    )
    .unwrap();
    let structure = checker.data_structure_mut().unwrap();
    let counts = record_counts(structure)
        .map(|count| count.to_string())
        .join(" ");
    writeln!(out, "records {counts}").unwrap();
    writeln!(
        out,
        "sources {} shapes {}",
        structure.nb_source_shapes(),
        structure.nb_shapes()
    )
    .unwrap();
    for pair in structure.interferences().iter() {
        let (first, second) = pair.indices();
        let first_type = structure.shape_info(first).unwrap().shape_type() as i32;
        let second_type = structure.shape_info(second).unwrap().shape_type() as i32;
        writeln!(out, "pair {first} {second} {first_type} {second_type}").unwrap();
    }
}

fn spindle_torus() -> Shape {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(Axis3::default(), 2.0, 3.0).unwrap()).into_cast();
    let maker = BRepBuilderAPIMakeFace::from_surface(&surface, precision::confusion()).unwrap();
    maker.face().unwrap().into_shape()
}

fn two_arguments(single: &Shape, out: &mut String) {
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(
        &[single.clone(), single.clone()]
            .into_iter()
            .collect::<List<Shape>>(),
    );
    checker.perform();
    let errors = i32::from(checker.options().has_errors());
    let structure = i32::from(checker.data_structure().is_some());
    writeln!(out, "case two-arguments errors {errors} ds {structure}").unwrap();
}

#[test]
fn checker_si_matches_native_interference_tables() {
    let mut out = String::new();
    let single = cube(0.0, 0.0, 0.0, 10.0);
    let overlapping = compound(&single, &cube(5.0, 5.0, 5.0, 10.0));
    for level in [0, 2, 3, 4, 5, 9] {
        run("overlapping-boxes", &overlapping, level, &mut out);
    }
    run("single-box", &single, 9, &mut out);
    run(
        "nested-boxes",
        &compound(&single, &cube(2.0, 2.0, 2.0, 4.0)),
        9,
        &mut out,
    );
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(3.0, 3.0, 3.0))
        .unwrap()
        .vertex()
        .unwrap();
    run(
        "box-vertex",
        &compound(&single, &vertex.into_shape()),
        9,
        &mut out,
    );
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(5.0, 5.0, -5.0),
        Point3::new(5.0, 5.0, 15.0),
    );
    run(
        "box-edge",
        &compound(&single, &edge.unwrap().edge().unwrap().into_shape()),
        9,
        &mut out,
    );
    let mut nurbs = BRepBuilderAPINurbsConvert::with_shape(&single, false).unwrap();
    run("nurbs-box", &nurbs.shape().unwrap().clone(), 9, &mut out);
    run("spindle-torus", &spindle_torus(), 5, &mut out);
    two_arguments(&single, &mut out);
    let expected = include_str!("../../../tools/oracle/tkbo_checker_si.expected");
    for (line, (actual, wanted)) in out.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(out.lines().count(), expected.lines().count());
}
