//! Direct translation of Intf_Tool_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkgeomalgo::intf::Tool;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Direction2, Direction3, Hyperbola2, Hyperbola3, Parabola2, Parabola3, Point2,
    Point3,
};

fn domain2d(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> BoundingBox2 {
    let mut aDomain = BoundingBox2::default();
    aDomain.update_bounds([xmin, ymin], [xmax, ymax]);
    aDomain
}

fn domain3d() -> BoundingBox3 {
    let mut aDomain = BoundingBox3::default();
    aDomain.update_bounds([-5., -5., -5.], [5., 5., 5.]);
    aDomain
}

fn x_axis2d(x: f64, y: f64) -> Axis2d {
    Axis2d::new(Point2::new(x, y), Direction2::new(1., 0.).unwrap())
}

fn z_frame() -> Axis2 {
    Axis2::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap()
}

// Regression test for bugs #75/#76: Hypr2dBox must produce valid segments
// with finite begin/end parameters and not overflow fixed-size arrays.
#[test]
fn Intf_Tool__Hypr2dBox_ProducesValidSegments() {
    let anAxis = x_axis2d(0., 0.);
    let aHypr = Hyperbola2::from_x_axis(anAxis, 2.0, 1.0, true).unwrap();
    let aDomain = domain2d(-5., -5., 5., 5.);

    let mut aResult = BoundingBox2::default();
    let mut aTool = Tool::new();
    aTool.hypr2d_box(aHypr, &aDomain, &mut aResult).unwrap();

    let aNbSeg = aTool.nb_segments();
    assert!(aNbSeg >= 0);
    assert!(aNbSeg <= 6);

    for i in 1..=aNbSeg {
        let aBegin = aTool.begin_param(i).unwrap();
        let aEnd = aTool.end_param(i).unwrap();
        assert!(
            aBegin.is_finite() || aBegin == -precision::infinite(),
            "Segment {i} begin={aBegin}"
        );
        assert!(
            aEnd.is_finite() || aEnd == precision::infinite(),
            "Segment {i} end={aEnd}"
        );
        assert!(aBegin <= aEnd, "Segment {i} begin > end");
    }
}

// Regression test for bugs #75/#76: Parab2dBox must produce valid segments.
#[test]
fn Intf_Tool__Parab2dBox_ProducesValidSegments() {
    let anAxis = x_axis2d(0., 0.);
    let aParab = Parabola2::from_mirror_axis(anAxis, 1.0, true).unwrap();
    let aDomain = domain2d(-5., -5., 5., 5.);

    let mut aResult = BoundingBox2::default();
    let mut aTool = Tool::new();
    aTool.parab2d_box(aParab, &aDomain, &mut aResult).unwrap();

    let aNbSeg = aTool.nb_segments();
    assert!(aNbSeg >= 0);
    assert!(aNbSeg <= 6);

    for i in 1..=aNbSeg {
        let aBegin = aTool.begin_param(i).unwrap();
        let aEnd = aTool.end_param(i).unwrap();
        assert!(
            aBegin.is_finite() || aBegin == -precision::infinite(),
            "Segment {i} begin={aBegin}"
        );
        assert!(
            aEnd.is_finite() || aEnd == precision::infinite(),
            "Segment {i} end={aEnd}"
        );
        assert!(aBegin <= aEnd, "Segment {i} begin > end");
    }
}

// Regression test for bugs #75/#76: ParabBox (3D) must produce valid segments.
#[test]
fn Intf_Tool__ParabBox_ProducesValidSegments() {
    let aParab = Parabola3::new(z_frame(), 1.0).unwrap();
    let aDomain = domain3d();

    let mut aResult = BoundingBox3::default();
    let mut aTool = Tool::new();
    aTool.parab_box(aParab, &aDomain, &mut aResult).unwrap();

    let aNbSeg = aTool.nb_segments();
    assert!(aNbSeg >= 0);
    assert!(aNbSeg <= 6);

    for i in 1..=aNbSeg {
        let aBegin = aTool.begin_param(i).unwrap();
        let aEnd = aTool.end_param(i).unwrap();
        assert!(aBegin <= aEnd, "Segment {i} begin > end");
    }
}

// HyprBox (3D) produces valid segments.
#[test]
fn Intf_Tool__HyprBox_ProducesValidSegments() {
    let aHypr = Hyperbola3::new(z_frame(), 2.0, 1.0).unwrap();
    let aDomain = domain3d();

    let mut aResult = BoundingBox3::default();
    let mut aTool = Tool::new();
    aTool.hypr_box(aHypr, &aDomain, &mut aResult).unwrap();

    let aNbSeg = aTool.nb_segments();
    assert!(aNbSeg >= 0);
    assert!(aNbSeg <= 6);
}

// Small domain that doesn't intersect the curve should produce zero segments.
#[test]
fn Intf_Tool__Hypr2dBox_NoIntersection_ZeroSegments() {
    let anAxis = x_axis2d(100., 100.);
    let aHypr = Hyperbola2::from_x_axis(anAxis, 0.1, 0.05, true).unwrap();
    let aDomain = domain2d(-1., -1., 1., 1.);

    let mut aResult = BoundingBox2::default();
    let mut aTool = Tool::new();
    aTool.hypr2d_box(aHypr, &aDomain, &mut aResult).unwrap();

    assert_eq!(aTool.nb_segments(), 0);
}
