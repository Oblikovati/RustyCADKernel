//! Native angular parameter/status fixtures; invalid native integer conversions stay explicit.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution,
    GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis1, Axis3, Point3, dz};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitSurface, ShapeUpgradeSplitSurfaceAngle, SplitSurface,
};

fn cylinder() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast()
}
fn trim(surface: &Handle<dyn GeomSurface>, last: f64) -> Handle<dyn GeomSurface> {
    Handle::new(
        GeomRectangularTrimmedSurface::new(surface, (0., last), (0., 1.), (true, true)).unwrap(),
    )
    .cast()
}
fn offset(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    Handle::new(GeomOffsetSurface::new(surface, 0.2, false).unwrap()).cast()
}
fn expected_bits(name: &str) -> Vec<u64> {
    let rows = include_str!("../../../tools/oracle/tkshhealing_split_surface_angle.expected");
    let row = rows
        .lines()
        .find(|s| s.starts_with(&format!("{name} ")))
        .unwrap();
    row.split_whitespace()
        .skip(1)
        .map(|s| s.parse::<f64>().unwrap().to_bits())
        .collect()
}
fn observed_bits(splitter: &ShapeUpgradeSplitSurfaceAngle) -> Vec<u64> {
    let status = splitter.split_surface_data().with(|f| f.status);
    let mut row = vec![f64::from(status), 1.];
    row.extend(
        splitter
            .u_split_values()
            .sequence(|s| s.iter().copied().collect::<Vec<_>>()),
    );
    row.iter().map(|x| x.to_bits()).collect()
}
fn assert_native(
    name: &str,
    surface: &Handle<dyn GeomSurface>,
    last: f64,
    maximum: f64,
    seed: i32,
) {
    let splitter = ShapeUpgradeSplitSurfaceAngle::new(maximum);
    splitter
        .init_surface_bounds(surface, (0., last, 0., 1.), 0.)
        .unwrap();
    splitter.split_surface_data().with_mut(|f| f.status = seed);
    let previous = splitter.u_split_values();
    splitter.compute(false).unwrap();
    assert_eq!(
        previous.object_address(),
        splitter.u_split_values().object_address()
    );
    assert_eq!(observed_bits(&splitter), expected_bits(name), "{name}");
}
#[test]
fn native_angular_segment_counts_and_statuses_match() {
    let surface = cylinder();
    for (name, last, maximum) in [
        ("cylinder-two", std::f64::consts::TAU, std::f64::consts::PI),
        ("cylinder-one", 1., 2.),
        ("cylinder-equal", 2., 2.),
        ("cylinder-three", 5., 2.),
        ("negative-count", 5., -2.),
    ] {
        assert_native(name, &surface, last, maximum, 0);
    }
}
fn revolution() -> Handle<dyn GeomSurface> {
    let axis = Axis1::new(Point3::new(1., 0., 0.), dz());
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    Handle::new(GeomSurfaceOfRevolution::new(&curve, Axis1::new(Point3::default(), dz())).unwrap())
        .cast()
}
#[test]
fn all_native_rotational_dynamic_types_split() {
    let cone = Handle::new(GeomConicalSurface::new(Axis3::default(), 0.5, 1.).unwrap()).cast();
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    let torus = Handle::new(GeomToroidalSurface::new(Axis3::default(), 3., 1.).unwrap()).cast();
    for (name, surface) in [
        ("cone", cone),
        ("sphere", sphere),
        ("torus", torus),
        ("revolution", revolution()),
    ] {
        assert_native(name, &surface, 5., 2., 0);
    }
}
#[test]
fn wrapper_unwrapping_is_one_level_and_small_trims_retain_status() {
    let cylinder = cylinder();
    let small = trim(&cylinder, 1.);
    assert_native("trim-small-retains", &small, 1., 2., 8);
    assert_native("trim-equal", &small, 1., 1., 8);
    assert_native("trim-wide", &trim(&cylinder, 2.), 1., 2., 8);
    let offset = offset(&cylinder);
    assert_native("offset", &offset, 5., 2., 0);
    assert_native("trim-offset", &trim(&offset, 5.), 5., 2., 8);
    let plane = Handle::new(GeomPlane::new(Axis3::default())).cast();
    assert_native("plane-retains", &plane, 5., 2., 8);
}
#[test]
fn shared_angle_setting_and_virtual_dispatch_retain_parent_state() {
    let splitter = Handle::new(ShapeUpgradeSplitSurfaceAngle::new(1.));
    let alias = splitter.clone();
    alias.set_max_angle(2.);
    assert_eq!(splitter.max_angle(), 2.);
    splitter
        .init_surface_bounds(&cylinder(), (0., 5., 0., 1.), 0.)
        .unwrap();
    let base = splitter.cast::<ShapeUpgradeSplitSurface>();
    let criterion = splitter.cast::<dyn SplitSurface>();
    criterion.compute(true).unwrap();
    assert_eq!(base.u_split_values().sequence(|s| s.len()), 4);
    assert_eq!(
        base.u_split_values().object_address(),
        splitter.u_split_values().object_address()
    );
    criterion.build(true).unwrap();
    assert!(!splitter.result_surfaces().is_null());
}
#[test]
fn undefined_native_segment_integer_conversion_returns_explicit_failure() {
    for maximum in [0., f64::NAN, 1e-300] {
        let splitter = ShapeUpgradeSplitSurfaceAngle::new(maximum);
        splitter
            .init_surface_bounds(&cylinder(), (0., 5., 0., 1.), 0.)
            .unwrap();
        let failure = splitter.compute(false).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert_eq!(splitter.u_split_values().sequence(|s| s.len()), 2);
    }
}
