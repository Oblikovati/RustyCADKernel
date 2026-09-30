use super::sampling::CurveSamples;
use super::seam_insertion::SeamInsertion;
use super::*;
use rustycad_tkg3d::geom::{GeomLine, GeomPlane};
use rustycad_tkmath::gp::{Axis1, Direction3};

pub(super) fn plane_projector() -> ShapeConstructProjectCurveOnSurface {
    let surface = Handle::new(
        GeomPlane::from_point(Point3::default(), Direction3::new(0., 0., 1.).unwrap()).unwrap(),
    )
    .cast();
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init(&surface, 1e-7).unwrap();
    projector
}
pub(super) fn planar_line() -> Handle<dyn GeomCurve> {
    let direction = Direction3::new(1., 0., 0.).unwrap();
    let axis = Axis1::new(Point3::default(), direction);
    Handle::new(GeomLine::new(axis)).cast()
}
#[test]
fn null_curve_and_uninitialized_projector_keep_native_failure_status() {
    let projector = ShapeConstructProjectCurveOnSurface::new();
    let mut result = Handle::default();
    assert!(
        !projector
            .perform(&planar_line(), (0., 1.), &mut result, (-1., -1.))
            .unwrap()
    );
    assert!(projector.status(ShapeExtendStatus::Fail1));
    let projector = plane_projector();
    assert!(
        !projector
            .perform(&Handle::default(), (0., 1.), &mut result, (-1., -1.))
            .unwrap()
    );
    assert!(result.is_null());
    assert!(projector.status(ShapeExtendStatus::Fail1));
}
#[test]
fn identical_analysis_identity_preserves_endpoint_cache() {
    let projector = plane_projector();
    let analysis = projector.analysis();
    projector
        .fields
        .write()
        .cache
        .push((Point3::default(), Point2::new(1., 2.)));
    projector.set_analysis_surface(&analysis);
    assert_eq!(projector.fields.read().cache.len(), 1);
    projector.set_surface(&analysis.surface()).unwrap();
    assert!(projector.fields.read().cache.is_empty());
}
#[test]
fn interpolation_retains_native_minimal_curve_for_all_coincident_points() {
    let projector = plane_projector();
    let mut samples = CurveSamples::generate(&planar_line(), (0., 1.)).unwrap();
    samples.uv.init(Point2::default());
    let result = projector.interpolate_pcurve(&samples.uv, &samples.parameters);
    assert!(!result.is_null());
    let last = result.value(1.).unwrap();
    let expected = 1e-7 / (100. * 23.);
    assert_eq!(last.x(), expected);
    assert_eq!(last.y(), expected);
}
#[test]
fn seam_insertion_keeps_middle_sample_when_wrapped_coordinate_is_outside() {
    let projector = plane_projector();
    let curve = planar_line();
    let mut samples = CurveSamples::with_count(2);
    samples.parameters[1] = 0.;
    samples.parameters[2] = 10.;
    samples.points[1] = curve.value(0.).unwrap();
    samples.points[2] = curve.value(10.).unwrap();
    let mut seam = SeamInsertion {
        coordinate: 1,
        period: 10.,
        tolerance: 1e-6,
        adjusting: false,
    };
    let mut index = 2;
    let mut current = 9.9;
    // The true midpoint lies between the unwrapped endpoints, so native inserts it.
    seam.apply(
        &projector,
        &curve,
        &mut samples,
        &mut index,
        &mut current,
        0.1,
    )
    .unwrap();
    assert_eq!(samples.points.length(), 3);
    assert_eq!(index, 3);
    assert!(!seam.adjusting);
    assert!(samples.parameters[2] > samples.parameters[1]);
}
