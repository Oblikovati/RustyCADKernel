//! Behavior of the IntPatch polygons, bit sets and polyhedron interference beyond the
//! numeric probe, which compares them with native OCCT bit for bit.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchInterferencePolyhedron, IntPatchPolyArc, IntPatchPolyLine, IntPatchPolygo,
    IntPatchPolyhedron, IntPatchPrmPrmIntersectionT3Bits, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkgeomalgo::intf::Polygon2d;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};

/// A walking line of four points on the `u` axis of the first surface and on `v = 1` of
/// the second one.
fn straight_wline() -> Handle<IntPatchWLine> {
    let line = IntSurfLineOn2S::new(None);
    for i in 0..4 {
        let x = 0.1 * f64::from(i);
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, 0.0, 0.0), x, 0.0, x, 1.0);
        line.add(&p);
    }
    Handle::new(IntPatchWLine::new_undecided(&Handle::new(line), false))
}

/// The vertical line `u = 1` of a parametric space over `v` in `[0, 1]`.
fn vertical_arc() -> TheArc {
    let dir = Direction2::new(0.0, 1.0).unwrap();
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(1.0, 0.0), dir)).cast();
    Handle::new(Geom2dAdaptorCurve::with_range(&line, 0.0, 1.0).unwrap()).cast()
}

fn other_box() -> BoundingBox2 {
    let mut b = BoundingBox2::default();
    b.update_bounds([-10.0, -10.0], [10.0, 10.0]);
    b
}

#[test]
fn polyline_follows_the_side_of_its_walking_line() {
    let line = straight_wline();
    let mut first = IntPatchPolyLine::new();
    first.set_w_line(true, &line).unwrap();
    assert_eq!(first.nb_points().unwrap(), 4);
    assert_eq!(first.nb_segments(), 3);
    assert_eq!(first.point(2).unwrap(), Point2::new(0.1, 0.0));
    let mut second = IntPatchPolyLine::new();
    second.set_w_line(false, &line).unwrap();
    assert_eq!(second.point(4).unwrap().y(), 1.0);
    assert!(second.point(5).is_err());
}

#[test]
fn polyline_without_a_line_has_no_points() {
    let mut polygon = IntPatchPolyLine::with_deflection(1e-3);
    assert_eq!(polygon.error(), 1e-3);
    assert!(polygon.nb_points().is_err());
    polygon.reset_error();
    assert_eq!(polygon.error(), IntPatchPolyLine::new().error());
}

#[test]
fn polyarc_samples_the_arc_and_moves_by_the_offset() {
    let mut polygon = IntPatchPolyArc::new(&vertical_arc(), 5, 0.0, 1.0, &other_box()).unwrap();
    assert_eq!(polygon.nb_points().unwrap(), 5);
    assert_eq!(polygon.parameter(1).unwrap(), 0.0);
    assert_eq!(polygon.parameter(5).unwrap(), 1.0);
    assert!(!polygon.is_closed());
    assert_eq!(polygon.point(5).unwrap(), Point2::new(1.0, 1.0));
    polygon.set_offset(0.5, -1.0).unwrap();
    assert_eq!(polygon.point(5).unwrap(), Point2::new(1.5, 0.0));
    assert!(polygon.parameter(6).is_err());
}

#[test]
fn polyarc_refuses_too_few_samples_and_infinite_bounds() {
    let arc = vertical_arc();
    assert!(IntPatchPolyArc::new(&arc, 1, 0.0, 1.0, &other_box()).is_err());
    assert!(IntPatchPolyArc::new(&arc, 5, f64::MIN, 1.0, &other_box()).is_err());
}

#[test]
fn t3bits_and_finds_the_common_bits_and_clears_them() {
    let (mut a, mut b) = (
        IntPatchPrmPrmIntersectionT3Bits::new(4),
        IntPatchPrmPrmIntersectionT3Bits::new(4),
    );
    a.add(5);
    a.add(40);
    b.add(40);
    let mut index = 0;
    assert_eq!(a.and(&mut b, &mut index), 1);
    assert_eq!(index, 40);
    assert_eq!(a.val(5), 1 << 5);
    a.raz(5);
    assert_eq!(a.val(5), 0);
}

#[test]
fn perform_on_one_polyhedron_finds_nothing_unlike_its_constructor() {
    let axis = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(axis.unwrap(), 1.0).unwrap()).cast();
    let adaptor = Handle::new(GeomAdaptorSurface::with_surface(&sphere).unwrap()).cast();
    let polyhedron = IntPatchPolyhedron::with_subdivisions(&adaptor, 6, 6).unwrap();
    let mut interference = IntPatchInterferencePolyhedron::new();
    interference.perform_self(&polyhedron);
    assert_eq!(interference.nb_section_points(), 0);
    assert_eq!(interference.nb_section_lines(), 0);
    assert_eq!(interference.nb_tangent_zones(), 0);
    IntPatchInterferencePolyhedron::of_self(&polyhedron).unwrap();
}
