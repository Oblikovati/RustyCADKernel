// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_HelixCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_HelixCurve`: the analytic helix as an `Adaptor3d_Curve`, with its point
//! and first two derivatives in closed form.
use super::errors::{array_failure, construction_error, domain_error, not_implemented};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurveD1, GeomCurveD2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The protected fields of the native class.
#[derive(Clone, Copy, Debug)]
struct HelixCurveState {
    first: f64,
    last: f64,
    pitch: f64,
    r_start: f64,
    taper_angle: f64,
    is_clock_wise: bool,
    c1: f64,
    tg_beta: f64,
    tol_angle: f64,
}

impl Default for HelixCurveState {
    // The default constructor: one clockwise turn of pitch 1 and radius 1.
    fn default() -> Self {
        let last: f64 = 2. * PI;
        let pitch: f64 = 1.;
        Self {
            first: 0.,
            last,
            pitch,
            r_start: 1.,
            taper_angle: 0.,
            is_clock_wise: true,
            c1: pitch / last,
            tg_beta: 0.,
            tol_angle: 1.0e-4,
        }
    }
}

/// The helix parameters of `Load` and of the builders' `CurveParameters`: range, pitch,
/// start radius, taper angle and sense.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelixParameters {
    /// Start parameter (angle in radians).
    pub t1: f64,
    /// End parameter (angle in radians).
    pub t2: f64,
    /// Rise per turn (2*PI radians).
    pub pitch: f64,
    /// Radius at parameter 0.
    pub r_start: f64,
    /// Taper angle in radians (0 for a cylindrical helix).
    pub taper_angle: f64,
    /// Clockwise when true (the native `aIsCW`).
    pub is_cw: bool,
}

/// `HelixGeom_HelixCurve`: `C(t) = (R(t) cos t, ±R(t) sin t, c1 t)` with
/// `c1 = pitch / 2*PI` and `R(t) = RStart + c1 tan(taper) t` (docs/helix-design.md
/// section 1).
///
/// ```
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// use rustycad_tkhelix::helix_geom::HelixGeomHelixCurve;
/// let helix = HelixGeomHelixCurve::new();
/// helix.load(0.0, 4.0 * std::f64::consts::PI, 10.0, 5.0, 0.0, true)?;
/// assert_eq!(helix.value(0.0)?.x(), 5.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct HelixGeomHelixCurve {
    parent: StandardTransient,
    state: SharedState<HelixCurveState>,
}
impl_transient!(
    HelixGeomHelixCurve,
    "HelixGeom_HelixCurve",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl Default for HelixGeomHelixCurve {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HelixGeomHelixCurve {
    // The copy constructor: a new object with the same field values.
    fn clone(&self) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(self.state.snapshot()),
        }
    }
}

impl HelixGeomHelixCurve {
    /// `HelixGeom_HelixCurve()`: one clockwise turn `[0, 2*PI]` of pitch 1 and radius 1,
    /// untapered; `HelixGeomHelixCurve::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(HelixCurveState::default()),
        }
    }

    /// `Load()`: reloads the current parameters, so it raises exactly when they are
    /// invalid (after a failed `Load`, they are); `helix.load_current()?`.
    pub fn load_current(&self) -> Result<(), Failure> {
        let s: HelixCurveState = *self.state.read();
        self.load(
            s.first,
            s.last,
            s.pitch,
            s.r_start,
            s.taper_angle,
            s.is_clock_wise,
        )
    }

    /// `Load(aT1, aT2, aPitch, aRStart, aTaperAngle, aIsCW)`: stores the parameters,
    /// then raises Standard_ConstructionError for `t1 >= t2`, a negative pitch or
    /// radius, or `|taper| >= PI/2`; `helix.load(0.0, 6.28, 5.0, 2.0, 0.0, true)?`.
    ///
    /// Native quirks kept: the parameters are stored before the checks, so a failed
    /// call leaves them in the curve (with `c1` and `tan(taper)` of the last valid
    /// load); a taper within 1e-4 of zero keeps the previous `tan(taper)` instead of
    /// resetting it (docs/tkhelix-progress.md).
    pub fn load(
        &self,
        t1: f64,
        t2: f64,
        pitch: f64,
        r_start: f64,
        taper_angle: f64,
        is_cw: bool,
    ) -> Result<(), Failure> {
        let mut s = self.state.write();
        (s.first, s.last, s.pitch) = (t1, t2, pitch);
        (s.r_start, s.taper_angle, s.is_clock_wise) = (r_start, taper_angle, is_cw);
        check_load(&s)?;
        s.c1 = s.pitch / (2. * PI);
        if s.taper_angle.abs() > s.tol_angle {
            s.tg_beta = s.taper_angle.tan();
        }
        Ok(())
    }

    /// `new HelixGeom_HelixCurve(*this)` as the adaptor handle the approximation takes;
    /// `helix.to_adaptor()`.
    pub fn to_adaptor(&self) -> Handle<dyn Adaptor3dCurve> {
        Handle::new(self.clone()).cast()
    }

    // The point, first and second derivatives shared by the EvalD* members, in the
    // native operation order.
    fn point_at(s: &HelixCurveState, t: f64, cos_t: f64, sin_t: f64) -> Point3 {
        let a1: f64 = s.r_start + s.c1 * s.tg_beta * t;
        let x: f64 = a1 * cos_t;
        let y: f64 = oriented(s, a1 * sin_t);
        Point3::new(x, y, s.c1 * t)
    }

    // First derivative; returns it with (a1, a2) = (c1 tan(beta), R(t)).
    fn first_derivative(
        s: &HelixCurveState,
        t: f64,
        cos_t: f64,
        sin_t: f64,
    ) -> (Vector3, f64, f64) {
        let a1: f64 = s.c1 * s.tg_beta;
        let a2: f64 = s.r_start + a1 * t;
        let x: f64 = a1 * cos_t - a2 * sin_t;
        let y: f64 = oriented(s, a1 * sin_t + a2 * cos_t);
        (Vector3::new(x, y, s.c1), a1, a2)
    }
}

// The counter-clockwise helix mirrors Y.
fn oriented(s: &HelixCurveState, y: f64) -> f64 {
    if s.is_clock_wise { y } else { -y }
}

// The checks of Load, in the native order; `buf` is the native message.
fn check_load(s: &HelixCurveState) -> Result<(), Failure> {
    let half_pi: f64 = 0.5 * PI;
    let invalid: Option<String> = if s.first >= s.last {
        Some(format!("range [{}, {}]: expected T1 < T2", s.first, s.last))
    } else if s.pitch < 0. {
        Some(format!("pitch {}: expected >= 0", s.pitch))
    } else if s.r_start < 0. {
        Some(format!("start radius {}: expected >= 0", s.r_start))
    } else if s.taper_angle <= -half_pi || s.taper_angle >= half_pi {
        Some(format!(
            "taper angle {}: expected in (-PI/2, PI/2)",
            s.taper_angle
        ))
    } else {
        None
    };
    match invalid {
        Some(reason) => Err(construction_error(&format!(
            "HelixGeom_HelixCurve::Load: {reason}"
        ))),
        None => Ok(()),
    }
}

impl Adaptor3dCurve for HelixGeomHelixCurve {
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().last)
    }
    /// `GeomAbs_CN`: the helix is infinitely differentiable.
    fn continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    /// One interval whatever the continuity.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    /// `T(1) = First; T(2) = Last` (absolute indices: Standard_OutOfRange when the
    /// array does not hold them).
    fn intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let (first, last): (f64, f64) = {
            let s = self.state.read();
            (s.first, s.last)
        };
        *bounds
            .change_value(1)
            .map_err(array_failure("NCollection_Array1::operator()"))? = first;
        *bounds
            .change_value(2)
            .map_err(array_failure("NCollection_Array1::operator()"))? = last;
        Ok(())
    }
    fn resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented("HelixGeom_HelixCurve::Resolution"))
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented("HelixGeom_HelixCurve::IsClosed"))
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented("HelixGeom_HelixCurve::IsPeriodic"))
    }
    fn period(&self) -> Result<f64, Failure> {
        Err(domain_error("HelixGeom_HelixCurve::Period"))
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let s: HelixCurveState = *self.state.read();
        Ok(Self::point_at(&s, u, u.cos(), u.sin()))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let s: HelixCurveState = *self.state.read();
        let (cos_t, sin_t): (f64, f64) = (u.cos(), u.sin());
        let point: Point3 = Self::point_at(&s, u, cos_t, sin_t);
        let (d1, _, _) = Self::first_derivative(&s, u, cos_t, sin_t);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let s: HelixCurveState = *self.state.read();
        let (cos_t, sin_t): (f64, f64) = (u.cos(), u.sin());
        let point: Point3 = Self::point_at(&s, u, cos_t, sin_t);
        let (d1, a1, a2) = Self::first_derivative(&s, u, cos_t, sin_t);
        let a1: f64 = 2. * a1;
        let x: f64 = -a2 * cos_t - a1 * sin_t;
        let y: f64 = oriented(&s, -a2 * sin_t - a1 * cos_t);
        let d2 = Vector3::new(x, y, 0.);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    /// Orders 1 and 2 only; Standard_NotImplemented for any other order.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        match n {
            1 => Ok(self.eval_d1(u)?.d1),
            2 => Ok(self.eval_d2(u)?.d2),
            _ => Err(not_implemented(&format!(
                "HelixGeom_HelixCurve::EvalDN: order {n}; expected 1 or 2"
            ))),
        }
    }
}
