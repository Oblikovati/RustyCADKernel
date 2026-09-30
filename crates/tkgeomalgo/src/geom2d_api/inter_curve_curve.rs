// Rust translation of Geom2dAPI_InterCurveCurve.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAPI_InterCurveCurve`: the intersection points and overlap segments of two
//! 2D curves, or the self intersections of one curve.
use super::check_index;
use crate::geom2d_int::GInter;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::Point2;

/// The two trimmed curves of a segment.
pub type CurvePair = (Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>);

/// `Geom2dAPI_InterCurveCurve`: Geom2dInt_GInter on adaptors of the curves, with
/// the tolerance as both the confusion and the intersection tolerance; the curves
/// are copied for the segments.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let l1: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())).cast();
/// let l2: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(1.0, -1.0), Direction2::new(0.0, 1.0).unwrap())).cast();
/// let inter = InterCurveCurve::between(&l1, &l2, 1.0e-6).unwrap();
/// assert_eq!(inter.nb_points(), 1);
/// assert_eq!(inter.point(1).unwrap(), Point2::new(1.0, 0.0));
/// ```
pub struct InterCurveCurve {
    my_is_done: bool,
    my_curve1: Handle<dyn Geom2dCurve>,
    my_curve2: Handle<dyn Geom2dCurve>,
    my_intersector: GInter,
}

impl Default for InterCurveCurve {
    fn default() -> Self {
        Self::new()
    }
}

// Standard_NullObject for a null curve handle.
fn check_curve(curve: &Handle<dyn Geom2dCurve>, message: &str) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(Failure::with_type("Standard_NullObject", message, ""));
    }
    Ok(())
}

// `occ::down_cast<Geom2d_Curve>(C->Copy())`.
fn copy_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let Some(source) = curve.get() else {
        return Ok(Handle::default());
    };
    Ok(source.copy()?.cast())
}

impl InterCurveCurve {
    /// `Geom2dAPI_InterCurveCurve()`: not done. `InterCurveCurve::new()`.
    pub fn new() -> Self {
        Self {
            my_is_done: false,
            my_curve1: Handle::default(),
            my_curve2: Handle::default(),
            my_intersector: GInter::new(),
        }
    }

    /// `Geom2dAPI_InterCurveCurve(C1, C2, Tol)` (Tol 1.0e-6 natively by default).
    /// `InterCurveCurve::between(&c1, &c2, 1.0e-6)?`.
    pub fn between(
        c1: &Handle<dyn Geom2dCurve>,
        c2: &Handle<dyn Geom2dCurve>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.init(c1, c2, tol)?;
        Ok(inter)
    }

    /// `Geom2dAPI_InterCurveCurve(C1, Tol)`: the self intersections.
    /// `InterCurveCurve::of_self(&c1, 1.0e-6)?`.
    pub fn of_self(c1: &Handle<dyn Geom2dCurve>, tol: f64) -> Result<Self, Failure> {
        let mut inter: Self = Self::new();
        inter.init_self(c1, tol)?;
        Ok(inter)
    }

    /// `Init(C1, C2, Tol)`: Standard_NullObject for a null curve.
    /// `inter.init(&c1, &c2, 1.0e-6)?`.
    pub fn init(
        &mut self,
        c1: &Handle<dyn Geom2dCurve>,
        c2: &Handle<dyn Geom2dCurve>,
        tol: f64,
    ) -> Result<(), Failure> {
        check_curve(c1, "Geom2dAPI_InterCurveCurve::Init - C1 is null")?;
        check_curve(c2, "Geom2dAPI_InterCurveCurve::Init - C2 is null")?;
        self.my_curve1 = copy_curve(c1)?;
        self.my_curve2 = copy_curve(c2)?;
        let ac1 = Geom2dAdaptorCurve::with_curve(c1)?;
        let ac2 = Geom2dAdaptorCurve::with_curve(c2)?;
        self.my_intersector = GInter::between(&ac1, &ac2, tol, tol)?;
        self.my_is_done = self.my_intersector.is_done();
        Ok(())
    }

    /// `Init(C1, Tol)`: the self intersections; Standard_NullObject for a null
    /// curve. `inter.init_self(&c1, 1.0e-6)?`.
    pub fn init_self(&mut self, c1: &Handle<dyn Geom2dCurve>, tol: f64) -> Result<(), Failure> {
        check_curve(c1, "Geom2dAPI_InterCurveCurve::Init - C1 is null")?;
        self.my_curve1 = copy_curve(c1)?;
        self.my_curve2.nullify();
        let ac1 = Geom2dAdaptorCurve::with_curve(c1)?;
        self.my_intersector = GInter::of_self(&ac1, tol, tol)?;
        self.my_is_done = self.my_intersector.is_done();
        Ok(())
    }

    /// `NbPoints()`: 0 when the intersection is not done. `inter.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        if !self.my_is_done {
            return 0;
        }
        // The intersector is done, so it cannot raise StdFail_NotDone.
        self.my_intersector.nb_points().unwrap_or(0)
    }

    /// `Point(Index)`: Standard_OutOfRange outside `1..=nb_points()`.
    /// `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point2, Failure> {
        check_index(index, self.nb_points(), "Geom2dAPI_InterCurveCurve::Point")?;
        Ok(self.my_intersector.point(index)?.value())
    }

    /// `NbSegments()`: 0 when the intersection is not done. `inter.nb_segments()`.
    pub fn nb_segments(&self) -> i32 {
        if !self.my_is_done {
            return 0;
        }
        // The intersector is done, so it cannot raise StdFail_NotDone.
        self.my_intersector.nb_segments().unwrap_or(0)
    }

    /// `Segment(Index, Curve1, Curve2)`: the copies of both curves (the first one
    /// twice for a self intersection) trimmed to the segment, the second one's
    /// bounds swapped when the curves run opposite; bounds the segment lacks stay
    /// those of the curve. Standard_OutOfRange outside `1..=nb_segments()`.
    /// `let (c1, c2) = inter.segment(1)?;`.
    pub fn segment(&self, the_index: i32) -> Result<CurvePair, Failure> {
        check_index(
            the_index,
            self.nb_segments(),
            "Geom2dAPI_InterCurveCurve::Segment",
        )?;
        let Some(curve1) = self.my_curve1.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "Geom2dAPI_InterCurveCurve::Segment",
                "",
            ));
        };
        let mut a_u1: f64 = curve1.first_parameter();
        let mut a_u2: f64 = curve1.last_parameter();
        let (mut a_v1, mut a_v2) = match self.my_curve2.get() {
            None => (a_u1, a_u2),
            Some(curve2) => (curve2.first_parameter(), curve2.last_parameter()),
        };
        let a_seg = self.my_intersector.segment(the_index)?;
        let is_opposite: bool = a_seg.is_opposite();
        if a_seg.has_first_point() {
            let an_ipf = a_seg.first_point()?;
            a_u1 = an_ipf.param_on_first();
            if is_opposite {
                a_v2 = an_ipf.param_on_second();
            } else {
                a_v1 = an_ipf.param_on_second();
            }
        }
        if a_seg.has_last_point() {
            let an_ipl = a_seg.last_point()?;
            a_u2 = an_ipl.param_on_first();
            if is_opposite {
                a_v1 = an_ipl.param_on_second();
            } else {
                a_v2 = an_ipl.param_on_second();
            }
        }
        let the_curve1 = trimmed(&self.my_curve1, a_u1, a_u2)?;
        let basis2 = if self.my_curve2.is_null() {
            &self.my_curve1
        } else {
            &self.my_curve2
        };
        let the_curve2 = trimmed(basis2, a_v1, a_v2)?;
        Ok((the_curve1, the_curve2))
    }

    /// `Intersector()`: the underlying Geom2dInt_GInter. `inter.intersector()`.
    pub fn intersector(&self) -> &GInter {
        &self.my_intersector
    }
}

// `new Geom2d_TrimmedCurve(C, U1, U2)` with the default sense and periodic
// adjustment.
fn trimmed(
    curve: &Handle<dyn Geom2dCurve>,
    u1: f64,
    u2: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, u1, u2, true, true)?).cast())
}
