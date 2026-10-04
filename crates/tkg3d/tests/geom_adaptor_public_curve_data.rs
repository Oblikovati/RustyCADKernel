//! Native aggregate defaults and handle-copy behavior, not extra legacy declarations.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    geom::GeomBezierCurve,
    geom_adaptor::{CurveBSplineData, CurveBezierData, CurveDataVariant, CurveOffsetData},
};
use rustycad_tkmath::{
    bsplclib::{Cache, SharedBSplineCache},
    gp::{Direction3, Point3},
};

fn cache_inputs() -> (Array1<'static, f64>, Array1<'static, Point3>) {
    let mut knots = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    let mut poles = Array1::with_bounds(1, 2).unwrap();
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(1.0, 2.0, 3.0);
    (knots, poles)
}

#[test]
fn curve_aggregates_match_native_null_and_direction_defaults() {
    let offset = CurveOffsetData::default();
    let bezier = CurveBezierData::default();
    let spline = CurveBSplineData::default();
    assert!(offset.basis_adaptor.is_null() && offset.eval_rep.is_null());
    assert_eq!(offset.offset, 0.0);
    assert_eq!(offset.direction, Direction3::X);
    assert!(bezier.curve.is_null() && bezier.cache.is_null() && bezier.eval_rep.is_null());
    assert!(spline.curve.is_null() && spline.cache.is_null() && spline.eval_rep.is_null());
    assert!(matches!(
        CurveDataVariant::default(),
        CurveDataVariant::Uninitialized
    ));
}

#[test]
fn variant_copy_preserves_curve_and_mutable_cache_identity() {
    let (knots, poles) = cache_inputs();
    let record = CurveBezierData {
        curve: Handle::new(GeomBezierCurve::new(&poles).unwrap()),
        cache: SharedBSplineCache::new(Cache::new(1, false, &knots, &poles, None).unwrap()),
        ..CurveBezierData::default()
    };
    let variant = CurveDataVariant::Bezier(record.clone());
    let CurveDataVariant::Bezier(copy) = variant.clone() else {
        panic!("expected Bezier variant")
    };
    assert!(record.curve == copy.curve);
    assert!(record.cache.is_same(&copy.cache));
    copy.cache
        .with_mut(|cache| cache.build_cache(0.25, &knots, &poles, None).unwrap());
    assert_eq!(
        record.cache.with(|cache| cache.d0::<Point3>(0.25).unwrap()),
        Some(Point3::new(0.25, 0.5, 0.75))
    );
}

#[test]
fn every_native_variant_alternative_can_be_constructed_and_copied() {
    let variants = [
        CurveDataVariant::Uninitialized,
        CurveDataVariant::Line(Default::default()),
        CurveDataVariant::Circle(Default::default()),
        CurveDataVariant::Ellipse(Default::default()),
        CurveDataVariant::Hyperbola(Default::default()),
        CurveDataVariant::Parabola(Default::default()),
        CurveDataVariant::Offset(Default::default()),
        CurveDataVariant::Bezier(Default::default()),
        CurveDataVariant::BSpline(Default::default()),
    ];
    let kinds: Vec<_> = variants.iter().map(std::mem::discriminant).collect();
    for (index, variant) in variants.iter().enumerate() {
        assert_eq!(std::mem::discriminant(&variant.clone()), kinds[index]);
        assert!(!kinds[..index].contains(&kinds[index]));
    }
}
