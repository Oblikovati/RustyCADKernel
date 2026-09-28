//=================================================================================================
// Line Tests
//=================================================================================================

#[test]
fn BndLibTest__Line_FiniteSegment() {
    // Line along X axis from (0,0,0) in direction (1,0,0)
    let aLine = Line3::new(pnt(0., 0., 0.), dir(1., 0., 0.));
    let mut aBox = BoundingBox3::default();

    bnd_lib::add_lin(&aLine, 0., 10., 0., &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 10., precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, 0., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 0., precision::confusion());
}

#[test]
fn BndLibTest__Line_PositiveDirection_NegativeInfinite() {
    // Line along +X axis with P1 = -Inf, P2 = 5
    // As t -> -Inf, x -> -Inf, so Xmin should be open
    let aLine = Line3::new(pnt(0., 0., 0.), dir(1., 0., 0.));
    let mut aBox = BoundingBox3::default();

    bnd_lib::add_lin(&aLine, -f64::INFINITY, 5., 0., &mut aBox).unwrap();

    assert!(aBox.is_open_xmin(), "Xmin should be open for line in +X direction with P1=-Inf");
    assert!(!aBox.is_open_xmax(), "Xmax should not be open");
}

#[test]
fn BndLibTest__Line_NegativeDirection_NegativeInfinite() {
    // Line along -X axis with P1 = -Inf, P2 = 5
    // Direction is (-1, 0, 0), so as t -> -Inf, x -> +Inf
    // Therefore Xmax should be open, not Xmin
    let aLine = Line3::new(pnt(0., 0., 0.), dir(-1., 0., 0.));
    let mut aBox = BoundingBox3::default();

    bnd_lib::add_lin(&aLine, -f64::INFINITY, 5., 0., &mut aBox).unwrap();

    assert!(aBox.is_open_xmax(), "Xmax should be open for line in -X direction with P1=-Inf");
    assert!(!aBox.is_open_xmin(), "Xmin should not be open");
}

#[test]
fn BndLibTest__Line_NegativeDirection_PositiveInfinite() {
    // Line along -X axis with P1 = -5, P2 = +Inf
    // Direction is (-1, 0, 0), so as t -> +Inf, x -> -Inf
    // Therefore Xmin should be open
    let aLine = Line3::new(pnt(0., 0., 0.), dir(-1., 0., 0.));
    let mut aBox = BoundingBox3::default();

    bnd_lib::add_lin(&aLine, -5., f64::INFINITY, 0., &mut aBox).unwrap();

    assert!(aBox.is_open_xmin(), "Xmin should be open for line in -X direction with P2=+Inf");
    assert!(!aBox.is_open_xmax(), "Xmax should not be open");
}

#[test]
fn BndLibTest__Line_DiagonalDirection_BothInfinite() {
    // Line in diagonal direction (-1, -1, -1) with P1=-Inf, P2=+Inf
    // As t -> -Inf: position goes to (+Inf, +Inf, +Inf) - opens max
    // As t -> +Inf: position goes to (-Inf, -Inf, -Inf) - opens min
    let aLine = Line3::new(pnt(0., 0., 0.), dir(-1., -1., -1.));
    let mut aBox = BoundingBox3::default();

    bnd_lib::add_lin(&aLine, -f64::INFINITY, f64::INFINITY, 0., &mut aBox).unwrap();

    assert!(aBox.is_open_xmin(), "Xmin should be open");
    assert!(aBox.is_open_xmax(), "Xmax should be open");
    assert!(aBox.is_open_ymin(), "Ymin should be open");
    assert!(aBox.is_open_ymax(), "Ymax should be open");
    assert!(aBox.is_open_zmin(), "Zmin should be open");
    assert!(aBox.is_open_zmax(), "Zmax should be open");
}

//=================================================================================================
// Circle Tests
//=================================================================================================

#[test]
fn BndLibTest__Circle_Full() {
    // Circle of radius 5 centered at origin in XY plane
    let aCirc = Circle3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 5.).unwrap();
    let mut aBoxFull = BoundingBox3::default();
    let mut aBoxArc = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_circ(&aCirc, aTol, &mut aBoxFull).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBoxFull);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 0., precision::confusion());

    // Full circle via parameters [0, 2*PI]
    bnd_lib::add_circ_arc(&aCirc, 0., 2. * PI, aTol, &mut aBoxArc).unwrap();
    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBoxArc);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
}

#[test]
fn BndLibTest__Circle_Arc_FirstQuadrant() {
    // Circle arc from 0 to PI/2 (first quadrant)
    let aCirc = Circle3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_circ_arc(&aCirc, 0., PI / 2., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
}

#[test]
fn BndLibTest__Circle_RotatedAxis() {
    // Circle in XZ plane (axis along Y)
    let aCirc = Circle3::new(ax2(pnt(0., 0., 0.), dir(0., 1., 0.)), 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_circ(&aCirc, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -3., precision::confusion());
    EXPECT_NEAR(aXmax, 3., precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, 0., precision::confusion());
    EXPECT_NEAR(aZmin, -3., precision::confusion());
    EXPECT_NEAR(aZmax, 3., precision::confusion());
}

//=================================================================================================
// Ellipse Tests
//=================================================================================================

#[test]
fn BndLibTest__Ellipse_Full() {
    // Ellipse with major radius 10, minor radius 5 in XY plane
    let aElips = Ellipse3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_elips(&aElips, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -10., precision::confusion());
    EXPECT_NEAR(aXmax, 10., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 0., precision::confusion());
}

#[test]
fn BndLibTest__Ellipse_Arc() {
    // Ellipse arc from 0 to PI/2
    let aElips = Ellipse3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_elips_arc(&aElips, 0., PI / 2., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // At t=0: (10, 0, 0), at t=PI/2: (0, 5, 0)
    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 10., precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
}

//=================================================================================================
// Hyperbola Tests
//=================================================================================================

#[test]
fn BndLibTest__Hyperbola_SimpleArc() {
    // Hyperbola with RealAxis=3, ImaginaryAxis=2 in XY plane
    // Parameterization: (3*cosh(t), 2*sinh(t), 0)
    let aHypr = Hyperbola3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 3., 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_hypr(&aHypr, -1., 1., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // At t=-1: (3*cosh(-1), 2*sinh(-1)) = (4.629..., -2.350...)
    // At t=1:  (3*cosh(1), 2*sinh(1))   = (4.629...,  2.350...)
    // At t=0:  (3, 0) - minimum X
    let aExpectedXmin = 3.; // at t=0
    let aExpectedXmax = 3. * f64::cosh(1.); // at t=+-1
    let aExpectedYmin = 2. * f64::sinh(-1.); // at t=-1
    let aExpectedYmax = 2. * f64::sinh(1.); // at t=1

    EXPECT_NEAR(aXmin, aExpectedXmin, precision::confusion());
    EXPECT_NEAR(aXmax, aExpectedXmax, precision::confusion());
    EXPECT_NEAR(aYmin, aExpectedYmin, precision::confusion());
    EXPECT_NEAR(aYmax, aExpectedYmax, precision::confusion());
}

#[test]
fn BndLibTest__Hyperbola_Rotated_MultipleExtrema() {
    // Rotated hyperbola where X, Y, Z extrema occur at different parameter values.
    // This tests the fix for the extrema loop that previously broke after finding
    // the first extremum.
    let aAx2 = Axis2::with_x_direction(pnt(0., 0., 0.), dir(1., 1., 1.), dir(1., -1., 0.)).unwrap();
    let aHypr = Hyperbola3::new(aAx2, 5., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_hypr(&aHypr, -2., 2., aTol, &mut aBox).unwrap();

    // Verify box is valid and contains reasonable bounds
    assert!(!aBox.is_void(), "Box should not be void");

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // For a rotated hyperbola, the box should be non-degenerate in all dimensions
    assert!(aXmin < aXmax, "X range should be non-zero");
    assert!(aYmin < aYmax, "Y range should be non-zero");
    // Z may be zero if hyperbola lies in a plane, but should still be valid
}

#[test]
fn BndLibTest__Hyperbola_InfiniteParameter() {
    // Hyperbola with one infinite parameter
    let aHypr = Hyperbola3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 3., 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_hypr(&aHypr, 0., f64::INFINITY, aTol, &mut aBox).unwrap();

    assert!(aBox.is_open_xmax(), "Xmax should be open for hyperbola with P2=+Inf");
    assert!(aBox.is_open_ymax(), "Ymax should be open for hyperbola with P2=+Inf");
}

//=================================================================================================
// Parabola Tests
//=================================================================================================

#[test]
fn BndLibTest__Parabola_FiniteArc() {
    // Parabola with focal length 2 in XY plane
    // Parameterization: (t^2/(4*f), t, 0) = (t^2/8, t, 0) for f=2
    let aParab = Parabola3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_parab(&aParab, -4., 4., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // At t=0: (0, 0, 0)
    // At t=+-4: (16/8, +-4, 0) = (2, +-4, 0)
    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 2., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
}

#[test]
fn BndLibTest__Parabola_InfiniteParameter() {
    // Parabola with infinite parameter
    let aParab = Parabola3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_parab(&aParab, 0., f64::INFINITY, aTol, &mut aBox).unwrap();

    assert!(aBox.is_open_xmax(), "Xmax should be open for parabola with P2=+Inf");
    assert!(aBox.is_open_ymax(), "Ymax should be open for parabola with P2=+Inf");
}
