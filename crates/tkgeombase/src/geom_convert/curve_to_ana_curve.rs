// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CurveToAnaCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_CurveToAnaCurve`: the recognition of a line, a circle or an ellipse
//! in an arbitrary curve, within a tolerance.
use super::ConvType;
use super::curve_to_ana_conic::{compute_circle, compute_ellipse, get_circle};
use crate::geom_lib::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Direction3, Line3, Point3, Vector3};

/// The in-out outputs `cf`, `cl` and `Deviation` (or `theGap`) of the static
/// recognizers: the parameter range of the result on the recognized curve and its
/// greatest distance to the input. A field keeps its value when the native member
/// leaves the output untouched.
///
/// `let mut fit = AnaCurveFit::new(0.0, 1.0);`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnaCurveFit {
    /// `cf`: the first parameter on the result.
    pub first: f64,
    /// `cl`: the last parameter on the result.
    pub last: f64,
    /// `Deviation` or `theGap`: the greatest distance of the input to the result.
    pub deviation: f64,
}

impl AnaCurveFit {
    /// The outputs before a call: the range `[first, last]` and a zero deviation;
    /// `AnaCurveFit::new(0.0, 1.0)`.
    pub fn new(first: f64, last: f64) -> Self {
        Self {
            first,
            last,
            deviation: 0.0,
        }
    }

    // Writes the range (the native `cf = c1; cl = c2`).
    fn set_range(&mut self, first: f64, last: f64) {
        self.first = first;
        self.last = last;
    }
}

/// `GeomConvert_CurveToAnaCurve`: converts a curve into the analytic curve (line,
/// circle, ellipse) chosen by the conversion type among the candidates within the
/// tolerance.
///
/// `let mut conv = CurveToAnaCurve::with_curve(&curve);`
#[derive(Clone)]
pub struct CurveToAnaCurve {
    my_curve: Handle<dyn GeomCurve>,
    my_gap: f64,
    my_conv_type: ConvType,
    my_target: CurveType,
}

impl Default for CurveToAnaCurve {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveToAnaCurve {
    /// The empty converter: gap Precision::Infinite(), MinGap, target line;
    /// `CurveToAnaCurve::new()`.
    pub fn new() -> Self {
        Self {
            my_curve: Handle::default(),
            my_gap: precision::infinite(),
            my_conv_type: ConvType::MinGap,
            my_target: CurveType::Line,
        }
    }

    /// The converter of `curve`; `CurveToAnaCurve::with_curve(&curve)`.
    pub fn with_curve(curve: &Handle<dyn GeomCurve>) -> Self {
        let mut conv = Self::new();
        conv.my_curve = curve.clone();
        conv
    }

    /// `Init(C)`: a new curve, the gap reset to Precision::Infinite() (the type and
    /// target are kept); `conv.init(&curve)`.
    pub fn init(&mut self, curve: &Handle<dyn GeomCurve>) {
        self.my_curve = curve.clone();
        self.my_gap = precision::infinite();
    }

    /// `Gap()`: the deviation of the last conversion; `conv.gap()`.
    pub fn gap(&self) -> f64 {
        self.my_gap
    }

    /// `GetConvType()`; `conv.get_conv_type()`.
    pub fn get_conv_type(&self) -> ConvType {
        self.my_conv_type
    }

    /// `SetConvType(theConvType)`; `conv.set_conv_type(ConvType::Simplest)`.
    pub fn set_conv_type(&mut self, conv_type: ConvType) {
        self.my_conv_type = conv_type;
    }

    /// `GetTarget()`; `conv.get_target()`.
    pub fn get_target(&self) -> CurveType {
        self.my_target
    }

    /// `SetTarget(theTarget)`: the curve type of ConvType::Target;
    /// `conv.set_target(CurveType::Circle)`.
    pub fn set_target(&mut self, target: CurveType) {
        self.my_target = target;
    }

    /// `ConvertToAnalytical(theTol, theResultCurve, F, L, NewF, NewL)`: recognizes
    /// the basis of the (trimmed) curve on `[F, L]`; true with `result` and
    /// `new_range` set on success. `new_range` is written as soon as a curve is set,
    /// also on failure; `conv.convert_to_analytical(1e-7, &mut res, (0.0, 1.0), &mut range)?`.
    pub fn convert_to_analytical(
        &mut self,
        tol: f64,
        result: &mut Handle<dyn GeomCurve>,
        (first, last): (f64, f64),
        new_range: &mut (f64, f64),
    ) -> Result<bool, Failure> {
        if self.my_curve.is_null() {
            return Ok(false);
        }
        let mut curve: Handle<dyn GeomCurve> = self.my_curve.clone();
        while let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            curve = trimmed.basis_curve();
        }
        let mut fit = AnaCurveFit {
            first: new_range.0,
            last: new_range.1,
            deviation: self.my_gap,
        };
        let computed = Self::compute_curve(
            &curve,
            tol,
            (first, last),
            &mut fit,
            self.my_conv_type,
            self.my_target,
        );
        *new_range = (fit.first, fit.last);
        self.my_gap = fit.deviation;
        let converted: Handle<dyn GeomCurve> = computed?;
        if converted.is_null() {
            return Ok(false);
        }
        *result = converted;
        Ok(true)
    }

    /// `IsLinear(aPoints, tolerance, Deviation)`: Some(deviation) when every point
    /// is within `tolerance` of the line through the two farthest points; None for
    /// fewer than 2 points, confused points or a point too far (the native false,
    /// Deviation untouched); `CurveToAnaCurve::is_linear(&points, 1e-7)?`.
    pub fn is_linear(points: &Array1<'_, Point3>, tolerance: f64) -> Result<Option<f64>, Failure> {
        let poles: &[Point3] = points.as_slice();
        if poles.len() < 2 {
            return Ok(None);
        }
        let (d_max, i_max1, i_max2): (f64, usize, usize) = farthest_pair(poles);
        if d_max < precision::square_confusion() {
            return Ok(None);
        }
        let tol2: f64 = tolerance * tolerance;
        let adir = Direction3::from_vector(Vector3::between(poles[i_max1], poles[i_max2]))?;
        let alin = Line3::new(poles[i_max1], adir);
        let mut a_max: f64 = 0.0;
        for pole in poles {
            let dist: f64 = alin.square_distance(*pole);
            if dist > tol2 {
                return Ok(None);
            }
            if dist > a_max {
                a_max = dist;
            }
        }
        Ok(Some(a_max.sqrt()))
    }

    /// `GetLine(P1, P2, cf, cl)`: the line from `p1` towards `p2` and the parameters
    /// of both points; Standard_ConstructionError for confused points;
    /// `CurveToAnaCurve::get_line(a, b)?`.
    pub fn get_line(p1: Point3, p2: Point3) -> Result<(Line3, f64, f64), Failure> {
        let adir = Direction3::from_vector(Vector3::between(p1, p2))?;
        let alin = Line3::new(p1, adir);
        let cf: f64 = elclib::parameter(&alin, p1)?;
        let cl: f64 = elclib::parameter(&alin, p2)?;
        Ok((alin, cf, cl))
    }

    /// `GetCircle(Circ, P0, P1, P2)`: the circle through three points, its X axis
    /// towards `p0`; false for a coordinate beyond sqrt(Precision::Infinite()), no
    /// circle, or a radius below gp::Resolution(). `circ` is written as soon as the
    /// circle is built, as natively; `CurveToAnaCurve::get_circle(&mut c, a, b, d)?`.
    pub fn get_circle(
        circ: &mut Circle3,
        p0: Point3,
        p1: Point3,
        p2: Point3,
    ) -> Result<bool, Failure> {
        get_circle(circ, p0, p1, p2)
    }

    /// `ComputeLine(curve, tolerance, c1, c2, cf, cl, Deviation)`: the line of a
    /// linear curve on `[c1, c2]` (a Geom_Line is returned itself), tested on the
    /// B-spline or Bezier poles, or on 23 samples; null otherwise. The range is
    /// written once the end points differ, the deviation on success only;
    /// `CurveToAnaCurve::compute_line(&curve, 1e-7, (0.0, 1.0), &mut fit)?`.
    pub fn compute_line(
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
        (c1, c2): (f64, f64),
        fit: &mut AnaCurveFit,
    ) -> Result<Handle<GeomLine>, Failure> {
        if curve.is_null() {
            return Ok(Handle::default());
        }
        let line: Handle<GeomLine> = curve.cast::<GeomLine>();
        if line.is_some() {
            fit.set_range(c1, c2);
            fit.deviation = 0.0;
            return Ok(line);
        }
        let object: &dyn GeomCurve = live(curve, "GeomConvert_CurveToAnaCurve::ComputeLine")?;
        let p1: Point3 = object.value(c1)?;
        let p2: Point3 = object.value(c2)?;
        if p1.square_distance(&p2) < precision::square_confusion() {
            return Ok(Handle::default());
        }
        fit.set_range(c1, c2);
        let poles: Array1<'static, Point3> = line_test_points(curve, object, (c1, c2), (p1, p2))?;
        let Some(deviation) = Self::is_linear(&poles, tolerance)? else {
            return Ok(Handle::default());
        };
        fit.deviation = deviation;
        let (alin, cf, cl) = Self::get_line(p1, p2)?;
        fit.set_range(cf, cl);
        Ok(Handle::new(GeomLine::from_lin(alin)))
    }

    /// `ComputeCircle(curve, tolerance, c1, c2, cf, cl, Deviation)`: the circle
    /// through the points at c1, c1 + (c2-c1)/3 and c1 + 2(c2-c1)/3 when 21 samples
    /// are within `tolerance` (a Geom_Circle is returned itself); null otherwise.
    /// A null curve is Standard_NullObject;
    /// `CurveToAnaCurve::compute_circle(&curve, 1e-7, (0.0, 1.0), &mut fit)?`.
    pub fn compute_circle(
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
        range: (f64, f64),
        fit: &mut AnaCurveFit,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        compute_circle(curve, tolerance, range, fit)
    }

    /// `ComputeEllipse(curve, tolerance, c1, c2, cf, cl, Deviation)`: the ellipse of
    /// the conic through 5 planar samples when 20 samples are within `tolerance` (a
    /// Geom_Ellipse is returned itself); null for a non-planar, non-elliptic or
    /// circular fit. A null curve is Standard_NullObject;
    /// `CurveToAnaCurve::compute_ellipse(&curve, 1e-7, (0.0, 1.0), &mut fit)?`.
    pub fn compute_ellipse(
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
        range: (f64, f64),
        fit: &mut AnaCurveFit,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        compute_ellipse(curve, tolerance, range, fit)
    }
}

// The index pair (zero-based) of the farthest points (the first of equal square
// distances) and their square distance.
fn farthest_pair(poles: &[Point3]) -> (f64, usize, usize) {
    let (mut d_max, mut i_max1, mut i_max2): (f64, usize, usize) = (0.0, 0, 0);
    for i in 0..poles.len() - 1 {
        for j in i + 1..poles.len() {
            let dist: f64 = poles[i].square_distance(&poles[j]);
            if dist > d_max {
                (d_max, i_max1, i_max2) = (dist, i, j);
            }
        }
    }
    (d_max, i_max1, i_max2)
}

// The poles of a B-spline or Bezier curve, or 23 samples of `[c1, c2]` otherwise.
fn line_test_points(
    curve: &Handle<dyn GeomCurve>,
    object: &dyn GeomCurve,
    (c1, c2): (f64, f64),
    (p1, p2): (Point3, Point3),
) -> Result<Array1<'static, Point3>, Failure> {
    if let Some(bspline) = curve.cast::<GeomBSplineCurve>().get() {
        return Ok(bspline.poles());
    }
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        return Ok(bezier.poles());
    }
    let nb_poles: i32 = 23;
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, nb_poles)
        .map_err(|error| crate::native_failure::array_failure(error, "Poles"))?;
    let dt: f64 = (c2 - c1) / f64::from(nb_poles - 1);
    poles[1] = p1;
    poles[nb_poles] = p2;
    for i in 2..nb_poles {
        poles[i] = object.value(c1 + f64::from(i - 1) * dt)?;
    }
    Ok(poles)
}
