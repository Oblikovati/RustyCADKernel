//! Native p-curve range repair behavior for `ShapeFix_Wire::FixGap2d(..., false)`.
#[path = "support/wire_gap2d_cubic.rs"]
mod cubic;
#[path = "support/wire_gap2d_ranges.rs"]
mod support;
use rustycad_tkbrep::{brep::BRepTool, topods};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixWire};
use support::*;

#[test]
fn intersecting_line_pcurves_extend_only_to_the_shared_parameter() {
    let (face, wire) = crossing_line_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert!(!tool.context().is_null());
    assert_line_observation(&tool, &face, 1, [0.0, 0.8, 0.0, 0.0, 0.8, 0.0]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [-0.282842712474619, 2.0_f64.sqrt(), 0.8, 0.0, 2.0, 1.2],
    );
}

#[test]
fn parallel_line_pcurves_repair_their_nearest_endpoints() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.1, 0.2, 0.0), Point3::new(2.1, 0.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.0, 1.05, 0.0, 0.0, 1.05, 0.0]);
    assert_line_observation(&tool, &face, 2, [-0.05, 1.0, 1.05, 0.2, 2.1, 0.2]);
}

#[test]
fn reversed_first_line_uses_the_oriented_range_end() {
    let mut first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    first.reverse();
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.8, 1.0, 0.8, 0.0, 1.0, 0.0]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [-0.282842712474619, 2.0_f64.sqrt(), 0.8, 0.0, 2.0, 1.2],
    );
}

#[test]
fn finite_surface_keeps_a_nearest_hit_when_boundary_lies_beyond_gap_segment() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.1, 0.2, 0.0), Point3::new(2.1, 0.2, 0.0));
    let surface: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(
            &Handle::new(GeomPlane::new(Axis3::default())).cast(),
            (0.0, 1.0),
            (0.0, 1.0),
            (true, true),
        )
        .unwrap(),
    )
    .cast();
    let (face, wire) = wire_with_surface(&first, &second, &surface);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.0, 1.0, 0.0, 0.0, 1.0, 0.0]);
    assert_line_observation(&tool, &face, 2, [-0.05, 1.0, 1.05, 0.2, 2.1, 0.2]);
}

#[test]
fn surface_boundary_clips_an_intersection_beyond_the_trimmed_domain() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(0.8, 0.0, 0.0));
    let second = edge_between(Point3::new(0.8, 0.4, 0.0), Point3::new(1.8, -0.6, 0.0));
    let base: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let surface: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&base, (0.0, 1.0), (0.0, 1.0), (true, true)).unwrap(),
    )
    .cast();
    let (face, wire) = wire_with_surface(&first, &second, &surface);
    let builder = rustycad_tkbrep::brep::BRepBuilder::default();
    let first_curve: Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve> = Handle::new(
        rustycad_tkg2d::geom2d::Geom2dLine::new(rustycad_tkmath::gp::Axis2d::new(
            rustycad_tkmath::gp::Point2::new(0.0, 0.0),
            rustycad_tkmath::gp::Direction2::new(1.0, 0.0).unwrap(),
        )),
    )
    .cast();
    let second_curve: Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve> = Handle::new(
        rustycad_tkg2d::geom2d::Geom2dLine::new(rustycad_tkmath::gp::Axis2d::new(
            rustycad_tkmath::gp::Point2::new(0.8, 0.4),
            rustycad_tkmath::gp::Direction2::new(1.0, -1.0).unwrap(),
        )),
    )
    .cast();
    set_pcurve_range(&builder, &first, &face, &first_curve, 0.8);
    set_pcurve_range(&builder, &second, &face, &second_curve, 2.0_f64.sqrt());
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(&tool, &face, 1, [0.0, 1.0, 0.0, 0.0, 1.0, 0.0]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [0.5656854249492381, 2.0_f64.sqrt(), 1.2, 0.0, 1.8, -0.6],
    );
}

#[test]
fn one_edge_circular_pcurve_expands_both_parameter_ends() {
    let (face, wire) = one_circular_arc_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(1, false).unwrap());
    let result_wire = tool.wire().unwrap();
    let result_shape = result_wire.into_shape();
    let result_iter = rustycad_tkbrep::topods::ShapeIterator::new(&result_shape, false, false);
    let result_edge = result_iter.value().unwrap();
    let result_edge = topods::edge(result_edge).unwrap();
    let result_curve = BRepTool::curve_on_face(&result_edge, &face).unwrap();
    assert!((result_curve.first + 2.641592653589793).abs() < 1e-12);
    assert!((result_curve.last - 3.641592653589793).abs() < 1e-12);
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            -2.641592653589793,
            3.641592653589793,
            -0.8775825618903728,
            -0.479425538604203,
            -0.8775825618903728,
            -0.4794255386042027,
        ],
    );
}

#[test]
fn one_edge_elliptical_pcurve_expands_both_parameter_ends() {
    let (face, wire) = one_elliptical_arc_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(1, false).unwrap());
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            -2.641592653589793,
            3.641592653589793,
            -2.6327476856711183,
            -0.479425538604203,
            -2.6327476856711183,
            -0.4794255386042027,
        ],
    );
}

#[test]
fn intersecting_circle_pcurves_use_full_conic_domains() {
    let (face, wire) = intersecting_circle_wire();
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(
        &tool,
        &face,
        1,
        [0.0, 0.1, 1.0, 0.0, 0.9950041652780258, 0.09983341664682815],
    );
    assert_line_observation(
        &tool,
        &face,
        2,
        [
            3.0415926525847725,
            3.241592653589793,
            0.8049958348223092,
            0.09983341764682793,
            0.8049958347219745,
            -0.09983341664682809,
        ],
    );
}

#[test]
fn disjoint_circular_pcurves_use_extrema_when_ginter_has_no_hit() {
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let first = circle_arc(Point3::default(), normal, 0.05, 0.1);
    let second = circle_arc(
        Point3::new(2.1, 0.0, 0.0),
        normal,
        std::f64::consts::PI - 0.1,
        std::f64::consts::PI - 0.05,
    );
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            0.05,
            std::f64::consts::TAU,
            0.9987502603949663,
            0.04997916927067833,
            1.0,
            0.0,
        ],
    );
    assert_line_observation(
        &tool,
        &face,
        2,
        [
            -std::f64::consts::PI,
            std::f64::consts::PI - 0.05,
            1.1,
            0.0,
            1.1012497396050338,
            0.049979169270678275,
        ],
    );
}

#[test]
fn intersecting_trimmed_pcurves_keep_trimmed_curve_wrappers() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_trimmed_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_trimmed_pcurve(&tool, &face, 1, [0.0, 0.8]);
    assert_trimmed_pcurve(&tool, &face, 2, [-0.28284271247461906, 2.0_f64.sqrt()]);
}

#[test]
fn intersecting_offset_pcurves_keep_offset_curve_wrappers() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_offset_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_offset_pcurve(&tool, &face, 1, [0.0, 0.7828427124746191]);
    assert_offset_pcurve(&tool, &face, 2, [0.482_842_712_474_619_1, 2.0_f64.sqrt()]);
}

#[test]
fn trimmed_offsets_keep_both_native_curve_wrappers() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(1.0, 0.2, 0.0), Point3::new(2.0, 1.2, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_trimmed_offset_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_trimmed_offset_pcurve(&tool, &face, 1, [0.0, 0.7828427124746191]);
    assert_trimmed_offset_pcurve(&tool, &face, 2, [0.4828427124746191, 2.0_f64.sqrt()]);
}

#[test]
fn intersecting_bezier_pcurves_use_their_finite_parameter_domains() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.3, -0.2, 0.0), Point3::new(1.3, 0.8, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_bezier_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bezier_pcurve(&tool, &face, 1, [0.0, 0.5]);
    assert_bezier_pcurve(&tool, &face, 2, [0.2, 1.0]);
}

#[test]
fn intersecting_bspline_pcurves_use_their_full_knot_domains() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.3, -0.2, 0.0), Point3::new(1.3, 0.8, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_bspline_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bspline_pcurve(&tool, &face, 1, [0.0, 0.5]);
    assert_bspline_pcurve(&tool, &face, 2, [0.2, 1.0]);
}

#[test]
fn multispan_bspline_intersection_respects_internal_knot() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.0, 0.1, 0.0), Point3::new(1.0, 0.1, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_multispan_bspline_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bspline_pcurve(&tool, &face, 1, [0.0, 0.44364916731037085]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [0.44364916731037085, 1.0, 0.44364916731037085, 0.1, 1.0, 0.1],
    );
}

#[test]
fn cubic_bspline_intersection_matches_native_parameter() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.0, 0.1, 0.0), Point3::new(1.0, 0.1, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    cubic::set_cubic_spline_and_line(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bspline_pcurve(&tool, &face, 1, [0.0, 0.354_292_891_867_218_2]);
    assert_line_observation(
        &tool,
        &face,
        2,
        [
            0.354_292_891_867_218_2,
            1.0,
            0.354_292_891_867_218_2,
            0.1,
            1.0,
            0.1,
        ],
    );
}

#[test]
fn tangent_bezier_and_line_intersection_is_selected() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.0, 0.25, 0.0), Point3::new(1.0, 0.25, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_tangent_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bezier_pcurve(&tool, &face, 1, [0.0, 0.5]);
    assert_line_observation(&tool, &face, 2, [0.5, 1.0, 0.5, 0.25, 1.0, 0.25]);
}

#[test]
fn tangent_bspline_and_line_intersection_is_selected() {
    let first = edge_between(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_between(Point3::new(0.0, 0.25, 0.0), Point3::new(1.0, 0.25, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_tangent_bspline_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_bspline_pcurve(&tool, &face, 1, [0.0, 0.5]);
    assert_line_observation(&tool, &face, 2, [0.5, 1.0, 0.5, 0.25, 1.0, 0.25]);
}

#[test]
fn parabolic_extension_retains_native_not_done_failure_state() {
    let first = edge_between(Point3::new(0.04, 0.2, 0.0), Point3::new(0.01, 0.1, 0.0));
    let second = edge_between(Point3::new(0.2, 0.0, 0.0), Point3::new(1.2, 0.0, 0.0));
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_parabola_line_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    let error = tool.fix_gap_2d(2, false).unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    assert!(!tool.context().is_null());
    assert!(!tool.last_fix_status(ShapeExtendStatus::Done));
    assert!(!tool.last_fix_status(ShapeExtendStatus::Fail));
    assert_line_observation(&tool, &face, 1, [0.1, 0.2, 0.01, 0.1, 0.04, 0.2]);
    assert_line_observation(&tool, &face, 2, [0.0, 1.0, 0.2, 0.0, 1.2, 0.0]);
}

#[test]
fn intersecting_ellipses_use_the_full_conic_search_domain() {
    let first = edge_between(
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0 * 0.1_f64.cos(), 0.1_f64.sin(), 0.0),
    );
    let second = edge_between(
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(
            5.0 + 3.0 * (std::f64::consts::PI + 0.1).cos(),
            (std::f64::consts::PI + 0.1).sin(),
            0.0,
        ),
    );
    let (face, wire) = wire_with_disconnected_edges(&first, &second);
    set_intersecting_ellipse_pcurves(&face, &first, &second);
    let tool = ShapeFixWire::new();
    tool.load(&wire).unwrap();
    tool.set_face(&face).unwrap();
    assert!(tool.fix_gap_2d(2, false).unwrap());
    assert_line_observation(
        &tool,
        &face,
        1,
        [
            0.0,
            0.1000000010050206,
            3.0,
            0.0,
            2.9850124955330735,
            0.099_833_417_646_827_84,
        ],
    );
    assert_line_observation(
        &tool,
        &face,
        2,
        [
            3.041_592_652_584_772,
            3.241_592_653_589_793,
            2.0149875044669265,
            0.099_833_417_646_828_4,
            2.0149875041659224,
            -0.099_833_416_646_828_11,
        ],
    );
}
