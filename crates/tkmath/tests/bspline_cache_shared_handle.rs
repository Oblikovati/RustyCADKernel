//! Runtime descriptor and shared owner behavior for the mutable cache wrappers.
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{RuntimeTyped, StandardTransient, Transient},
};
use rustycad_tkmath::{
    bsplclib::{Cache as CurveCache, SharedBSplineCache},
    bsplslib::{Cache as SurfaceCache, SharedBSplineSurfaceCache},
    gp::Point3,
};

fn curve_cache() -> CurveCache {
    let knots = flat_knots();
    let poles = flat_poles();
    CurveCache::new(1, false, &knots, &poles, None).unwrap()
}

fn surface_cache() -> SurfaceCache {
    let knots = flat_knots();
    SurfaceCache::new(1, false, &knots, 1, false, &knots, None).unwrap()
}

fn flat_knots() -> Array1<'static, f64> {
    let mut knots = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    knots
}

fn flat_poles() -> Array1<'static, Point3> {
    let mut poles = Array1::with_bounds(1, 2).unwrap();
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(1.0, 0.0, 0.0);
    poles
}

fn curve_shared_report() -> String {
    let cache = SharedBSplineCache::new(curve_cache());
    let alias = cache.clone();
    let base = cache.handle().cast::<StandardTransient>();
    format!(
        "{} {} {} {} {} {}\n",
        cache.dynamic_type().unwrap().name(),
        i32::from(cache.is_instance(CurveCache::static_type())),
        i32::from(cache.is_same(&alias)),
        i32::from(cache.is_kind(StandardTransient::static_type())),
        i32::from(cache.object_address() == base.object_address()),
        cache.ref_count()
    )
}

fn surface_shared_report() -> String {
    let cache = SharedBSplineSurfaceCache::new(surface_cache());
    let alias = cache.clone();
    let base = cache.handle().cast::<StandardTransient>();
    format!(
        "{} {} {} {} {} {}\n",
        cache.dynamic_type().unwrap().name(),
        i32::from(cache.is_instance(SurfaceCache::static_type())),
        i32::from(cache.is_same(&alias)),
        i32::from(cache.is_kind(StandardTransient::static_type())),
        i32::from(cache.object_address() == base.object_address()),
        cache.ref_count()
    )
}

#[test]
fn mutable_shared_cache_owners_match_native_handle_identity_and_type() {
    assert_eq!(
        format!("{}{}", curve_shared_report(), surface_shared_report()),
        include_str!("../../../tools/oracle/tkmath_bspline_cache_shared_handle.expected")
    );
}

#[test]
fn null_shared_cache_has_no_dynamic_object() {
    let curve = SharedBSplineCache::default();
    let surface = SharedBSplineSurfaceCache::default();
    assert!(curve.dynamic_type().is_none());
    assert!(surface.dynamic_type().is_none());
    assert!(!curve.is_instance(CurveCache::static_type()));
    assert!(!surface.is_instance(SurfaceCache::static_type()));
}

#[test]
fn shared_owner_aliases_increment_the_embedded_transient_counter() {
    let cache = SharedBSplineCache::new(curve_cache());
    let alias = cache.clone();
    assert!(cache.is_same(&alias));
    assert_eq!(
        cache
            .with(|value| value.transient_base().ref_count())
            .unwrap(),
        2
    );
}
