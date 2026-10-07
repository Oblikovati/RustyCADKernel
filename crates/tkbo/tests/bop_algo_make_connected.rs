//! Native replay of BOPAlgo_MakeConnected; see tools/oracle/tkbo_make_connected.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::{
    AlertMultiDimensionalArguments, AlertTooFewArguments, BOPAlgoMakeConnected, PeriodicityParams,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::RuntimeTyped;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use shape_dump::dump_result;
use std::fmt::Write;

fn profile() -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, z) in [
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 10.0),
        (5.0, 10.0),
        (5.0, 5.0),
        (0.0, 5.0),
    ] {
        polygon.add_point(Point3::new(x, 0.0, z)).unwrap();
    }
    polygon.close().unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap();
    let mut prism =
        BRepPrimAPIMakePrism::new(&face.into_shape(), Vector3::new(0.0, 5.0, 0.0), false, true)
            .unwrap();
    prism.shape().unwrap().clone()
}

fn box_at(x: f64, y: f64, z: f64, size: (f64, f64, f64)) -> Shape {
    let mut maker =
        BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), size.0, size.1, size.2).unwrap();
    maker.shape().unwrap().clone()
}

/// Sorted by index like the oracle: the history images follow TShape-address order.
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

fn explore(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    found
}

/// The maps of the arguments and of their sub-shapes.
struct Inputs {
    arguments: IndexedMapOfShape,
    sub_shapes: IndexedMapOfShape,
}

impl Inputs {
    fn of(arguments: &[Shape]) -> Self {
        let mut inputs = Inputs {
            arguments: IndexedMapOfShape::default(),
            sub_shapes: IndexedMapOfShape::default(),
        };
        for argument in arguments {
            inputs.arguments.add(argument.clone());
            topexp::map_shapes(argument, &mut inputs.sub_shapes, true, true);
        }
        inputs
    }
}

fn materials(
    label: &str,
    maker: &BOPAlgoMakeConnected,
    result: &Shape,
    inputs: &Inputs,
    rows: &mut String,
) {
    writeln!(
        rows,
        "materials {label} warnings {}",
        i32::from(maker.options().has_warnings())
    )
    .unwrap();
    let all = dump_result(result, rows);
    for face in explore(result, ShapeEnum::Face) {
        write!(rows, "face {}", all.find_index(&face)).unwrap();
        list(
            "positive",
            maker.materials_on_positive_side(&face).iter(),
            &inputs.arguments,
            rows,
        );
        list(
            "negative",
            maker.materials_on_negative_side(&face).iter(),
            &inputs.arguments,
            rows,
        );
        list(
            "origins",
            maker.get_origins(&face).iter(),
            &inputs.sub_shapes,
            rows,
        );
        writeln!(rows).unwrap();
    }
    for (position, input) in inputs.sub_shapes.iter().enumerate() {
        if input.shape_type().unwrap() != ShapeEnum::Face {
            continue;
        }
        write!(rows, "input {}", position + 1).unwrap();
        list(
            "modified",
            maker.get_modified(input).unwrap().iter(),
            &all,
            rows,
        );
        let generated = maker.history().get().unwrap().generated(input).unwrap();
        list("generated", generated.iter(), &all, rows);
        writeln!(rows).unwrap();
    }
}

fn refused(label: &str, arguments: &[Shape], rows: &mut String) {
    let mut maker = BOPAlgoMakeConnected::new();
    maker.set_arguments(&arguments.iter().cloned().collect::<List<Shape>>());
    maker.options_mut().set_run_parallel(false);
    maker.perform().unwrap();
    let options = maker.options();
    let too_few = options.has_error(AlertTooFewArguments::static_type());
    let multi = options.has_error(AlertMultiDimensionalArguments::static_type());
    let errors = i32::from(options.has_errors());
    writeln!(
        rows,
        "refused {label} errors {errors} too-few {} multi-dim {}",
        i32::from(too_few),
        i32::from(multi)
    )
    .unwrap();
}

fn connected_case(arguments: &[Shape], rows: &mut String) {
    let inputs = Inputs::of(arguments);
    let mut maker = BOPAlgoMakeConnected::new();
    maker.set_arguments(&arguments.iter().cloned().collect::<List<Shape>>());
    maker.options_mut().set_run_parallel(false);
    maker.repeat_shape(0, 1).unwrap();
    writeln!(
        rows,
        "early repeat warnings {}",
        i32::from(maker.options().has_warnings())
    )
    .unwrap();
    maker.perform().unwrap();
    writeln!(
        rows,
        "glued errors {}",
        i32::from(maker.options().has_errors())
    )
    .unwrap();
    materials("glued", &maker, &maker.shape().clone(), &inputs, rows);
    let mut params = PeriodicityParams::new();
    params.periodic = [true; 3];
    params.period = [10.0; 3];
    maker.make_periodic(&params).unwrap();
    materials(
        "periodic",
        &maker,
        &maker.periodic_shape().clone(),
        &inputs,
        rows,
    );
    maker.repeat_shape(0, 1).unwrap();
    maker.repeat_shape(2, -1).unwrap();
    materials(
        "repeated",
        &maker,
        &maker.periodic_shape().clone(),
        &inputs,
        rows,
    );
    maker.clear_repetitions().unwrap();
    materials(
        "cleared",
        &maker,
        &maker.periodic_shape().clone(),
        &inputs,
        rows,
    );
}

fn single_case(box2: &Shape, rows: &mut String) {
    let mut one = BOPAlgoMakeConnected::new();
    one.set_arguments(&[box2.clone()].into_iter().collect());
    one.options_mut().set_run_parallel(false);
    one.perform().unwrap();
    let (errors, same) = (one.options().has_errors(), one.shape().is_same(box2));
    writeln!(
        rows,
        "single errors {} same {}",
        i32::from(errors),
        i32::from(same)
    )
    .unwrap();
    materials(
        "single",
        &one,
        &one.shape().clone(),
        &Inputs::of(std::slice::from_ref(box2)),
        rows,
    );
    one.clear().unwrap();
    let null = i32::from(one.shape().is_null());
    writeln!(
        rows,
        "cleared shape null {null} arguments {}",
        one.arguments().extent()
    )
    .unwrap();
}

#[test]
fn make_connected_matches_native_materials_and_history() {
    let box1 = profile();
    let box2 = box_at(0.0, 0.0, 5.0, (5.0, 5.0, 5.0));
    let box3 = box_at(0.0, 5.0, 0.0, (10.0, 5.0, 10.0));
    let mut rows = String::new();
    connected_case(&[box3.clone(), box1, box2.clone()], &mut rows);
    single_case(&box2, &mut rows);
    refused("empty", &[], &mut rows);
    refused(
        "mixed",
        &[box2, explore(&box3, ShapeEnum::Face)[0].clone()],
        &mut rows,
    );
    let expected = include_str!("../../../tools/oracle/tkbo_make_connected.expected");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
