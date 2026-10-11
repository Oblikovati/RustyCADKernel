//! Native replay of a fuse of a box with a trimmed torus ring crossing its seams; see
//! tools/oracle/tkbo_bop_torus.cxx.
#[path = "shape_dump/mod.rs"]
mod shape_dump;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeRevol};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use shape_dump::dump_result;
use std::f64::consts::PI;
use std::fmt::Write;

#[test]
fn box_torus_fuse_matches_native_oracle() {
    let box_shape = BRepPrimAPIMakeBox::from_points(
        Point3::new(-187.0, -20.0, -67.5),
        Point3::new(187.0, 0.0, 67.5),
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone();
    let frame = Axis2::new(
        Point3::new(200.0, -20.0, 0.0),
        Direction3::new(0.0, -1.0, 0.0).unwrap(),
    )
    .unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, 17.5).unwrap())
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    let profile = BRepBuilderAPIMakeFace::from_wire(&wire, true)
        .unwrap()
        .face()
        .unwrap();
    let axis = Axis1::new(
        Point3::new(0.0, -20.0, 0.0),
        Direction3::new(0.0, 0.0, -1.0).unwrap(),
    );
    let ring = BRepPrimAPIMakeRevol::new(&profile, axis, 2.0 * PI, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let fuse = BRepAlgoAPIFuse::from_shapes(&box_shape, &ring).unwrap();
    let mut rows = String::new();
    writeln!(
        rows,
        "done {} warnings {}",
        i32::from(fuse.is_done()),
        i32::from(fuse.has_warnings())
    )
    .unwrap();
    dump_result(fuse.shape(), &mut rows);
    // Run 38119937683: pinned OCCT compiled with -ffp-contract=off agrees bit for bit.
    let expected = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        include_str!("../../../tools/oracle/tkbo_bop_torus.macos.expected")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        include_str!("../../../tools/oracle/tkbo_bop_torus.windows.expected")
    } else {
        include_str!("../../../tools/oracle/tkbo_bop_torus.expected")
    };
    // Native Windows CRT streams use CRLF; every numeric and topology field stays exact.
    let native_snapshot_rows = expected.replace("\r\n", "\n");
    for (line, (actual, wanted)) in rows.lines().zip(native_snapshot_rows.lines()).enumerate() {
        assert_eq!(actual, wanted, "first difference at line {}", line + 1);
    }
    assert_eq!(rows, native_snapshot_rows);
}
