//! Native replay of BOPAlgo_CellsBuilder; see tools/oracle/tkbo_cells_builder.cxx.
#[path = "support/native_snapshot.rs"]
mod native_snapshot;
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::bop_algo::BOPAlgoCellsBuilder;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::{self, Axis2, Circle3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use shape_dump::{dump_result, o};
use std::fmt::Write;

fn list(shapes: &[&Shape]) -> List<Shape> {
    shapes.iter().map(|shape| (*shape).clone()).collect()
}

fn cube(x: f64, y: f64, z: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), 10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn step(label: &str, cells: &BOPAlgoCellsBuilder, arguments: &List<Shape>, rows: &mut String) {
    let warnings = i32::from(cells.options().has_warnings());
    let kind = cells.shape().shape_type().unwrap() as i32;
    writeln!(rows, "step {label} warnings {warnings} type {kind}").unwrap();
    let all = dump_result(cells.shape(), rows);
    for argument in arguments.iter() {
        write!(
            rows,
            "history deleted {} modified:",
            i32::from(cells.is_deleted(argument).unwrap())
        )
        .unwrap();
        for modified in cells.modified(argument).unwrap() {
            write!(
                rows,
                " {}{}",
                all.find_index(&modified),
                o(modified.orientation())
            )
            .unwrap();
        }
        writeln!(rows).unwrap();
    }
}

fn performed(arguments: &List<Shape>, label: &str, rows: &mut String) -> BOPAlgoCellsBuilder {
    let mut cells = BOPAlgoCellsBuilder::new();
    cells.set_arguments(arguments);
    cells.options_mut().set_run_parallel(false);
    cells.perform();
    writeln!(
        rows,
        "case {label} errors {}",
        i32::from(cells.options().has_errors())
    )
    .unwrap();
    cells
}

fn solids(rows: &mut String) {
    let (b1, b2, b3) = (
        cube(0.0, 0.0, 0.0),
        cube(5.0, 0.0, 0.0),
        cube(2.5, 0.0, 5.0),
    );
    let args = list(&[&b1, &b2, &b3]);
    let none = List::default();
    let mut cells = performed(&args, "solids", rows);
    dump_result(cells.all_parts(), rows);
    step("empty", &cells, &args, rows);
    cells.add_to_result(&list(&[&b1]), &none, 1, false).unwrap();
    step("b1", &cells, &args, rows);
    cells.remove_internal_boundaries().unwrap();
    step("b1-rib", &cells, &args, rows);
    cells.remove_all_from_result().unwrap();
    cells
        .add_to_result(&list(&[&b1]), &list(&[&b3]), 1, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&b2]), &list(&[&b3]), 1, false)
        .unwrap();
    step("b1b2-not-b3", &cells, &args, rows);
    cells.remove_internal_boundaries().unwrap();
    step("b1b2-not-b3-rib", &cells, &args, rows);
    cells.remove_all_from_result().unwrap();
    cells.add_to_result(&list(&[&b1]), &none, 1, false).unwrap();
    cells
        .add_to_result(&list(&[&b2]), &list(&[&b1]), 2, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&b3]), &list(&[&b1]), 2, true)
        .unwrap();
    step("two-materials-updated", &cells, &args, rows);
    cells.remove_from_result(&list(&[&b2, &b3]), &none).unwrap();
    step("removed-b2b3", &cells, &args, rows);
    cells.add_all_to_result(0, false).unwrap();
    cells.make_containers().unwrap();
    step("all-containers", &cells, &args, rows);
    cells
        .remove_from_result(&list(&[&b1]), &list(&[&b2]))
        .unwrap();
    step("containers-removed", &cells, &args, rows);
    cells.add_all_to_result(3, true).unwrap();
    step("all-material", &cells, &args, rows);
}

fn square(low: f64, high: f64) -> Shape {
    let maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (low, high, low, high))
            .unwrap();
    maker.face().unwrap().into_shape()
}

fn faces(rows: &mut String) {
    let (f1, f2) = (square(0.0, 10.0), square(5.0, 15.0));
    let args = list(&[&f1, &f2]);
    let none = List::default();
    let mut cells = performed(&args, "faces", rows);
    cells
        .add_to_result(&list(&[&f1, &f2]), &none, 0, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&f1]), &list(&[&f2]), 0, false)
        .unwrap();
    step("common-and-f1", &cells, &args, rows);
    cells.make_containers().unwrap();
    step("shell", &cells, &args, rows);
    cells.remove_from_result(&list(&[&f1, &f2]), &none).unwrap();
    step("shell-without-common", &cells, &args, rows);
}

fn mixed(rows: &mut String) {
    let solid = cube(0.0, 0.0, 0.0);
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(5.0, 5.0, -5.0),
        Point3::new(5.0, 5.0, 15.0),
    );
    let edge = edge.unwrap().edge().unwrap().into_shape();
    let args = list(&[&solid, &edge]);
    let none = List::default();
    let mut cells = performed(&args, "mixed", rows);
    cells
        .add_to_result(&list(&[&edge, &solid]), &none, 0, false)
        .unwrap();
    step("edge-in-box", &cells, &args, rows);
    cells
        .add_to_result(&list(&[&edge]), &list(&[&solid]), 0, false)
        .unwrap();
    step("edge-outside", &cells, &args, rows);
    cells
        .add_to_result(&list(&[&solid]), &none, 1, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&edge]), &list(&[&solid]), 1, true)
        .unwrap();
    step("mixed-material", &cells, &args, rows);
}

fn circle_face(center: Point3) -> Shape {
    let frame = Axis2::new(center, gp::dz()).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, 10.0).unwrap()).unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge.edge().unwrap())
        .unwrap()
        .wire()
        .unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, false).unwrap();
    face.face().unwrap().into_shape()
}

/// Faces of one material unified, keeping the material boundaries (history A5).
fn unified_faces(rows: &mut String) {
    let f1 = circle_face(Point3::new(0.0, 0.0, 0.0));
    let f2 = circle_face(Point3::new(10.0, 0.0, 0.0));
    let f3 = circle_face(Point3::new(10.0, 10.0, 0.0));
    let args = list(&[&f1, &f2, &f3]);
    let none = List::default();
    let mut cells = performed(&args, "unified-faces", rows);
    cells.add_to_result(&list(&[&f1]), &none, 1, false).unwrap();
    cells.remove_internal_boundaries().unwrap();
    step("f1-rib", &cells, &args, rows);
    cells.remove_all_from_result().unwrap();
    cells
        .add_to_result(&list(&[&f1]), &list(&[&f3]), 1, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&f2]), &list(&[&f3]), 1, false)
        .unwrap();
    cells.remove_internal_boundaries().unwrap();
    step("f1f2-not-f3-rib", &cells, &args, rows);
    cells.remove_all_from_result().unwrap();
    cells.add_to_result(&list(&[&f1]), &none, 1, false).unwrap();
    cells
        .add_to_result(&list(&[&f2]), &list(&[&f1]), 2, false)
        .unwrap();
    cells
        .add_to_result(&list(&[&f3]), &list(&[&f1]), 2, false)
        .unwrap();
    cells.remove_internal_boundaries().unwrap();
    step("two-materials-rib", &cells, &args, rows);
    cells.make_containers().unwrap();
    step("two-materials-containers", &cells, &args, rows);
}

fn segment(from: (f64, f64), to: (f64, f64)) -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(from.0, from.1, 0.0),
        Point3::new(to.0, to.1, 0.0),
    );
    edge.unwrap().edge().unwrap().into_shape()
}

/// Edges of one material unified, a crossing edge of another material kept apart.
fn unified_edges(rows: &mut String) {
    let e1 = segment((0.0, 0.0), (10.0, 0.0));
    let e2 = segment((5.0, 0.0), (15.0, 0.0));
    let e3 = segment((7.0, -5.0), (7.0, 5.0));
    let args = list(&[&e1, &e2, &e3]);
    let none = List::default();
    let mut cells = performed(&args, "unified-edges", rows);
    cells.add_to_result(&list(&[&e1]), &none, 1, false).unwrap();
    cells.add_to_result(&list(&[&e2]), &none, 1, false).unwrap();
    step("e1e2", &cells, &args, rows);
    cells.remove_internal_boundaries().unwrap();
    step("e1e2-rib", &cells, &args, rows);
    cells.add_to_result(&list(&[&e3]), &none, 2, false).unwrap();
    cells.remove_internal_boundaries().unwrap();
    step("e3-rib", &cells, &args, rows);
}

#[test]
fn cells_builder_matches_native_steps() {
    let mut rows = String::new();
    solids(&mut rows);
    faces(&mut rows);
    mixed(&mut rows);
    unified_faces(&mut rows);
    unified_edges(&mut rows);
    // Native OCCT run 38123916241 pins each host's exact arithmetic.
    let expected = native_snapshot::native_boolean_snapshot!("tkbo_cells_builder");
    for (line, (actual, wanted)) in rows.lines().zip(expected.lines()).enumerate() {
        assert_eq!(actual, wanted, "line {}", line + 1);
    }
    assert_eq!(rows.lines().count(), expected.lines().count());
}
