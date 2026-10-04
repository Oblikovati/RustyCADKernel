//! Public `GeomAdaptor_Surface` records and nullable shared surface-cache identity.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorSurfaceBSplineData, GeomAdaptorSurfaceBezierData, GeomAdaptorSurfaceDataVariant,
    GeomAdaptorSurfaceExtrusionData, GeomAdaptorSurfaceOffsetData,
    GeomAdaptorSurfaceRevolutionData,
};
use rustycad_tkmath::bsplslib::{Cache, SharedBSplineSurfaceCache};
use rustycad_tkmath::gp::{Point3, Xyz};

fn cache_inputs() -> (Array1<'static, f64>, Array2<'static, Point3>) {
    let mut flat_knots = Array1::with_bounds(1, 4).unwrap();
    flat_knots
        .as_mut_slice()
        .copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    poles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    poles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    poles[(2, 2)] = Point3::new(1.0, 1.0, 1.0);
    (flat_knots, poles)
}

fn assert_extrusion_defaults() {
    let record = GeomAdaptorSurfaceExtrusionData::default();
    assert!(record.basis_curve.is_null());
    assert_eq!(record.direction, Xyz::default());
    assert!(record.eval_rep.is_null());
}

fn assert_revolution_defaults() {
    let record = GeomAdaptorSurfaceRevolutionData::default();
    assert!(record.basis_curve.is_null());
    assert_eq!(record.axis.location(), Point3::default());
    assert_eq!(record.axis.direction().z(), 1.0);
    assert!(record.eval_rep.is_null());
}

fn assert_offset_defaults() {
    let record = GeomAdaptorSurfaceOffsetData::default();
    assert!(record.basis_adaptor.is_null());
    assert!(record.equivalent_adaptor.is_null());
    assert!(record.offset_surface.is_null());
    assert_eq!(record.offset, 0.0);
    assert!(record.eval_rep.is_null());
}

fn assert_spline_defaults() {
    let bezier = GeomAdaptorSurfaceBezierData::default();
    let bspline = GeomAdaptorSurfaceBSplineData::default();
    assert!(bezier.surface.is_null());
    assert!(bezier.cache.is_null());
    assert!(bezier.eval_rep.is_null());
    assert!(bspline.surface.is_null());
    assert!(bspline.cache.is_null());
    assert!(bspline.eval_rep.is_null());
}

#[test]
fn public_surface_records_match_native_defaults() {
    assert_extrusion_defaults();
    assert_revolution_defaults();
    assert_offset_defaults();
    assert_spline_defaults();
    assert!(matches!(
        GeomAdaptorSurfaceDataVariant::default(),
        GeomAdaptorSurfaceDataVariant::Uninitialized
    ));
}

#[test]
fn cloned_bezier_record_shares_mutable_cache() {
    let (knots, poles) = cache_inputs();
    let cache = Cache::new(1, false, &knots, 1, false, &knots, None).unwrap();
    let shared = SharedBSplineSurfaceCache::new(cache);
    let source = GeomAdaptorSurfaceBezierData {
        cache: shared.clone(),
        ..GeomAdaptorSurfaceBezierData::default()
    };
    let copy = source.clone();
    assert!(shared.is_same(&copy.cache));

    shared.with_mut(|value| {
        value
            .build_cache(0.25, 0.5, &knots, &knots, &poles, None)
            .unwrap()
    });
    let cached_point = copy
        .cache
        .with(|value| value.d0(0.25, 0.5).unwrap())
        .unwrap();
    assert_eq!(cached_point, Point3::new(0.25, 0.5, 0.125));
}
