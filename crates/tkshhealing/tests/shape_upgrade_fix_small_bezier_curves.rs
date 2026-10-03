//! Outcomes frozen from the pinned native small-Bézier oracle.
#[path = "support/small_bezier/mod.rs"]
mod fixture;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkshhealing::{
    shape_extend::{ShapeExtend, ShapeExtendStatus},
    shape_upgrade::{FixSmallCurves, ShapeUpgradeFixSmallBezierCurves, ShapeUpgradeFixSmallCurves},
};

fn configured(
    spatial: bool,
    planar: bool,
    seam: bool,
    mode: i32,
    geometry: u8,
) -> Handle<ShapeUpgradeFixSmallBezierCurves> {
    let edge = fixture::edge(spatial);
    let face = if planar {
        fixture::face(&edge, seam, geometry)
    } else {
        Face::default()
    };
    let fixer = Handle::new(ShapeUpgradeFixSmallBezierCurves::new());
    fixer.init(&edge, &face);
    fixer.set_split_curve_3d_tool(&Handle::new(fixture::SpatialCriterion::new(mode)).cast());
    fixer.set_split_curve_2d_tool(&Handle::new(fixture::PlanarCriterion::new(mode)).cast());
    fixer
}
fn native_row(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_fix_small_bezier_curves.expected")
        .lines()
        .find(|s| s.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect()
}
fn assert_case(
    name: &str,
    fixer: &Handle<ShapeUpgradeFixSmallBezierCurves>,
    first: f64,
    last: f64,
) {
    let (mut first, mut last) = (first, last);
    let mut spatial: Handle<dyn GeomCurve> = Handle::default();
    let mut planar: Handle<dyn Geom2dCurve> = Handle::default();
    let mut reversed = Handle::default();
    let result = fixer.cast::<dyn FixSmallCurves>().approx(
        &mut spatial,
        &mut planar,
        &mut reversed,
        &mut first,
        &mut last,
    );
    let result = result.map_or(-1., f64::from);
    let actual = vec![
        result,
        first,
        last,
        f64::from(!spatial.is_null()),
        f64::from(!planar.is_null()),
        f64::from(!reversed.is_null()),
        f64::from(fixer.status_flag(ShapeExtendStatus::Ok)),
    ];
    assert_eq!(actual, native_row(name), "{name}");
    if !spatial.is_null() {
        assert!((spatial.value(0.5).unwrap().x() - 0.5).abs() < 1e-7);
    }
    if !planar.is_null() {
        assert!((planar.value(0.5).unwrap().x() - 0.5).abs() < 1e-7);
    }
    if !reversed.is_null() {
        assert!((reversed.value(0.5).unwrap().y() - 1.).abs() < 1e-7);
    }
}
#[test]
fn small_bezier_geometry_and_seam_bounds_match_native() {
    assert_case("empty", &configured(false, false, false, 0, 0), -1., 2.);
    assert_case("spatial", &configured(true, false, false, 0, 0), -1., 2.);
    assert_case("planar", &configured(false, true, false, 0, 0), -1., 2.);
    assert_case("both", &configured(true, true, false, 0, 0), 0.25, 0.75);
    assert_case("seam-ok", &configured(false, true, true, 0, 0), 0.25, 0.75);
    assert_case(
        "seam-done",
        &configured(false, true, true, 2, 0),
        0.25,
        0.75,
    );
}
#[test]
fn small_bezier_splitter_failures_and_partial_outputs_match_native() {
    for (suffix, mode) in [("fail", 1), ("many", 3), ("throw", 4)] {
        assert_case(
            &format!("spatial-{suffix}"),
            &configured(true, false, false, mode, 0),
            -1.,
            2.,
        );
        assert_case(
            &format!("planar-{suffix}"),
            &configured(false, true, false, mode, 0),
            -1.,
            2.,
        );
    }
}
#[test]
fn small_bezier_trim_exceptions_and_cn_pcurve_match_native() {
    assert_case("seam-cn", &configured(false, true, true, 2, 2), 0.25, 0.75);
    assert_case(
        "spatial-zero",
        &configured(true, false, false, 0, 0),
        0.5,
        0.5,
    );
    assert_case(
        "planar-zero",
        &configured(false, true, false, 0, 0),
        0.5,
        0.5,
    );
    assert_case("planar-cn", &configured(false, true, false, 0, 1), -1., 2.);
}
#[test]
fn small_bezier_real_parent_shares_tools_precision_and_retained_status() {
    let fixer = configured(false, false, false, 0, 0);
    let base = fixer.cast::<ShapeUpgradeFixSmallCurves>();
    assert!(!base.is_null());
    base.set_precision(0.01);
    assert_eq!(fixer.precision(), 0.01);
    base.fix_small_curves_data()
        .with_mut(|f| f.status = ShapeExtend::encode_status(ShapeExtendStatus::Done3));
    let (mut c3, mut c2, mut cr) = (Handle::default(), Handle::default(), Handle::default());
    let (mut first, mut last) = (0., 1.);
    assert!(
        fixer
            .approx(&mut c3, &mut c2, &mut cr, &mut first, &mut last)
            .unwrap()
    );
    assert!(base.status_flag(ShapeExtendStatus::Done3));
    assert_eq!(
        base.split_curve_3d_tool().object_address(),
        fixer.split_curve_3d_tool().object_address()
    );
}
