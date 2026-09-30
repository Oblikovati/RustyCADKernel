// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_BisecAna`: the analytic bisector of two circles, lines or points, a
//! trimmed conic.
mod curve;
mod distance;
mod perform_circles;
mod perform_curves;
mod perform_lines;
mod perform_points;
mod solutions;

use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dGeometry, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola, errors,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point2;
use solutions::Trimmed;

use super::BisectorCurve;

/// `Bisector_BisecAna`: a `Geom2d_TrimmedCurve` on a line, circle, ellipse,
/// hyperbola or parabola that bisects two circles, lines or points
/// (docs/topalgo-design.md section 3). The evaluators evaluate the basis conic; the
/// parameter range is the one of the trimmed curve. A default object has no curve:
/// a member that dereferences it panics, as a null handle dereference crashes
/// natively.
/// ```
/// use rustycad_tktopalgo::bisector::BisecAna;
/// use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dPoint};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Point2, Vector2};
/// let p1: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0)).cast();
/// let p2: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(2.0, 0.0)).cast();
/// let bisector = BisecAna::new();
/// bisector.perform_points(&p1, &p2, Point2::new(1.0, 0.0), Vector2::new(0.0, 1.0),
///     Vector2::new(0.0, -1.0), 1.0, 0.0, true)?;
/// assert_eq!(bisector.value(bisector.first_parameter())?, Point2::new(1.0, 0.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BisecAna {
    parent: StandardTransient,
    thebisector: SharedState<Trimmed>,
}
impl_transient!(
    BisecAna,
    "Bisector_BisecAna",
    dyn BisectorCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn BisectorCurve]
);

impl Default for BisecAna {
    fn default() -> Self {
        Self::new()
    }
}

impl BisecAna {
    /// `Bisector_BisecAna()`: no curve yet; `BisecAna::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            thebisector: SharedState::new(Handle::default()),
        }
    }
    /// `Init(bisector)`: the trimmed conic becomes the bisector (shared, not copied);
    /// `bisector.init(&trimmed)`.
    pub fn init(&self, bisector: &Trimmed) {
        *self.thebisector.write() = bisector.clone();
    }
    // The trimmed curve, read without holding the lock during its evaluation.
    fn trimmed(&self) -> Trimmed {
        self.thebisector.read().clone()
    }
    // Stores the result of a Perform, which the native code leaves unchanged when
    // it builds no curve.
    fn store(&self, result: Option<Trimmed>) {
        if let Some(bisector) = result {
            self.init(&bisector);
        }
    }
    /// `SetTrim(Cu)`: the native body is commented out, so it does nothing;
    /// `bisector.set_trim_curve(&curve)`.
    pub fn set_trim_curve(&self, _cu: &Handle<dyn Geom2dCurve>) {}
    /// `SetTrim(uf, ul)`: trims the conic to `[uf, ul]`; `bisector.set_trim(0.0, 1.0)?`.
    pub fn set_trim(&self, uf: f64, ul: f64) -> Result<(), Failure> {
        self.trimmed().set_trim(uf, ul, true, true)
    }
    /// `Geom2dCurve()`: the basis conic; `bisector.geom2d_curve()`.
    pub fn geom2d_curve(&self) -> Handle<dyn Geom2dCurve> {
        self.trimmed().basis_curve()
    }
    /// `ParameterOfStartPoint()`: the first trim parameter;
    /// `bisector.parameter_of_start_point()`.
    pub fn parameter_of_start_point(&self) -> f64 {
        self.trimmed().first_parameter()
    }
    /// `ParameterOfEndPoint()`: the last trim parameter;
    /// `bisector.parameter_of_end_point()`.
    pub fn parameter_of_end_point(&self) -> f64 {
        self.trimmed().last_parameter()
    }
    /// `Dump(Deep, Offset)`: the class name after `Offset` spaces, then `Offset`
    /// spaces again, as `std::cout` gets them; `bisector.dump(0, 2, &mut text)`.
    pub fn dump(&self, _deep: i32, offset: i32, out: &mut String) {
        let indent: String = " ".repeat(usize::try_from(offset).unwrap_or(0));
        out.push_str(&format!("{indent}Bisector_BisecAna\n{indent}"));
    }
}

impl BisectorCurve for BisecAna {
    fn parameter(&self, p: Point2) -> Result<f64, Failure> {
        let basis: Handle<dyn Geom2dCurve> = self.geom2d_curve();
        if let Some(line) = basis.cast::<Geom2dLine>().get() {
            return elclib::parameter(&line.lin2d(), p);
        }
        if let Some(circle) = basis.cast::<Geom2dCircle>().get() {
            return elclib::parameter(&circle.circ2d()?, p);
        }
        if let Some(hyperbola) = basis.cast::<Geom2dHyperbola>().get() {
            return elclib::parameter(&hyperbola.hypr2d()?, p);
        }
        if let Some(parabola) = basis.cast::<Geom2dParabola>().get() {
            return elclib::parameter(&parabola.parab2d()?, p);
        }
        if let Some(ellipse) = basis.cast::<Geom2dEllipse>().get() {
            return elclib::parameter(&ellipse.elips2d()?, p);
        }
        Ok(0.)
    }
    fn is_extend_at_start(&self) -> bool {
        false
    }
    fn is_extend_at_end(&self) -> bool {
        false
    }
    fn nb_intervals(&self) -> i32 {
        1
    }
    fn interval_first(&self, index: i32) -> Result<f64, Failure> {
        check_interval(index)?;
        Ok(self.first_parameter())
    }
    fn interval_last(&self, index: i32) -> Result<f64, Failure> {
        check_interval(index)?;
        Ok(self.last_parameter())
    }
}

// `IntervalFirst`/`IntervalLast`: the only interval is 1.
fn check_interval(index: i32) -> Result<(), Failure> {
    if index != 1 {
        return Err(errors::out_of_range(&format!(
            "Bisector_BisecAna: interval {index}; expected 1"
        )));
    }
    Ok(())
}
