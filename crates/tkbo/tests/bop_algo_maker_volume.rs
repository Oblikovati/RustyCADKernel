//! Native replay of BOPAlgo_MakerVolume; see tools/oracle/tkbo_maker_volume.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::BOPAlgoMakerVolume;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use shape_dump::dump_result;
use std::fmt::Write;

fn faces(shape: &Shape) -> List<Shape> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut result = List::default();
    while explorer.more() {
        result.append(explorer.current().clone());
        explorer.next().unwrap();
    }
    result
}

fn run(label: &str, arguments: &List<Shape>, intersect: bool, avoid: bool, rows: &mut String) {
    let mut maker = BOPAlgoMakerVolume::new();
    maker.set_arguments(arguments);
    maker.set_intersect(intersect);
    maker.set_avoid_internal_shapes(avoid);
    maker.options_mut().set_run_parallel(false);
    maker.perform();
    let (errors, warnings) = (maker.options().has_errors(), maker.options().has_warnings());
    let null = maker.shape().is_null();
    writeln!(
        rows,
        "case {label} errors {} warnings {} faces {} null {}",
        i32::from(errors),
        i32::from(warnings),
        maker.faces().extent(),
        i32::from(null)
    )
    .unwrap();
    if errors || null {
        return;
    }
    writeln!(rows, "type {}", maker.shape().shape_type().unwrap() as i32).unwrap();
    dump_result(&maker.bounding_box().clone().into_shape(), rows);
    dump_result(maker.shape(), rows);
}

#[test]
fn maker_volume_matches_native_volumes() {
    let mut rows = String::new();
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0).unwrap();
    let box_shape = box_maker.shape().unwrap().clone();
    run("box-faces", &faces(&box_shape), true, false, &mut rows);
    run(
        "box-faces-no-intersect",
        &faces(&box_shape),
        false,
        false,
        &mut rows,
    );
    let mut sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(10.0, 10.0, 10.0), 5.0).unwrap();
    let solids: List<Shape> = [box_shape.clone(), sphere.shape().unwrap().clone()]
        .into_iter()
        .collect();
    run("box-sphere", &solids, true, false, &mut rows);
    let mut with_edge = faces(&box_shape);
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(2.0, 2.0, 2.0), Point3::new(8.0, 8.0, 8.0));
    with_edge.append(edge.unwrap().edge().unwrap().into_shape());
    run("internal-edge", &with_edge, true, false, &mut rows);
    run("internal-edge-avoided", &with_edge, true, true, &mut rows);
    run("nothing", &List::default(), true, false, &mut rows);
    let expected = include_str!("../../../tools/oracle/tkbo_maker_volume.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
