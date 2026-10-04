//! Public `Geom2dAdaptor_Curve` nested data records and shared cache identity.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg2d::geom2d_adaptor::{
    BSplineData, BezierData, CurveDataVariant, OffsetData, SharedBSplineCache,
};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::gp::Point2;

fn cache_inputs() -> (Array1<'static, f64>, Array1<'static, Point2>) {
    let mut knots = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    let mut poles = Array1::with_bounds(1, 2).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(1.0, 1.0);
    (knots, poles)
}

#[test]
fn nested_records_have_native_default_values() {
    let offset = OffsetData::default();
    let bezier = BezierData::default();
    let bspline = BSplineData::default();

    assert!(offset.basis_adaptor.is_null());
    assert_eq!(offset.offset, 0.0);
    assert!(offset.eval_rep.is_null());
    assert!(bezier.curve.is_null());
    assert!(bezier.cache.is_null());
    assert!(bezier.eval_rep.is_null());
    assert!(bspline.curve.is_null());
    assert!(bspline.cache.is_null());
    assert!(bspline.eval_rep.is_null());
    let null_cache = SharedBSplineCache::default();
    assert!(null_cache.is_null());
    assert!(null_cache.is_same(&SharedBSplineCache::default()));
    assert!(null_cache.with(|value| value.params().degree).is_none());
    assert!(null_cache.with_mut(|value| value.params().degree).is_none());
    assert!(matches!(
        CurveDataVariant::default(),
        CurveDataVariant::Uninitialized
    ));
}

#[test]
fn cloned_records_keep_shared_cache_identity_and_mutations() {
    let (knots, poles) = cache_inputs();
    let cache = Cache::new(1, false, &knots, &poles, None).unwrap();
    let shared = SharedBSplineCache::new(cache);
    let source = BezierData {
        cache: shared.clone(),
        ..BezierData::default()
    };
    let copy = source.clone();
    let copy_cache = &copy.cache;
    assert!(shared.is_same(copy_cache));
    let mut released = copy_cache.clone();
    released.nullify();
    assert!(released.is_null());
    assert!(!shared.is_null());
    assert!(!shared.is_same(&released));

    shared.with_mut(|value| value.build_cache(0.25, &knots, &poles, None).unwrap());
    let cached_point: Point2 = copy_cache
        .with(|value| value.d0::<Point2>(0.25).unwrap())
        .unwrap();
    assert_eq!(cached_point, Point2::new(0.25, 0.25));
}
