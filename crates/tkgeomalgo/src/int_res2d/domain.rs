// Rust translation of IntRes2d_Domain.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntRes2d_Domain`: the parameter domain of a 2D curve.
use super::failures::domain_error;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

// The status bits of the native domain: has a first point, has a last point, closed.
const HAS_FIRST: i32 = 1;
const HAS_LAST: i32 = 2;
const CLOSED: i32 = 4;

/// `IntRes2d_Domain`: 0, 1 or 2 extremities (a point, a parameter and a
/// tolerance each), and for a closed curve the two equivalent parameters of its
/// period. Infinite values are clamped to `Precision::Infinite()` (occ 438) so that
/// arithmetic on the bounds stays finite.
///
/// ```
/// use rustycad_tkgeomalgo::int_res2d::Domain;
/// use rustycad_tkmath::gp::Point2;
/// let domain = Domain::bounded(Point2::new(0.0, 0.0), 0.0, 1e-7, Point2::new(1.0, 0.0), 1.0, 1e-7);
/// assert_eq!(domain.last_parameter().unwrap(), 1.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Domain {
    status: i32,
    first_param: f64,
    last_param: f64,
    first_tol: f64,
    last_tol: f64,
    first_point: Point2,
    last_point: Point2,
    periodfirst: f64,
    periodlast: f64,
}

impl Default for Domain {
    /// `IntRes2d_Domain()`: the infinite domain. `Domain::default().has_first_point() == false`.
    fn default() -> Self {
        Self {
            status: 0,
            first_param: 0.0,
            last_param: 0.0,
            first_tol: 0.0,
            last_tol: 0.0,
            first_point: Point2::new(0.0, 0.0),
            last_point: Point2::new(0.0, 0.0),
            periodfirst: 0.0,
            periodlast: 0.0,
        }
    }
}

// `LimitInfinite`: clamps a value to +/- Precision::Infinite().
fn limit_infinite(value: f64) -> f64 {
    let inf_val: f64 = precision::infinite();
    if value.abs() > inf_val {
        return if value > 0.0 { inf_val } else { -inf_val };
    }
    value
}

fn limited_point(point: Point2) -> Point2 {
    Point2::new(limit_infinite(point.x()), limit_infinite(point.y()))
}

impl Domain {
    /// `IntRes2d_Domain()`: an infinite domain. `Domain::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IntRes2d_Domain(Pnt1, Par1, Tol1, Pnt2, Par2, Tol2)`: a bounded domain;
    /// `Domain::bounded(p1, 0.0, 1e-7, p2, 1.0, 1e-7)`.
    pub fn bounded(pnt1: Point2, par1: f64, tol1: f64, pnt2: Point2, par2: f64, tol2: f64) -> Self {
        let mut domain: Self = Self::default();
        domain.set_values(pnt1, par1, tol1, pnt2, par2, tol2);
        domain
    }
    /// `IntRes2d_Domain(Pnt, Par, Tol, First)`: a semi-infinite domain whose point is
    /// the first one when `first`, the last one otherwise;
    /// `Domain::semi_infinite(p, 0.0, 1e-7, true)`.
    pub fn semi_infinite(pnt: Point2, par: f64, tol: f64, first: bool) -> Self {
        let mut domain: Self = Self::default();
        domain.set_semi_infinite_values(pnt, par, tol, first);
        domain
    }
    /// `SetValues(Pnt1, Par1, Tol1, Pnt2, Par2, Tol2)`: the bounded domain;
    /// `domain.set_values(p1, 0.0, 1e-7, p2, 1.0, 1e-7)`.
    pub fn set_values(
        &mut self,
        pnt1: Point2,
        par1: f64,
        tol1: f64,
        pnt2: Point2,
        par2: f64,
        tol2: f64,
    ) {
        self.status = HAS_FIRST | HAS_LAST;
        self.periodfirst = 0.0;
        self.periodlast = 0.0;
        self.first_param = limit_infinite(par1);
        self.first_point = limited_point(pnt1);
        self.first_tol = tol1;
        self.last_param = limit_infinite(par2);
        self.last_point = limited_point(pnt2);
        self.last_tol = tol2;
    }
    /// `SetValues()`: the infinite domain (the bounds keep their old values);
    /// `domain.set_infinite_values()`.
    pub fn set_infinite_values(&mut self) {
        self.status = 0;
        self.periodfirst = 0.0;
        self.periodlast = 0.0;
    }
    /// `SetValues(Pnt, Par, Tol, First)`: the semi-infinite domain;
    /// `domain.set_semi_infinite_values(p, 0.0, 1e-7, false)`.
    pub fn set_semi_infinite_values(&mut self, pnt: Point2, par: f64, tol: f64, first: bool) {
        self.periodfirst = 0.0;
        self.periodlast = 0.0;
        if first {
            self.status = HAS_FIRST;
            self.first_param = limit_infinite(par);
            self.first_point = limited_point(pnt);
            self.first_tol = tol;
        } else {
            self.status = HAS_LAST;
            self.last_param = limit_infinite(par);
            self.last_point = limited_point(pnt);
            self.last_tol = tol;
        }
    }
    /// `SetEquivalentParameters(zero, period)`: closes a bounded domain;
    /// Standard_DomainError when a bound is missing;
    /// `domain.set_equivalent_parameters(0.0, 2.0 * PI)?`.
    pub fn set_equivalent_parameters(&mut self, p_first: f64, p_last: f64) -> Result<(), Failure> {
        if (self.status & 3) != 3 {
            return Err(domain_error(
                "IntRes2d_Domain::SetEquivalentParameters",
                &format!(
                    "status {} is not bounded; expected both bounds",
                    self.status
                ),
            ));
        }
        self.status |= CLOSED;
        self.periodfirst = p_first;
        self.periodlast = p_last;
        Ok(())
    }
    /// `HasFirstPoint()`: the domain has a lowest parameter. `domain.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        (self.status & HAS_FIRST) != 0
    }
    /// `FirstParameter()`; Standard_DomainError without a first point.
    /// `domain.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        self.check_first("IntRes2d_Domain::FirstParameter")?;
        Ok(self.first_param)
    }
    /// `FirstPoint()`; Standard_DomainError without a first point. `domain.first_point()?`.
    pub fn first_point(&self) -> Result<Point2, Failure> {
        self.check_first("IntRes2d_Domain::FirstPoint")?;
        Ok(self.first_point)
    }
    /// `FirstTolerance()`; Standard_DomainError without a first point.
    /// `domain.first_tolerance()?`.
    pub fn first_tolerance(&self) -> Result<f64, Failure> {
        self.check_first("IntRes2d_Domain::FirstTolerance")?;
        Ok(self.first_tol)
    }
    /// `HasLastPoint()`: the domain has a highest parameter. `domain.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        (self.status & HAS_LAST) != 0
    }
    /// `LastParameter()`; Standard_DomainError without a last point.
    /// `domain.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        self.check_last("IntRes2d_Domain::LastParameter")?;
        Ok(self.last_param)
    }
    /// `LastPoint()`; Standard_DomainError without a last point. `domain.last_point()?`.
    pub fn last_point(&self) -> Result<Point2, Failure> {
        self.check_last("IntRes2d_Domain::LastPoint")?;
        Ok(self.last_point)
    }
    /// `LastTolerance()`; Standard_DomainError without a last point.
    /// `domain.last_tolerance()?`.
    pub fn last_tolerance(&self) -> Result<f64, Failure> {
        self.check_last("IntRes2d_Domain::LastTolerance")?;
        Ok(self.last_tol)
    }
    /// `IsClosed()`: the domain has equivalent parameters. `domain.is_closed()`.
    pub fn is_closed(&self) -> bool {
        (self.status & CLOSED) != 0
    }
    /// `EquivalentParameters(zero, zeroplusperiod)`: the two parameters of the
    /// period. The native member does not raise on an open domain (despite its
    /// documentation): it returns the stored values, 0 and 0 unless closed.
    /// `let (zero, end) = domain.equivalent_parameters();`
    pub fn equivalent_parameters(&self) -> (f64, f64) {
        (self.periodfirst, self.periodlast)
    }

    fn check_first(&self, member: &str) -> Result<(), Failure> {
        if !self.has_first_point() {
            return Err(domain_error(
                member,
                &format!(
                    "status {} has no first point; expected HasFirstPoint()",
                    self.status
                ),
            ));
        }
        Ok(())
    }
    fn check_last(&self, member: &str) -> Result<(), Failure> {
        if !self.has_last_point() {
            return Err(domain_error(
                member,
                &format!(
                    "status {} has no last point; expected HasLastPoint()",
                    self.status
                ),
            ));
        }
        Ok(())
    }
}
