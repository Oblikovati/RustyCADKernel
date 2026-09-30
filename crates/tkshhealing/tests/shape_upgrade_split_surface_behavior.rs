//! Native SplitSurface contracts; the differential corpus covers dynamic geometry.
use rustycad_tkernel::{
    ncollection::{Array2, HSequence},
    standard::Handle,
};
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCurve, GeomLine, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkmath::gp::{Axis1, Axis3, Point3, dz};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeSplitSurface, SplitSurface},
};
fn bezier_surface() -> Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            poles
                .set_value(u, v, Point3::new(u as f64, v as f64, (u * v) as f64))
                .unwrap();
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
fn add_midpoint(split: &ShapeUpgradeSplitSurface) {
    let values = Handle::new(HSequence::default());
    values.append(0.5);
    split.set_u_split_values(&values).unwrap();
    split.set_v_split_values(&values).unwrap();
}
#[test]
fn natural_single_patch_reuses_surface_and_existing_composite() {
    let surface = bezier_surface();
    let split = ShapeUpgradeSplitSurface::new();
    assert!(split.result_surfaces().is_null());
    split.init_surface(&surface).unwrap();
    let result = split.result_surfaces();
    split.perform(true).unwrap();
    assert_eq!(
        result.object_address(),
        split.result_surfaces().object_address()
    );
    assert_eq!(
        result.patch(1, 1).unwrap().object_address(),
        surface.object_address()
    );
    assert!(split.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn multipart_bezier_segments_even_with_segment_false() {
    let surface = bezier_surface();
    let split = ShapeUpgradeSplitSurface::new();
    split.init_surface(&surface).unwrap();
    add_midpoint(&split);
    split.build(false).unwrap();
    let result = split.result_surfaces();
    assert_eq!(result.nb_u_patches().unwrap(), 2);
    assert_eq!(result.nb_v_patches().unwrap(), 2);
    assert!(split.status_flag(ShapeExtendStatus::Done1));
    assert!(split.status_flag(ShapeExtendStatus::Done3));
    let piece = result.patch(1, 1).unwrap();
    assert!(!piece.cast::<GeomBezierSurface>().is_null());
    assert_eq!(
        piece.value(1., 1.).unwrap(),
        surface.value(0.5, 0.5).unwrap()
    );
    assert_eq!(surface.bounds(), (0., 1., 0., 1.));
}
#[test]
fn trimmed_wrapper_replaces_composite_but_retains_parent_row_fields() {
    let surface = bezier_surface();
    let trimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&surface, (0., 1.), (0., 1.), (true, true)).unwrap(),
    )
    .cast();
    let split = ShapeUpgradeSplitSurface::new();
    split.init_surface(&trimmed).unwrap();
    let previous = split.result_surfaces();
    add_midpoint(&split);
    split.build(true).unwrap();
    assert_ne!(
        previous.object_address(),
        split.result_surfaces().object_address()
    );
    assert!(previous.patches().is_null());
    assert_eq!(
        split.split_surface_data().with(|f| (f.rows, f.columns)),
        (1, 1)
    );
    assert_eq!(split.result_surfaces().nb_u_patches().unwrap(), 2);
}
#[test]
fn explicit_area_measures_isos_and_natural_init_retains_metrics() {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let split = ShapeUpgradeSplitSurface::new();
    split
        .init_surface_bounds(&surface, (0., 2., 0., 3.), 4.)
        .unwrap();
    assert_eq!(
        split
            .split_surface_data()
            .with(|f| (f.area, f.u_size, f.v_size)),
        (4., 2., 3.)
    );
    split.init_surface(&surface).unwrap();
    assert_eq!(
        split
            .split_surface_data()
            .with(|f| (f.area, f.u_size, f.v_size)),
        (4., 2., 3.)
    );
}
#[test]
fn empty_mutable_split_sequence_raises_without_changing_result() {
    let split = ShapeUpgradeSplitSurface::new();
    split.init_surface(&bezier_surface()).unwrap();
    split.build(true).unwrap();
    let result = split.result_surfaces();
    split.u_split_values().change_sequence(|s| s.clear(None));
    assert_eq!(
        split.build(true).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        split.result_surfaces().object_address(),
        result.object_address()
    );
}
#[test]
fn native_sweep_null_curve_crash_returns_explicit_rust_failure() {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, dz()).unwrap()).cast();
    let split = ShapeUpgradeSplitSurface::new();
    split
        .init_surface_bounds(&surface, (0., 1., 0., 1.), 0.)
        .unwrap();
    add_midpoint(&split);
    let error = split.build(false).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert!(split.result_surfaces().patches().is_null());
    assert!(split.status_flag(ShapeExtendStatus::Done1));
}

#[test]
fn one_cross_direction_value_keeps_revolution_grid_unmodified_on_error() {
    use rustycad_tkg3d::geom::GeomSurfaceOfRevolution;
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(&curve, Axis1::default()).unwrap()).cast();
    let split = ShapeUpgradeSplitSurface::new();
    split
        .init_surface_bounds(&surface, (0., 1., 0., 1.), 0.)
        .unwrap();
    split.u_split_values().change_sequence(|s| s.clear(None));
    split.u_split_values().append(0.2);
    assert_eq!(
        split.build(true).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(split.result_surfaces().patches().is_null());
}
