// Helper: create a linear BSpline curve from two points, parametrized on [0,1].
fn makeLinearBSpline(theP1: Point3, theP2: Point3) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = theP1;
    aPoles[2] = theP2;
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 2;
    aMults[2] = 2;
    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 1, false).unwrap())
}

// Helper: create a quadratic BSpline curve through 3 control points on [0,1].
fn makeQuadraticBSpline(theP1: Point3, theP2: Point3, theP3: Point3) -> Handle<GeomBSplineCurve> {
    makeQuadraticBSplineOn(theP1, theP2, theP3, 0.0, 1.0)
}

fn makeQuadraticBSplineOn(
    theP1: Point3,
    theP2: Point3,
    theP3: Point3,
    theFirstParameter: f64,
    theLastParameter: f64,
) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = theP1;
    aPoles[2] = theP2;
    aPoles[3] = theP3;
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = theFirstParameter;
    aKnots[2] = theLastParameter;
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 3;
    aMults[2] = 3;
    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 2, false).unwrap())
}

fn makeIntersectionGrid(
    theProfiles: &Array1<'_, Handle<GeomBSplineCurve>>,
    theGuideParams: &Array1<'_, f64>,
) -> Array2<'static, Point3> {
    let mut aPoints: Array2<'static, Point3> =
        Array2::with_bounds(1, theGuideParams.len() as i32, 1, theProfiles.len() as i32).unwrap();
    for aProfileIdx in 0..theProfiles.len() as i32 {
        for aGuideIdx in 0..theGuideParams.len() as i32 {
            let profile = theProfiles[theProfiles.lower() + aProfileIdx].get().unwrap();
            let param = theGuideParams[theGuideParams.lower() + aGuideIdx];
            aPoints[(aGuideIdx + 1, aProfileIdx + 1)] = profile.value(param).unwrap();
        }
    }
    aPoints
}

fn makeUnitWeightGrid(
    theProfiles: &Array1<'_, Handle<GeomBSplineCurve>>,
    theGuideParams: &Array1<'_, f64>,
) -> Array2<'static, f64> {
    let mut aWeights: Array2<'static, f64> =
        Array2::with_bounds(1, theGuideParams.len() as i32, 1, theProfiles.len() as i32).unwrap();
    aWeights.init(1.0);
    aWeights
}

fn makeRationalQuadraticBSpline(
    theP1: Point3,
    theP2: Point3,
    theP3: Point3,
    theMiddleWeight: f64,
) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = theP1;
    aPoles[2] = theP2;
    aPoles[3] = theP3;
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = 1.0;
    aWeights[2] = theMiddleWeight;
    aWeights[3] = 1.0;
    let (aKnots, aMults) = end_knots(3);
    Handle::new(GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap())
}

// The knots 0 and 1 with multiplicity `m` at both ends.
fn end_knots(m: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = m;
    aMults[2] = m;
    (aKnots, aMults)
}

fn makeWeightedLinearBSpline(theP1: Point3, theP2: Point3, theWeight: f64) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = theP1;
    aPoles[2] = theP2;
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aWeights[1] = theWeight;
    aWeights[2] = theWeight;
    let (aKnots, aMults) = end_knots(2);
    // `Geom_BSplineCurve(Poles, Weights, Knots, Mults, 1, false, false)`: CheckRational off.
    let curve = GeomBSplineCurve::with_weights_checked(&aPoles, &aWeights, &aKnots, &aMults, 1, false, false);
    Handle::new(curve.unwrap())
}

fn makeRationalLineBSpline(theP1: Point3, theP2: Point3, theDegree: i32) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, theDegree + 1).unwrap();
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, theDegree + 1).unwrap();
    for aPoleIdx in 1..=theDegree + 1 {
        let aRatio = f64::from(aPoleIdx - 1) / f64::from(theDegree);
        let xyz = theP1.xyz() * (1.0 - aRatio) + theP2.xyz() * aRatio;
        aPoles[aPoleIdx] = Point3::new(xyz.x(), xyz.y(), xyz.z());
        aWeights[aPoleIdx] = if aPoleIdx == (theDegree + 2) / 2 { 0.5 } else { 1.0 };
    }
    let (aKnots, aMults) = end_knots(theDegree + 1);
    Handle::new(GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, theDegree, false).unwrap())
}

fn makeRationalQuadraticLineBSpline(
    theP1: Point3,
    theP2: Point3,
    theStartWeight: f64,
    theMiddleWeight: f64,
    theEndWeight: f64,
) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = theP1;
    let mid = (theP1.xyz() + theP2.xyz()) * 0.5;
    aPoles[2] = Point3::new(mid.x(), mid.y(), mid.z());
    aPoles[3] = theP2;
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = theStartWeight;
    aWeights[2] = theMiddleWeight;
    aWeights[3] = theEndWeight;
    let (aKnots, aMults) = end_knots(3);
    Handle::new(GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap())
}

// The periodic cubic of `makePeriodicProfile` and `makePeriodicGuide` on the poles.
fn periodic_cubic(aPoles: &Array1<'static, Point3>) -> Handle<GeomBSplineCurve> {
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    for aKnotIdx in 1..=6 {
        aKnots[aKnotIdx] = 0.2 * f64::from(aKnotIdx - 1);
    }
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 6).unwrap();
    aMults.init(1);
    Handle::new(GeomBSplineCurve::new(aPoles, &aKnots, &aMults, 3, true).unwrap())
}

fn makePeriodicProfile(theY: f64) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 5).unwrap();
    aPoles[1] = Point3::new(1.0, theY, 0.0);
    aPoles[2] = Point3::new(0.309, theY, 0.951);
    aPoles[3] = Point3::new(-0.809, theY, 0.588);
    aPoles[4] = Point3::new(-0.809, theY, -0.588);
    aPoles[5] = Point3::new(0.309, theY, -0.951);
    periodic_cubic(&aPoles)
}

fn makePeriodicGuide(theX: f64) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 5).unwrap();
    aPoles[1] = Point3::new(theX, 1.0, 0.0);
    aPoles[2] = Point3::new(theX, 0.309, 0.951);
    aPoles[3] = Point3::new(theX, -0.809, 0.588);
    aPoles[4] = Point3::new(theX, -0.809, -0.588);
    aPoles[5] = Point3::new(theX, 0.309, -0.951);
    periodic_cubic(&aPoles)
}

fn makeClosedBSplineProfile(theY: f64, theXRadius: f64, theZRadius: f64) -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 9).unwrap();
    aPoles[1] = Point3::new(theXRadius, theY, 0.0);
    aPoles[2] = Point3::new(theXRadius, theY, 0.7 * theZRadius);
    aPoles[3] = Point3::new(0.3 * theXRadius, theY, theZRadius);
    aPoles[4] = Point3::new(-0.7 * theXRadius, theY, 0.8 * theZRadius);
    aPoles[5] = Point3::new(-theXRadius, theY, 0.0);
    aPoles[6] = Point3::new(-0.7 * theXRadius, theY, -0.8 * theZRadius);
    aPoles[7] = Point3::new(0.3 * theXRadius, theY, -theZRadius);
    aPoles[8] = Point3::new(theXRadius, theY, -0.7 * theZRadius);
    aPoles[9] = Point3::new(theXRadius, theY, 0.0);
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 7).unwrap();
    for aKnotIdx in 1..=7 {
        aKnots[aKnotIdx] = f64::from(aKnotIdx - 1) / 6.0;
    }
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 7).unwrap();
    aMults[1] = 4;
    aMults[7] = 4;
    for aKnotIdx in 2..7 {
        aMults[aKnotIdx] = 1;
    }
    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// `GeomAPI_Interpolate(Points, [Parameters,] Periodic, Precision::Confusion())`, then
// Perform and Curve.
fn interpolated(points: Vec<Point3>, params: Option<Vec<f64>>, periodic: bool) -> Handle<GeomBSplineCurve> {
    let mut aPoints: Array1<'static, Point3> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, p) in points.iter().enumerate() {
        aPoints[i as i32 + 1] = *p;
    }
    let aPnts = Handle::new(HArray1::from_owned(aPoints));
    let mut anInterp = match params {
        Some(params) => {
            let mut aParams: Array1<'static, f64> = Array1::with_bounds(1, params.len() as i32).unwrap();
            for (i, t) in params.iter().enumerate() {
                aParams[i as i32 + 1] = *t;
            }
            let aParams = Handle::new(HArray1::from_owned(aParams));
            Interpolate::with_parameters(&aPnts, &aParams, periodic, precision::confusion()).unwrap()
        }
        None => Interpolate::new(&aPnts, periodic, precision::confusion()).unwrap(),
    };
    anInterp.perform().unwrap();
    anInterp.curve().unwrap()
}

fn makeInterpolatedPeriodicProfile(theY: f64, theSeamShift: i32) -> Handle<GeomBSplineCurve> {
    const THE_NB_POINTS: i32 = 8;
    const THE_PI: f64 = 3.14159265358979323846;
    let points: Vec<Point3> = (0..THE_NB_POINTS)
        .map(|aPointIdx| {
            let anAngle = 2.0 * THE_PI * f64::from(aPointIdx + theSeamShift) / f64::from(THE_NB_POINTS);
            Point3::new(anAngle.cos(), theY, anAngle.sin())
        })
        .collect();
    interpolated(points, None, true)
}

// Helper: create a cubic BSpline through 4 interpolation points on [0,1].
fn makeCubicInterpBSpline(theP1: Point3, theP2: Point3, theP3: Point3, theP4: Point3) -> Handle<GeomBSplineCurve> {
    interpolated(vec![theP1, theP2, theP3, theP4], Some(vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]), false)
}

// Helper: create a sinusoidal BSpline curve via interpolation.
// X varies linearly, Z = amplitude * sin(pi * t), Y = constY.
fn makeSineBSpline(theXStart: f64, theXEnd: f64, theY: f64, theAmplitude: f64) -> Handle<GeomBSplineCurve> {
    makeSineBSplineN(theXStart, theXEnd, theY, theAmplitude, 11)
}

fn makeSineBSplineN(theXStart: f64, theXEnd: f64, theY: f64, theAmplitude: f64, theNbPts: i32) -> Handle<GeomBSplineCurve> {
    let (mut points, mut params) = (Vec::new(), Vec::new());
    for i in 1..=theNbPts {
        let aT = f64::from(i - 1) / f64::from(theNbPts - 1);
        let aX = theXStart + aT * (theXEnd - theXStart);
        let aZ = theAmplitude * (PI * aT).sin();
        points.push(Point3::new(aX, theY, aZ));
        params.push(aT);
    }
    interpolated(points, Some(params), false)
}

fn makeMeanderingProfile(theY: f64) -> Handle<GeomBSplineCurve> {
    let points = vec![
        Point3::new(0.0, theY, 0.0),
        Point3::new(0.62, theY + 0.04, 0.01),
        Point3::new(0.38, theY - 0.04, -0.01),
        Point3::new(0.5, theY, 0.0),
        Point3::new(0.64, theY + 0.03, 0.01),
        Point3::new(0.44, theY - 0.02, -0.01),
        Point3::new(1.0, theY, 0.0),
    ];
    let params: Vec<f64> = (1..=7).map(|aParamIdx| f64::from(aParamIdx - 1) / 6.0).collect();
    interpolated(points, Some(params), false)
}

// Helper: verify surface passes through a point at given parameters.
fn verifyPointOnSurface(theSurf: &Handle<GeomBSplineSurface>, theU: f64, theV: f64, theExpected: Point3, theTol: f64) {
    let aSurfPt = theSurf.value(theU, theV).unwrap();
    let aDist = aSurfPt.distance(&theExpected);
    assert!(aDist < theTol, "Surface point at ({theU}, {theV}) = {aSurfPt:?} differs from expected {theExpected:?} by {aDist}");
}

type HeightFunction = fn(f64, f64) -> f64;

fn wavyHeight(theU: f64, theV: f64) -> f64 {
    0.35 * (PI * theU).sin() * (0.4 + theV) + 0.12 * (2.0 * PI * theV).sin() * (1.0 - theU)
}

fn saddleHeight(theU: f64, theV: f64) -> f64 {
    0.6 * (theU - 0.5) * (theV - 0.5) + 0.15 * (PI * theU).sin() * (PI * theV).sin()
}

fn rippleHeight(theU: f64, theV: f64) -> f64 {
    0.2 * (2.0 * PI * theU).sin() * (PI * theV).cos() + 0.1 * (3.0 * PI * theV).sin()
}

struct AnalyticNetwork {
    Profiles: Array1<'static, Handle<dyn GeomCurve>>,
    Guides: Array1<'static, Handle<dyn GeomCurve>>,
}

impl AnalyticNetwork {
    fn new(theNbProfiles: usize, theNbGuides: usize, theHeight: HeightFunction) -> Self {
        Self::with_frame(theNbProfiles, theNbGuides, theHeight, Xyz::new(0.0, 0.0, 0.0), 1.0)
    }

    fn with_frame(theNbProfiles: usize, theNbGuides: usize, theHeight: HeightFunction, theOrigin: Xyz, theScale: f64) -> Self {
        let point = |aU: f64, aV: f64| {
            let xyz = theOrigin + Xyz::new(theScale * aU, theScale * aV, theScale * theHeight(aU, aV));
            Point3::new(xyz.x(), xyz.y(), xyz.z())
        };
        let mut Profiles: Array1<'static, Handle<dyn GeomCurve>> = Array1::with_bounds(1, theNbProfiles as i32).unwrap();
        for aProfileIdx in 0..theNbProfiles {
            let aV = aProfileIdx as f64 / (theNbProfiles - 1) as f64;
            let (mut points, mut params) = (Vec::new(), Vec::new());
            for aGuideIdx in 0..theNbGuides {
                let aU = aGuideIdx as f64 / (theNbGuides - 1) as f64;
                points.push(point(aU, aV));
                params.push(aU);
            }
            Profiles[aProfileIdx as i32 + 1] = interpolated(points, Some(params), false).cast();
        }
        let mut Guides: Array1<'static, Handle<dyn GeomCurve>> = Array1::with_bounds(1, theNbGuides as i32).unwrap();
        for aGuideIdx in 0..theNbGuides {
            let aU = aGuideIdx as f64 / (theNbGuides - 1) as f64;
            let (mut points, mut params) = (Vec::new(), Vec::new());
            for aProfileIdx in 0..theNbProfiles {
                let aV = aProfileIdx as f64 / (theNbProfiles - 1) as f64;
                points.push(point(aU, aV));
                params.push(aV);
            }
            Guides[aGuideIdx as i32 + 1] = interpolated(points, Some(params), false).cast();
        }
        Self { Profiles, Guides }
    }
}

fn verifyUniformNetworkInterpolation(theSurface: &Handle<GeomBSplineSurface>, theNetwork: &AnalyticNetwork, theTolerance: f64) {
    assert!(!theSurface.is_null());
    const THE_NB_SAMPLES: usize = 32;
    let s = theSurface.get().unwrap();
    let aUMin = s.u_knot(1).unwrap();
    let aUMax = s.u_knot(s.nb_u_knots()).unwrap();
    let aVMin = s.v_knot(1).unwrap();
    let aVMax = s.v_knot(s.nb_v_knots()).unwrap();
    let nb_p = theNetwork.Profiles.len();
    for aProfileIdx in 0..nb_p {
        let aV = aVMin + aProfileIdx as f64 / (nb_p - 1) as f64 * (aVMax - aVMin);
        let profile = &theNetwork.Profiles[aProfileIdx as i32 + 1];
        for aSampleIdx in 0..=THE_NB_SAMPLES {
            let aParam = aSampleIdx as f64 / THE_NB_SAMPLES as f64;
            let aU = aUMin + aParam * (aUMax - aUMin);
            let distance = s.value(aU, aV).unwrap().distance(&profile.value(aParam).unwrap());
            assert!(distance <= theTolerance, "{distance} > {theTolerance}");
        }
    }
    let nb_g = theNetwork.Guides.len();
    for aGuideIdx in 0..nb_g {
        let aU = aUMin + aGuideIdx as f64 / (nb_g - 1) as f64 * (aUMax - aUMin);
        let guide = &theNetwork.Guides[aGuideIdx as i32 + 1];
        for aSampleIdx in 0..=THE_NB_SAMPLES {
            let aParam = aSampleIdx as f64 / THE_NB_SAMPLES as f64;
            let aV = aVMin + aParam * (aVMax - aVMin);
            let distance = s.value(aU, aV).unwrap().distance(&guide.value(aParam).unwrap());
            assert!(distance <= theTolerance, "{distance} > {theTolerance}");
        }
    }
}

// `NCollection_Array1<occ::handle<Geom_Curve>>` of the given curves.
fn curves(list: &[Handle<GeomBSplineCurve>]) -> Array1<'static, Handle<dyn GeomCurve>> {
    let mut array: Array1<'static, Handle<dyn GeomCurve>> = Array1::with_bounds(1, list.len() as i32).unwrap();
    for (i, c) in list.iter().enumerate() {
        array[i as i32 + 1] = c.cast();
    }
    array
}

// `NCollection_Array1<occ::handle<Geom_BSplineCurve>>` of the given curves.
fn bsplines(list: &[Handle<GeomBSplineCurve>]) -> Array1<'static, Handle<GeomBSplineCurve>> {
    let mut array: Array1<'static, Handle<GeomBSplineCurve>> = Array1::with_bounds(1, list.len() as i32).unwrap();
    for (i, c) in list.iter().enumerate() {
        array[i as i32 + 1] = c.clone();
    }
    array
}

// `NCollection_Array1<double>` of the given values.
fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, v) in values.iter().enumerate() {
        array[i as i32 + 1] = *v;
    }
    array
}

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right} within {tolerance}");
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}
