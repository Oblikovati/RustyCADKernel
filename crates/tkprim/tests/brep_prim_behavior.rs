//! Behaviour tests of the BRepPrim package: the topology of the primitives, their
//! errors, and the native quirks docs/tkprim-progress.md lists.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkprim::brep_prim::BRepPrimDirection::{XMax, XMin, YMax, ZMax, ZMin};
use rustycad_tkprim::brep_prim::{
    BRepPrimCone, BRepPrimCylinder, BRepPrimDirection, BRepPrimOneAxis, BRepPrimSphere,
    BRepPrimTorus, BRepPrimWedge,
};
use std::f64::consts::PI;

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut n = 0;
    while explorer.more() {
        n += 1;
        explorer.next().unwrap();
    }
    n
}

#[test]
fn a_box_shell_has_six_faces_and_is_closed() {
    let mut wedge = BRepPrimWedge::with_box(Axis2::default(), 1.0, 2.0, 3.0);
    let shell = wedge.shell().unwrap();
    assert_eq!(count(&shell, ShapeEnum::Face), 6);
    assert_eq!(count(&shell, ShapeEnum::Edge), 24);
    assert!(shell.closed().unwrap());
}

#[test]
fn a_right_angular_wedge_with_ltx_zero_has_five_faces() {
    let mut wedge = BRepPrimWedge::with_ltx(Axis2::default(), 10.0, 10.0, 10.0, 0.0);
    assert!(!wedge.has_face(YMax));
    assert_eq!(count(&wedge.shell().unwrap(), ShapeEnum::Face), 5);
}

#[test]
fn a_degenerated_wedge_has_no_shell() {
    let mut wedge = BRepPrimWedge::new();
    assert!(wedge.is_degenerated_shape());
    let error = wedge.shell().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
}

#[test]
fn a_wedge_cannot_be_opened_once_built() {
    let mut wedge = BRepPrimWedge::with_box(Axis2::default(), 1.0, 1.0, 1.0);
    wedge.open(ZMax).unwrap();
    assert!(!wedge.has_face(ZMax));
    wedge.face(XMin).unwrap();
    assert_eq!(
        wedge.close(ZMax).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn directions_of_one_axis_have_no_edge() {
    let wedge = BRepPrimWedge::with_box(Axis2::default(), 1.0, 1.0, 1.0);
    let error = wedge.has_edge(XMin, XMax).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(wedge.has_edge(XMin, BRepPrimDirection::YMin).unwrap());
}

#[test]
fn a_segment_ymax_face_can_share_a_vertex_not_built_yet() {
    // Z2 is flat and X2 is not: building the vertex XMax ZMin YMax (6) first copies
    // the unbuilt vertex 2 into slot 3, so the vertex XMin ZMax YMax comes back null.
    let mut wedge = BRepPrimWedge::with_limits(
        Axis2::default(),
        [0.0, 0.0, 0.0, 2.0, 3.0],
        [4.0, 5.0, 6.0, 2.0, 7.0],
    );
    assert!(!wedge.vertex(XMax, ZMin, YMax).unwrap().is_null());
    assert!(wedge.vertex(XMin, ZMax, YMax).unwrap().is_null());
}

#[test]
fn a_cylinder_with_sides_has_five_faces() {
    let mut cylinder = BRepPrimCylinder::from_radius_height(5.0, 10.0);
    cylinder.set_angle(PI).unwrap();
    assert!(cylinder.has_sides());
    assert_eq!(count(&cylinder.shell().unwrap(), ShapeEnum::Face), 5);
    assert_eq!(
        cylinder.set_angle(1.0).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn an_infinite_cylinder_has_only_its_lateral_face() {
    let mut cylinder = BRepPrimCylinder::infinite(1.0);
    assert!(cylinder.vmin_infinite() && cylinder.vmax_infinite());
    assert!(!cylinder.has_top().unwrap());
    assert_eq!(count(&cylinder.shell().unwrap(), ShapeEnum::Face), 1);
}

#[test]
fn a_pointed_cone_has_no_top() {
    let mut cone = BRepPrimCone::from_radii(5.0, 0.0, 10.0).unwrap();
    assert!(!cone.has_top().unwrap());
    assert!(cone.has_bottom().unwrap());
    assert_eq!(count(&cone.shell().unwrap(), ShapeEnum::Face), 2);
}

#[test]
fn the_cone_rejects_equal_radii_and_null_heights() {
    let equal = BRepPrimCone::from_radii(2.0, 2.0, 1.0).err().unwrap();
    assert_eq!(equal.exception_type(), "Standard_DomainError");
    let flat = BRepPrimCone::from_radii(1.0, 2.0, 0.0).err().unwrap();
    assert_eq!(flat.exception_type(), "Standard_DomainError");
    let wide = BRepPrimCone::infinite(2.0).err().unwrap();
    assert_eq!(wide.exception_type(), "Standard_DomainError");
}

#[test]
fn the_cone_on_axes_without_radius_starts_at_the_apex() {
    // Native leaves the radius uninitialized; the port uses 0.
    let cone = BRepPrimCone::infinite_with_axes(0.5, Axis2::default()).unwrap();
    assert_eq!(cone.meridian_value(0.0).unwrap().x(), 0.0);
}

#[test]
fn a_sphere_offsets_its_meridian_edges() {
    let mut sphere = BRepPrimSphere::new(5.0).unwrap();
    assert!(!sphere.has_top().unwrap() && !sphere.has_bottom().unwrap());
    let shell = sphere.shell().unwrap();
    assert_eq!(count(&shell, ShapeEnum::Face), 1);
    let edge = sphere.start_edge().unwrap();
    let vertex_count = count(&edge, ShapeEnum::Vertex);
    assert_eq!(vertex_count, 2);
}

#[test]
fn a_full_torus_has_one_face_with_seams() {
    let mut torus = BRepPrimTorus::at_center(Point3::new(1.0, 2.0, 3.0), 10.0, 2.0).unwrap();
    assert!(torus.meridian_closed().unwrap());
    assert_eq!(count(&torus.shell().unwrap(), ShapeEnum::Face), 1);
}

#[test]
fn the_lateral_start_wire_shares_the_lateral_wire_slot() {
    let mut cylinder = BRepPrimCylinder::from_radius_height(1.0, 2.0);
    let lateral = cylinder.lateral_wire().unwrap();
    let start = cylinder.lateral_start_wire().unwrap();
    assert!(start.is_same(&lateral));
    assert_eq!(count(&start, ShapeEnum::Edge), 4);
}

#[test]
fn the_start_wire_alone_breaks_a_later_lateral_face() {
    // The lateral face then reuses the one-edge start wire and sets a p-curve on the top
    // edge, never built: natively a crash, an error here.
    let mut cylinder = BRepPrimCylinder::from_radius_height(1.0, 2.0);
    assert_eq!(
        count(&cylinder.lateral_start_wire().unwrap(), ShapeEnum::Edge),
        1
    );
    assert!(cylinder.lateral_face().is_err());
}
