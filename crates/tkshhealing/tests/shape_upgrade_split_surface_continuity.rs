//! Native parity cases for the 2D parameter scans and wrapper recursion.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomSweptSurface,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Xyz};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_upgrade::{ShapeUpgradeSplitSurfaceContinuity, SplitSurface};

#[test]
fn native_default_preserves_zero_continuity_order_and_explicit_c1_corrects_knots() {
    assert_native_case(
        "u_default",
        &run(smooth_surface(true, false), false, true, None),
    );
    assert_native_case(
        "u_remove",
        &run(smooth_surface(true, false), true, true, None),
    );
    assert_native_case(
        "v_remove",
        &run(smooth_surface(false, true), true, true, None),
    );
    assert_native_case(
        "uv_remove",
        &run(smooth_surface(true, true), true, true, None),
    );
}

#[test]
fn native_u_and_v_rejection_rows_keep_split_and_status_bits() {
    assert_native_case(
        "u_split",
        &run(corner_surface(true, false), true, true, None),
    );
    assert_native_case(
        "v_split",
        &run(corner_surface(false, true), true, true, None),
    );
    assert_native_case(
        "uv_split",
        &run(corner_surface(true, true), true, true, None),
    );
    assert_native_case(
        "u_split_tight",
        &run_with_tolerance(corner_surface(true, false), 0.0),
    );
}

#[test]
fn native_existing_ranges_and_segment_false_bounds_match() {
    assert_native_case(
        "u_split_existing",
        &run_with_bounds(corner_surface(true, false), (0.1, 0.9, 0.1, 0.9), true),
    );
    assert_native_case(
        "restricted_full_bounds",
        &run_with_tolerance_and_bounds(
            corner_surface(false, false),
            1.0e-9,
            true,
            false,
            Some((0.1, 0.9, 0.1, 0.9)),
        ),
    );
}

#[test]
fn extending_bounds_propagates_cleared_shared_sequence_range_failure() {
    let surface = corner_surface(false, false);
    let splitter = initialized_splitter(&surface, None);
    splitter
        .u_split_values()
        .change_sequence(|sequence| sequence.clear(None));

    let failure = splitter.compute(false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn native_trimmed_wrapper_recurses_over_clipped_basis_bounds() {
    let corner = corner_surface(true, true);
    let trimmed = Handle::new(
        GeomRectangularTrimmedSurface::new(&corner, (0.1, 0.9), (0.1, 0.9), (true, true)).unwrap(),
    )
    .cast();
    let trimmed_splitter = initialized_splitter(&trimmed, None);
    let shared_u = trimmed_splitter.u_split_values();
    let shared_v = trimmed_splitter.v_split_values();
    trimmed_splitter.set_criterion(Shape::C1);
    trimmed_splitter.set_tolerance(1.0e-9);
    trimmed_splitter.compute(true).unwrap();
    assert_shared_sequences(&trimmed_splitter, &shared_u, &shared_v);
    assert_native_case("trimmed", &trimmed_splitter);
}

fn assert_shared_sequences(
    splitter: &ShapeUpgradeSplitSurfaceContinuity,
    u_values: &rustycad_tkshhealing::shape_upgrade::SurfaceSplitValues,
    v_values: &rustycad_tkshhealing::shape_upgrade::SurfaceSplitValues,
) {
    assert_eq!(
        u_values.object_address(),
        splitter.u_split_values().object_address()
    );
    assert_eq!(
        v_values.object_address(),
        splitter.v_split_values().object_address()
    );
}

#[test]
fn native_offset_wrapper_escalates_basis_continuity() {
    let smooth = smooth_surface(true, true);
    let offset: Handle<dyn GeomSurface> =
        Handle::new(GeomOffsetSurface::new(&smooth, 1.0, true).unwrap()).cast();
    assert_native_case("offset", &run(offset.clone(), true, true, None));
    assert_native_case("offset_segment_false", &run(offset, true, false, None));
}

#[test]
fn native_linear_extrusion_replaces_its_basis_after_removal() {
    let curve = smooth_curve();
    let direction = Direction3::from_xyz(Xyz::new(0.0, 0.0, 1.0)).unwrap();
    let extrusion: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, direction).unwrap()).cast();
    let extrusion_splitter = run(extrusion.clone(), true, true, None);
    assert_native_case("linear_extrusion", &extrusion_splitter);
    let extrusion = extrusion.cast::<GeomSurfaceOfLinearExtrusion>();
    let basis = extrusion
        .get()
        .unwrap()
        .basis_curve()
        .cast::<GeomBSplineCurve>();
    assert_native_swept_multiplicity(
        "linear_extrusion",
        basis.get().unwrap().multiplicity(2).unwrap(),
    );
}

#[test]
fn native_revolution_keeps_its_basis_after_removal() {
    let curve = smooth_curve();
    let direction = Direction3::from_xyz(Xyz::new(0.0, 0.0, 1.0)).unwrap();
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), direction);
    let revolution: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(&curve, axis).unwrap()).cast();
    let revolution_splitter = run(revolution.clone(), true, true, None);
    assert_native_case("revolution", &revolution_splitter);
    let revolution = revolution.cast::<GeomSurfaceOfRevolution>();
    let basis = revolution
        .get()
        .unwrap()
        .basis_curve()
        .cast::<GeomBSplineCurve>();
    assert_native_swept_multiplicity("revolution", basis.get().unwrap().multiplicity(2).unwrap());
}

fn run(
    surface: Handle<dyn GeomSurface>,
    configure: bool,
    segment: bool,
    bounds: Option<(f64, f64, f64, f64)>,
) -> ShapeUpgradeSplitSurfaceContinuity {
    run_with_tolerance_and_bounds(surface, 1.0e-9, configure, segment, bounds)
}

fn run_with_tolerance(
    surface: Handle<dyn GeomSurface>,
    tolerance: f64,
) -> ShapeUpgradeSplitSurfaceContinuity {
    run_with_tolerance_and_bounds(surface, tolerance, true, true, None)
}

fn run_with_bounds(
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    segment: bool,
) -> ShapeUpgradeSplitSurfaceContinuity {
    run_with_tolerance_and_bounds(surface, 0.0, true, segment, Some(bounds))
}

fn run_with_tolerance_and_bounds(
    surface: Handle<dyn GeomSurface>,
    tolerance: f64,
    configure: bool,
    segment: bool,
    bounds: Option<(f64, f64, f64, f64)>,
) -> ShapeUpgradeSplitSurfaceContinuity {
    let splitter = ShapeUpgradeSplitSurfaceContinuity::new();
    initialize(&splitter, &surface, bounds);
    if configure {
        splitter.set_criterion(Shape::C1);
    }
    splitter.set_tolerance(tolerance);
    splitter.compute(segment).unwrap();
    splitter
}

fn initialize(
    splitter: &ShapeUpgradeSplitSurfaceContinuity,
    surface: &Handle<dyn GeomSurface>,
    bounds: Option<(f64, f64, f64, f64)>,
) {
    if let Some(bounds) = bounds {
        splitter.init_surface_bounds(surface, bounds, 0.0).unwrap();
    } else {
        splitter.init_surface(surface).unwrap();
    }
}

fn initialized_splitter(
    surface: &Handle<dyn GeomSurface>,
    bounds: Option<(f64, f64, f64, f64)>,
) -> ShapeUpgradeSplitSurfaceContinuity {
    let splitter = ShapeUpgradeSplitSurfaceContinuity::new();
    initialize(&splitter, surface, bounds);
    splitter
}

fn assert_native_case(name: &str, splitter: &ShapeUpgradeSplitSurfaceContinuity) {
    let row = oracle_row(name);
    let flags = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
    ];
    for (field, flag) in row[1..4].iter().zip(flags) {
        assert_eq!(
            splitter.status_flag(flag),
            *field == "1",
            "{name} status {flag:?}"
        );
    }
    assert_values(&splitter.u_split_values(), &row[4], name);
    assert_values(&splitter.v_split_values(), &row[5], name);
}

fn assert_values(
    values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>,
    given: &str,
    name: &str,
) {
    let expected: Vec<f64> = given
        .split(',')
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    let actual = values.sequence(|sequence| sequence.iter().copied().collect::<Vec<_>>());
    assert_eq!(actual.len(), expected.len(), "{name} split count");
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= expected.abs().max(1.0) * 1.0e-14,
            "{name}: {actual} != {expected}"
        );
    }
}

fn assert_native_swept_multiplicity(name: &str, actual: i32) {
    let row = oracle_row(name);
    assert_eq!(
        actual,
        row[6].parse::<i32>().unwrap(),
        "{name} basis curve knot multiplicity"
    );
}

fn oracle_row(name: &str) -> Vec<String> {
    include_str!("../../../tools/oracle/tkshhealing_split_surface_continuity.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .expect("named native oracle row")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn smooth_surface(insert_u: bool, insert_v: bool) -> Handle<dyn GeomSurface> {
    let poles = smooth_poles();
    let knots = Array1::from_shared(&[0.0, 1.0], 1);
    let mults = Array1::from_shared(&[3, 3], 1);
    let surface = GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (2, 2),
        (false, false),
    )
    .unwrap();
    if insert_u {
        surface.insert_u_knot(0.5, 2, 1.0e-12, false).unwrap();
    }
    if insert_v {
        surface.insert_v_knot(0.5, 2, 1.0e-12, false).unwrap();
    }
    Handle::new(surface).cast()
}

fn smooth_poles() -> Array2<'static, Point3> {
    let mut poles = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for u in 1..=3 {
        for v in 1..=3 {
            poles[(u, v)] = Point3::new((u - 1) as f64 / 2.0, (v - 1) as f64 / 2.0, 0.0);
        }
    }
    poles
}

fn smooth_curve() -> Handle<dyn GeomCurve> {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 1.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let poles = Array1::from_shared(&points, 1);
    let knots = Array1::from_shared(&[0.0, 1.0], 1);
    let mults = Array1::from_shared(&[3, 3], 1);
    let curve = GeomBSplineCurve::new(&poles, &knots, &mults, 2, false).unwrap();
    curve.insert_knot(0.5, 2, 1.0e-12, false).unwrap();
    Handle::new(curve).cast()
}

fn corner_surface(corner_u: bool, corner_v: bool) -> Handle<dyn GeomSurface> {
    let poles = corner_poles(corner_u, corner_v);
    let knots = Array1::from_shared(&[0.0, 0.5, 1.0], 1);
    let mults = Array1::from_shared(&[3, 2, 3], 1);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mults),
            (&knots, &mults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
    .cast()
}

fn corner_poles(corner_u: bool, corner_v: bool) -> Array2<'static, Point3> {
    let mut poles = Array2::with_bounds(1, 5, 1, 5).unwrap();
    for u in 1..=5 {
        for v in 1..=5 {
            let z = if corner_u { (u - 3).max(0) as f64 } else { 0.0 }
                + if corner_v { (v - 3).max(0) as f64 } else { 0.0 };
            poles[(u, v)] = Point3::new((u - 1) as f64 / 4.0, (v - 1) as f64 / 4.0, z);
        }
    }
    poles
}
