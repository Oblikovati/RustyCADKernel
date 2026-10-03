//! Differential regressions for the pinned `ShapeUpgrade_ConvertSurfaceToBezierBasis` oracle.
use rustycad_tkernel::{
    ncollection::{Array1, Array2, HSequence},
    standard::Handle,
};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomLine, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, StandardDirection3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeConvertSurfaceToBezierBasis, SplitSurface},
};

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn offset_plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomOffsetSurface::new(&plane(), 2., false).unwrap()).cast()
}

fn revolution() -> Handle<dyn GeomSurface> {
    let frame = Axis2::new(
        Point3::new(2., 0., 0.),
        Direction3::from_standard(StandardDirection3::Y),
    )
    .unwrap();
    let circle = Handle::new(rustycad_tkg3d::geom::GeomCircle::new(frame, 1.).unwrap()).cast();
    let axis = Axis1::from_standard(StandardDirection3::Z);
    Handle::new(GeomSurfaceOfRevolution::new(&circle, axis).unwrap()).cast()
}

fn extrusion() -> Handle<dyn GeomSurface> {
    let line = Handle::new(GeomLine::new(Axis1::from_standard(StandardDirection3::X))).cast();
    let direction = Direction3::from_standard(StandardDirection3::Z);
    Handle::new(GeomSurfaceOfLinearExtrusion::new(&line, direction).unwrap()).cast()
}

fn split_spline() -> Handle<dyn GeomSurface> {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(0., 1., 0.1),
        Point3::new(1., 0., 0.2),
        Point3::new(1., 1., 0.3),
        Point3::new(2., 0., 0.4),
        Point3::new(2., 1., 0.6),
    ];
    let poles = Array2::from_shared(&points, 1, 3, 1, 2).unwrap();
    let uk = [0., 0.5, 1.];
    let um = [2, 1, 2];
    let vk = [0., 1.];
    let vm = [2, 2];
    let u = Array1::from_shared(&uk, 1);
    let um = Array1::from_shared(&um, 1);
    let v = Array1::from_shared(&vk, 1);
    let vm = Array1::from_shared(&vm, 1);
    Handle::new(
        GeomBSplineSurface::new(&poles, (&u, &um), (&v, &vm), (1, 1), (false, false)).unwrap(),
    )
    .cast()
}

fn bezier() -> Handle<dyn GeomSurface> {
    let poles = [
        Point3::new(0., 0., 0.),
        Point3::new(0., 1., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 1.),
    ];
    let array = Array2::from_shared(&poles, 1, 2, 1, 2).unwrap();
    Handle::new(GeomBezierSurface::new(&array).unwrap()).cast()
}

#[test]
fn plane_compute_build_matches_pinned_patch_types_and_status() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter
        .init_surface_bounds(&plane(), (-2., 3., -1., 4.))
        .unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done1));
    let segments = converter.segments();
    assert_eq!(
        (
            segments.nb_u_patches().unwrap(),
            segments.nb_v_patches().unwrap()
        ),
        (1, 1)
    );
    let patch = segments.patch(1, 1).unwrap();
    assert!(!patch.cast::<GeomBezierSurface>().is_null());
    assert_eq!(patch.bounds(), (0., 1., 0., 1.));
    let result = converter.result_surfaces();
    assert!(
        !result
            .patch(1, 1)
            .unwrap()
            .cast::<GeomBezierSurface>()
            .is_null()
    );
    assert_eq!(result.u_joint_value(1).unwrap(), -2.);
    assert_eq!(result.u_joint_value(2).unwrap(), 3.);
}

#[test]
fn restricted_bezier_compute_reports_done2_and_builds_bezier_patch() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter
        .init_surface_bounds(&bezier(), (0.2, 0.8, 0.1, 0.9))
        .unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done2));
    assert!(
        !converter
            .segments()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomBezierSurface>()
            .is_null()
    );
    assert!(
        !converter
            .result_surfaces()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomBezierSurface>()
            .is_null()
    );
}

#[test]
fn full_bezier_compute_retains_the_original_segment_handle() {
    let input = bezier();
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter
        .init_surface_bounds(&input, (0., 1., 0., 1.))
        .unwrap();
    converter.compute(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Ok));
    assert_eq!(
        converter.segments().patch(1, 1).unwrap().object_address(),
        input.object_address()
    );
}

#[test]
fn disabled_plane_mode_uses_trimmed_fallback_with_ok_status() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter.set_plane_mode(false);
    converter
        .init_surface_bounds(&plane(), (-2., 3., -1., 4.))
        .unwrap();
    converter.perform(true).unwrap();
    assert!(!converter.status_flag(ShapeExtendStatus::Done));
    assert!(converter.status_flag(ShapeExtendStatus::Ok));
    assert!(
        !converter
            .segments()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomRectangularTrimmedSurface>()
            .is_null()
    );
}

#[test]
fn mode_defaults_match_native_constructor() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    assert!(converter.plane_mode());
    assert!(converter.revolution_mode());
    assert!(converter.extrusion_mode());
    assert!(converter.bspline_mode());
}

#[test]
fn bspline_conversion_inserts_bezier_boundaries_and_maps_split_cells() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter.init_surface(&split_spline()).unwrap();
    let split_values = Handle::new(HSequence::default());
    split_values.append(0.25);
    split_values.append(0.75);
    converter.set_u_split_values(&split_values).unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done1));
    assert_eq!(converter.segments().nb_u_patches().unwrap(), 2);
    assert_eq!(converter.segments().nb_v_patches().unwrap(), 1);
    let values = converter
        .u_split_values()
        .sequence(|s| s.iter().copied().collect::<Vec<_>>());
    assert_eq!(values, [0., 0.25, 0.5, 0.75, 1.]);
    assert_eq!(converter.result_surfaces().nb_u_patches().unwrap(), 4);
    assert_eq!(converter.result_surfaces().nb_v_patches().unwrap(), 1);
}

#[test]
fn offset_plane_build_wraps_the_converted_basis_patch() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter
        .init_surface_bounds(&offset_plane(), (-1., 2., -3., 1.))
        .unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done1));
    assert!(
        !converter
            .segments()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomBezierSurface>()
            .is_null()
    );
    assert!(
        !converter
            .result_surfaces()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomOffsetSurface>()
            .is_null()
    );
}

#[test]
fn revolution_basis_converts_to_eight_native_bezier_arc_patches() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter.init_surface(&revolution()).unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done1));
    assert_eq!(converter.segments().nb_u_patches().unwrap(), 1);
    assert_eq!(converter.segments().nb_v_patches().unwrap(), 8);
    assert_eq!(converter.result_surfaces().nb_u_patches().unwrap(), 1);
    assert_eq!(converter.result_surfaces().nb_v_patches().unwrap(), 8);
    assert!(
        !converter
            .result_surfaces()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomSurfaceOfRevolution>()
            .is_null()
    );
}

#[test]
fn extrusion_line_converts_to_a_bezier_surface_patch() {
    let converter = ShapeUpgradeConvertSurfaceToBezierBasis::new();
    converter
        .init_surface_bounds(&extrusion(), (-2., 3., -1., 2.))
        .unwrap();
    converter.perform(true).unwrap();
    assert!(converter.status_flag(ShapeExtendStatus::Done1));
    assert_eq!(converter.segments().nb_u_patches().unwrap(), 1);
    assert!(
        !converter
            .segments()
            .patch(1, 1)
            .unwrap()
            .cast::<GeomBezierSurface>()
            .is_null()
    );
}

#[test]
fn surface_converter_parent_cast_shares_state_and_preserves_virtual_dispatch() {
    use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeSplitSurface;
    let converter = Handle::new(ShapeUpgradeConvertSurfaceToBezierBasis::new());
    let parent = converter.cast::<ShapeUpgradeSplitSurface>();
    assert!(!parent.is_null());
    parent
        .init_surface_bounds(&plane(), (0., 1., 0., 1.), 0.)
        .unwrap();
    assert!(parent.u_split_values() == converter.u_split_values());
    let virtual_tool = converter.cast::<dyn SplitSurface>();
    virtual_tool.compute(true).unwrap();
    assert!(!converter.segments().is_null());
    assert!(parent.status_flag(ShapeExtendStatus::Done1));
}
