//! Native replay of BRepAlgoAPI_Defeaturing / BOPAlgo_RemoveFeatures; see
//! tools/oracle/tkbo_defeaturing.cxx.
#[path = "support/native_snapshot.rs"]
mod native_snapshot;
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::{
    AlertNoFacesToRemove, AlertTooFewArguments, AlertUnableToRemoveTheFeature, AlertUnsupportedType,
};
use rustycad_tkbo::brep_algo_api::{
    BRepAlgoAPIBooleanOperation, BRepAlgoAPICut, BRepAlgoAPIDefeaturing, BRepAlgoAPIFuse,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::RuntimeTyped;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Axis2, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
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

fn cube(corner: (f64, f64, f64), size: (f64, f64, f64)) -> Shape {
    let corner = Point3::new(corner.0, corner.1, corner.2);
    BRepPrimAPIMakeBox::at_point(corner, size.0, size.1, size.2)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn cylinder(x: f64, y: f64, z: f64, radius: f64, height: f64) -> Shape {
    let axes = Axis2::new(Point3::new(x, y, z), gp::dz()).unwrap();
    BRepPrimAPIMakeCylinder::with_axes(axes, radius, height)
        .shape()
        .unwrap()
        .clone()
}

fn plane_face(z: f64) -> Shape {
    let plane = Plane3::new(Point3::new(0.0, 0.0, z), gp::dz()).unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn i(flag: bool) -> i32 {
    i32::from(flag)
}

fn status(label: &str, api: &BRepAlgoAPIDefeaturing, rows: &mut String) {
    let errors = (
        api.has_error(AlertNoFacesToRemove::static_type()),
        api.has_error(AlertTooFewArguments::static_type()),
    );
    let unsupported = api.has_warning(AlertUnsupportedType::static_type());
    let unable = api.has_warning(AlertUnableToRemoveTheFeature::static_type());
    writeln!(
        rows,
        "case {label} done {} errors {} warnings {} no-faces {} too-few {} unsupported {} unable {}",
        i(api.is_done()),
        i(api.has_errors()),
        i(api.has_warnings()),
        i(errors.0),
        i(errors.1),
        i(unsupported),
        i(unable)
    )
    .unwrap();
}

fn history_rows(
    api: &BRepAlgoAPIDefeaturing,
    input: &Shape,
    all: &IndexedMapOfShape,
    rows: &mut String,
) {
    let flags = [
        api.has_history(),
        api.has_modified(),
        api.has_generated(),
        api.has_deleted(),
    ]
    .map(i);
    writeln!(
        rows,
        "history {} modified {} generated {} deleted {}",
        flags[0], flags[1], flags[2], flags[3]
    )
    .unwrap();
    let mut inputs = IndexedMapOfShape::default();
    topexp::map_shapes(input, &mut inputs, true, true);
    for (position, shape) in inputs.iter().enumerate() {
        if !matches!(
            shape.shape_type().unwrap(),
            ShapeEnum::Face | ShapeEnum::Edge | ShapeEnum::Solid
        ) {
            continue;
        }
        write!(
            rows,
            "input {} deleted {}",
            position + 1,
            i(api.is_deleted(shape).unwrap())
        )
        .unwrap();
        list("modified", api.modified(shape).unwrap().iter(), all, rows);
        list("generated", api.generated(shape).unwrap().iter(), all, rows);
        writeln!(rows).unwrap();
    }
}

fn run(label: &str, input: &Shape, faces: &[Shape], history: bool, rows: &mut String) {
    let mut api = BRepAlgoAPIDefeaturing::new();
    api.set_shape(input);
    api.add_faces_to_remove(&faces.iter().cloned().collect::<List<Shape>>());
    api.set_to_fill_history(history);
    api.set_run_parallel(false);
    api.build();
    status(label, &api, rows);
    if !api.is_done() {
        return;
    }
    let all = dump_result(api.shape(), rows);
    history_rows(&api, input, &all, rows);
}

/// The faces of the operation result coming from a face of `tool`.
fn faces_from(operation: &BRepAlgoAPIBooleanOperation, tool: &Shape) -> Vec<Shape> {
    let mut result = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(operation.shape(), ShapeEnum::Face, &mut result).unwrap();
    let mut faces = Vec::new();
    let mut explorer = Explorer::new(tool, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        let face = explorer.current();
        faces.extend(
            operation
                .modified(face)
                .unwrap()
                .into_iter()
                .filter(|image| result.contains(image)),
        );
        if result.contains(face) {
            faces.push(face.clone());
        }
        explorer.next().unwrap();
    }
    faces
}

fn compound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut result = compound.into_shape();
    for shape in shapes {
        builder.add(&mut result, shape).unwrap();
    }
    result
}

fn feature_cases(rows: &mut String) {
    let base = cube((0.0, 0.0, 0.0), (10.0, 10.0, 5.0));
    let boss = cylinder(5.0, 5.0, 5.0, 2.0, 3.0);
    let fuse = BRepAlgoAPIFuse::from_shapes(&base, &boss).unwrap();
    run("boss", fuse.shape(), &faces_from(&fuse, &boss), true, rows);
    let drill = cylinder(5.0, 5.0, -1.0, 2.0, 7.0);
    let hole = BRepAlgoAPICut::from_shapes(&base, &drill).unwrap();
    run("hole", hole.shape(), &faces_from(&hole, &drill), true, rows);
    run(
        "hole-no-history",
        hole.shape(),
        &faces_from(&hole, &drill),
        false,
        rows,
    );
    let pocket_tool = cube((3.0, 3.0, 3.0), (4.0, 4.0, 3.0));
    let pocket = BRepAlgoAPICut::from_shapes(&base, &pocket_tool).unwrap();
    run(
        "pocket",
        pocket.shape(),
        &faces_from(&pocket, &pocket_tool),
        true,
        rows,
    );
    let drill2 = cylinder(2.0, 2.0, -1.0, 1.0, 7.0);
    let both = BRepAlgoAPICut::from_shapes(fuse.shape(), &drill2).unwrap();
    let mut both_faces = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(both.shape(), ShapeEnum::Face, &mut both_faces).unwrap();
    let mut two_features = faces_from(&both, &drill2);
    two_features.extend(
        faces_from(&fuse, &boss)
            .into_iter()
            .filter(|face| both_faces.contains(face)),
    );
    run("two-features", both.shape(), &two_features, true, rows);
    let mixed = compound(&[hole.shape().clone(), plane_face(20.0)]);
    run(
        "compound-with-face",
        &mixed,
        &faces_from(&hole, &drill),
        true,
        rows,
    );
    more_cases(&base, &hole, &drill, rows);
    refused_cases(&base, rows);
}

/// A through slot, a compound of two solids of which one has the feature, and a box
/// fused on a cylinder (a curved adjacent face to extend).
fn more_cases(base: &Shape, hole: &BRepAlgoAPICut, drill: &Shape, rows: &mut String) {
    let slot_tool = cube((4.0, -1.0, 2.0), (2.0, 12.0, 4.0));
    let slot = BRepAlgoAPICut::from_shapes(base, &slot_tool).unwrap();
    run(
        "slot",
        slot.shape(),
        &faces_from(&slot, &slot_tool),
        true,
        rows,
    );
    let mut solids = Vec::new();
    let mut explorer = Explorer::new(hole.shape(), ShapeEnum::Solid, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        solids.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    solids.push(cube((20.0, 0.0, 0.0), (5.0, 5.0, 5.0)));
    run(
        "two-solids",
        &compound(&solids),
        &faces_from(hole, drill),
        true,
        rows,
    );
    let drum = cylinder(0.0, 0.0, 0.0, 10.0, 10.0);
    let knob = cube((8.0, -2.0, 3.0), (6.0, 4.0, 4.0));
    let knobbed = BRepAlgoAPIFuse::from_shapes(&drum, &knob).unwrap();
    run(
        "box-on-cylinder",
        knobbed.shape(),
        &faces_from(&knobbed, &knob),
        true,
        rows,
    );
}

fn refused_cases(base: &Shape, rows: &mut String) {
    let outside = [plane_face(0.0)];
    run("outside-face", base, &outside, true, rows);
    run("face-input", &plane_face(0.0), &outside, true, rows);
    let mut explorer = Explorer::new(base, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut faces = Vec::new();
    while explorer.more() {
        faces.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    run("five-box-faces", base, &faces[1..], true, rows);
}

#[test]
fn defeaturing_matches_native_results_and_history() {
    let mut rows = String::new();
    feature_cases(&mut rows);
    // Native OCCT run 38123916241 pins each host's exact arithmetic.
    let expected = native_snapshot::native_boolean_snapshot!("tkbo_defeaturing");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
