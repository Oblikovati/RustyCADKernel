//! Behavior tests of `BRepBndLib::AddOBB`. The upstream case that calls it,
//! `Bnd_OBB_Test.OCC30704_AddBoundingBoxToVoidBox` of the TKMath suite, builds its box with
//! BRepPrimAPI_MakeBox (TKPrim), which the port does not provide; here the box [0, 10]^3
//! is built with BRep_Builder by the probe's shape reader and the same property is checked
//! (the oriented box of the box, added to a void box, is centred on the box). The other
//! cases check the two paths of AddOBB: DiTO over the vertices of a polyhedral shape, and
//! the axis-aligned box in the frame of inertia of a curved shape without mesh.
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape.rs"]
mod shape;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape_geometry.rs"]
mod shape_geometry;
mod solid_records;

use print::Reader;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkmath::bnd::OrientedBoundingBox;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use shape::tp_shape;
use solid_records::{BOX10, SPHERE10};

fn solid(record: &[f64]) -> Shape {
    tp_shape(&mut Reader::new(record))
        .expect("shape record")
        .shape
}

fn obb_of(record: &[f64], flags: (bool, bool, bool)) -> OrientedBoundingBox {
    let mut obb = OrientedBoundingBox::default();
    BRepBndLib::add_obb(&solid(record), &mut obb, flags.0, flags.1, flags.2).unwrap();
    obb
}

fn assert_close(value: f64, expected: f64, tolerance: f64) {
    assert!(
        (value - expected).abs() <= tolerance,
        "{value} != {expected}"
    );
}

// After Bnd_OBB_Test.OCC30704_AddBoundingBoxToVoidBox.
#[test]
fn obb_of_a_box_added_to_a_void_box_is_centred_on_the_box() {
    let obb = obb_of(&BOX10, (false, false, false));
    let mut void_box = OrientedBoundingBox::default();
    void_box.add_box(&obb).unwrap();
    let center = void_box.center();
    assert_close(center.x(), 5.0, 1.0e-12);
    assert_close(center.y(), 5.0, 1.0e-12);
    assert_close(center.z(), 5.0, 1.0e-12);
}

#[test]
fn obb_of_a_box_encloses_its_corners_with_the_vertex_tolerance() {
    let obb = obb_of(&BOX10, (true, false, true));
    let sizes = obb.half_sizes();
    for size in [sizes.x, sizes.y, sizes.z] {
        assert_close(size, 5.0 + 1.0e-7, 1.0e-9);
    }
    assert!(obb.contains(Point3::new(10.0, 10.0, 10.0)));
    assert!(obb.contains(Point3::new(0.0, 0.0, 0.0)));
}

#[test]
fn obb_of_a_sphere_without_mesh_is_its_box_in_the_frame_of_inertia() {
    // The sphere face is not planar and has no triangulation, so AddOBB takes the PCA
    // path: the Add box of the sphere of radius 10 (face tolerance 1e-7 as gap).
    let obb = obb_of(&SPHERE10, (true, false, true));
    assert!(!obb.is_void());
    let sizes = obb.half_sizes();
    for size in [sizes.x, sizes.y, sizes.z] {
        assert_close(size, 10.0, 1.0e-5);
    }
    let center = obb.center();
    for coordinate in [center.x(), center.y(), center.z()] {
        assert_close(coordinate, 0.0, 1.0e-9);
    }
}

#[test]
fn obb_of_an_empty_shape_leaves_the_box_void() {
    let mut obb = OrientedBoundingBox::default();
    BRepBndLib::add_obb(&Compound::default().into(), &mut obb, true, false, true).unwrap();
    assert!(obb.is_void());
}

#[test]
fn obb_added_to_a_box_encloses_both() {
    let mut obb = OrientedBoundingBox::new(
        Point3::new(20.0, 0.0, 0.0),
        [Direction3::X, Direction3::Y, Direction3::Z],
        1.0,
        1.0,
        1.0,
    );
    BRepBndLib::add_obb(&solid(&BOX10), &mut obb, true, false, false).unwrap();
    for corner in [
        [0.0, 0.0, 0.0],
        [10.0, 10.0, 10.0],
        [21.0, 1.0, 1.0],
        [19.0, -1.0, -1.0],
    ] {
        let point = Point3::new(corner[0], corner[1], corner[2]);
        assert!(!obb.is_out_point(point), "{corner:?}");
    }
}
