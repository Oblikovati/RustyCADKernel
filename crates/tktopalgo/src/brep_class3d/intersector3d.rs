// Rust translation of BRepClass3d_Intersector3d.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass3d_Intersector3d`: the first intersection of a line with a face.
use crate::brep_class::BRepClassFaceClassifier;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_curve_surface::{HInter, IntersectionPoint, TransitionOnCurve};
use rustycad_tkmath::gp::{Line3, Point2, Point3};

/// `BRepClass3d_Intersector3d`: the intersection point of smallest line parameter
/// (above `-Tol`) of a line with a face, IN or ON the face, with its transition
/// (reversed for a REVERSED face).
///
/// ```
/// use rustycad_tktopalgo::brep_class3d::BRepClass3dIntersector3d;
/// let intersector = BRepClass3dIntersector3d::new();
/// assert!(!intersector.is_done() && !intersector.has_a_point());
/// ```
#[derive(Clone, Debug)]
pub struct BRepClass3dIntersector3d {
    pnt: Point3,
    u: f64,
    v: f64,
    w: f64,
    transition: TransitionOnCurve,
    done: bool,
    hasapoint: bool,
    state: State,
    face: Face,
}

impl Default for BRepClass3dIntersector3d {
    /// `BRepClass3d_Intersector3d()`: not done, no point, UNKNOWN.
    fn default() -> Self {
        Self {
            pnt: Point3::default(),
            u: 0.0,
            v: 0.0,
            w: 0.0,
            transition: TransitionOnCurve::default(),
            done: false,
            hasapoint: false,
            state: State::Unknown,
            face: Face::default(),
        }
    }
}

/// The periodic surface bounds of the face: `(U1, U2, uperiod)` or `None`, and V.
type PeriodicRange = Option<(f64, f64, f64)>;

impl BRepClass3dIntersector3d {
    /// `BRepClass3d_Intersector3d()`; `BRepClass3dIntersector3d::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(L, Prm, Tol, F)`: intersects the whole line with the surface of the
    /// face, then keeps, among the points (from the last) whose UV point (moved into
    /// the surface period) is IN or ON the face with the 2d tolerance `tol`, the one of
    /// smallest parameter above `-tol`. `_prm` is unused natively. The results of an
    /// earlier Perform stay when no point is kept, and done is set only when the
    /// intersection has points; `intersector.perform(&line, 10.0, 1e-7, &face)?`.
    pub fn perform(
        &mut self,
        line: &Line3,
        _prm: f64,
        tol: f64,
        face: &Face,
    ) -> Result<(), Failure> {
        let geomline: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(*line)).cast();
        let hll: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&geomline)?).cast();
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        let periods = periodic_ranges(&surface)?;
        let hsurface: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
        let mut hics = HInter::new();
        hics.perform(&hll, &hsurface)?;
        self.w = f64::MAX;
        if !hics.is_done() {
            return Ok(());
        }
        let mut classifier2d = BRepClassFaceClassifier::new();
        for index in (1..=hics.nb_points()?).rev() {
            let point = hics.point(index)?;
            let puv = into_period(Point2::new(point.u(), point.v()), periods);
            classifier2d.perform(face, &puv, tol, false, 0.1)?;
            let current = classifier2d.state();
            if matches!(current, State::In | State::On) {
                self.keep_nearer(&point, current, tol, face);
            }
            self.done = true;
        }
        Ok(())
    }

    /// Keeps the point when its parameter is below the kept one and above `-tol`
    /// (OCC7966).
    fn keep_nearer(&mut self, point: &IntersectionPoint, current: State, tol: f64, face: &Face) {
        let hicsw: f64 = point.w();
        if !(self.w > hicsw && hicsw > -tol) {
            return;
        }
        self.hasapoint = true;
        self.u = point.u();
        self.v = point.v();
        self.w = hicsw;
        self.transition = point.transition();
        self.pnt = point.pnt();
        self.state = current;
        self.face = face.clone();
        if face.orientation() == Orientation::Reversed {
            self.transition = if self.transition == TransitionOnCurve::In {
                TransitionOnCurve::Out
            } else {
                TransitionOnCurve::In
            };
        }
    }

    /// `IsDone()`; `intersector.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `HasAPoint()`; `intersector.has_a_point()`.
    pub fn has_a_point(&self) -> bool {
        self.hasapoint
    }
    /// `UParameter()`: the U parameter on the surface; `intersector.u_parameter()`.
    pub fn u_parameter(&self) -> f64 {
        self.u
    }
    /// `VParameter()`; `intersector.v_parameter()`.
    pub fn v_parameter(&self) -> f64 {
        self.v
    }
    /// `WParameter()`: the parameter on the line; `intersector.w_parameter()`.
    pub fn w_parameter(&self) -> f64 {
        self.w
    }
    /// `Pnt()`; `intersector.pnt()`.
    pub fn pnt(&self) -> &Point3 {
        &self.pnt
    }
    /// `Transition()`; `intersector.transition()`.
    pub fn transition(&self) -> TransitionOnCurve {
        self.transition
    }
    /// `State()`: IN or ON the face; `intersector.state()`.
    pub fn state(&self) -> State {
        self.state
    }
    /// `Face()`: the face of the kept point; `intersector.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }
}

/// The U and V bounds and periods of a periodic surface direction.
fn periodic_ranges(
    surface: &BRepAdaptorSurface,
) -> Result<(PeriodicRange, PeriodicRange), Failure> {
    let uperiod = if surface.is_u_periodic()? {
        Some(surface.u_period()?)
    } else {
        None
    };
    let vperiod = if surface.is_v_periodic()? {
        Some(surface.v_period()?)
    } else {
        None
    };
    let (u1, u2) = (surface.first_u_parameter()?, surface.last_u_parameter()?);
    let (v1, v2) = (surface.first_v_parameter()?, surface.last_v_parameter()?);
    Ok((uperiod.map(|p| (u1, u2, p)), vperiod.map(|p| (v1, v2, p))))
}

/// The UV point shifted by whole periods (`RealToInt` of the quotient) when it lies
/// beyond the bounds of a periodic direction.
fn into_period(puv: Point2, (u, v): (PeriodicRange, PeriodicRange)) -> Point2 {
    Point2::new(shift(puv.x(), u), shift(puv.y(), v))
}

fn shift(x: f64, range: PeriodicRange) -> f64 {
    let Some((first, last, period)) = range else {
        return x;
    };
    let mut n: i32 = 0;
    if x > last {
        n = real_to_int((x - first) / period);
    }
    if x < first {
        n = real_to_int((x - last) / period);
    }
    x - period * f64::from(n)
}
