// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_UniformDeflection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_UniformDeflection`: the points of a CPnts_UniformDeflection iteration on
//! each C2 part of the curve.
use super::deflection_common::{def_type, perform_circular, perform_composite, perform_linear};
use super::point_sequence::PointSequence;
use super::{DeflectionType, TCurveTypes};
use crate::cpnts::{self, PntsCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// `GCPnts_UniformDeflection`; `UniformDeflection::with_curve(curve, 0.01, true)?`.
///
/// `theWithControl` defaults to true natively; the port takes it explicitly.
#[derive(Default)]
pub struct UniformDeflection {
    my_done: bool,
    my_deflection: f64,
    my_points: PointSequence,
}

impl UniformDeflection {
    /// The empty algorithm, not done (`GCPnts_UniformDeflection()`);
    /// `UniformDeflection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The distribution on the whole curve
    /// (`GCPnts_UniformDeflection(C, Deflection, WithControl)`);
    /// `UniformDeflection::with_curve(curve, 0.01, true)?`.
    pub fn with_curve<C: TCurveTypes + ?Sized>(
        curve: &C,
        deflection: f64,
        with_control: bool,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.my_deflection = deflection;
        result.initialize(curve, deflection, with_control)?;
        Ok(result)
    }
    /// The distribution between `u1` and `u2`
    /// (`GCPnts_UniformDeflection(C, Deflection, U1, U2, WithControl)`);
    /// `UniformDeflection::with_range(curve, 0.01, 0.0, 1.0, true)?`.
    pub fn with_range<C: TCurveTypes + ?Sized>(
        curve: &C,
        deflection: f64,
        u1: f64,
        u2: f64,
        with_control: bool,
    ) -> Result<Self, Failure> {
        let mut result: Self = Self::new();
        result.my_deflection = deflection;
        result.initialize_range(curve, deflection, u1, u2, with_control)?;
        Ok(result)
    }
    /// Recompute on the whole curve (`Initialize(C, Deflection, WithControl)`);
    /// `algo.initialize(curve, 0.01, true)?`.
    pub fn initialize<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        deflection: f64,
        with_control: bool,
    ) -> Result<(), Failure> {
        let pnts = curve.pnts_curve();
        let (u1, u2) = (pnts.first_parameter()?, pnts.last_parameter()?);
        self.initialize_range(curve, deflection, u1, u2, with_control)
    }
    /// Recompute between `u1` and `u2`, swapped when `u1 > u2`
    /// (`Initialize(C, Deflection, U1, U2, WithControl)`): the ends of a linear curve,
    /// constant angular steps on a circle, CPnts_UniformDeflection on each C2 part
    /// otherwise, then the control of the last points;
    /// `algo.initialize_range(curve, 0.01, 0.0, 1.0, true)?`.
    pub fn initialize_range<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        deflection: f64,
        u1: f64,
        u2: f64,
        with_control: bool,
    ) -> Result<(), Failure> {
        let pnts: PntsCurve = curve.pnts_curve();
        let eps: f64 = pnts.resolution(precision::confusion())?;
        self.my_deflection = deflection;
        self.my_done = false;
        self.my_points.clear();
        let (a, b) = (std_min(u1, u2), std_max(u1, u2));
        let points: &mut PointSequence = &mut self.my_points;
        let mut perform = |sequence: &mut PointSequence, ua: f64, ub: f64| {
            perform_curve(sequence, pnts, deflection, (ua, ub), (eps, with_control))
        };
        self.my_done = match def_type(curve, Shape::C2)? {
            DeflectionType::Linear => perform_linear(pnts, points, a, b)?,
            DeflectionType::Circular => perform_circular(pnts, points, deflection, (a, b))?,
            DeflectionType::Curved => perform(points, a, b)?,
            DeflectionType::DefComposite => perform_composite(pnts, points, (a, b), &mut perform)?,
        };
        // control of the last points
        controle(pnts, &mut self.my_points, b)
    }
}

// `PerformCurve`: the points of a CPnts_UniformDeflection iteration on [U1, U2];
// returns its IsAllDone.
fn perform_curve(
    sequence: &mut PointSequence,
    curve: PntsCurve<'_>,
    deflection: f64,
    (u1, u2): (f64, f64),
    (eps, with_control): (f64, bool),
) -> Result<bool, Failure> {
    let mut iterator =
        cpnts::UniformDeflection::with_range(curve, deflection, u1, u2, eps, with_control)?;
    while iterator.more()? {
        sequence.append(iterator.value()?, iterator.point()?);
        iterator.next()?;
    }
    Ok(iterator.is_all_done())
}

// `Controle`: with more than two points, a last span shorter than a third of the two
// last spans moves the second-to-last point to the middle of the third-to-last and U2.
fn controle(curve: PntsCurve<'_>, sequence: &mut PointSequence, u2: f64) -> Result<(), Failure> {
    let nb_pnts: i32 = sequence.length();
    if nb_pnts <= 2 {
        return Ok(());
    }
    let ua: f64 = sequence.parameter(nb_pnts - 2)?;
    let ub: f64 = sequence.parameter(nb_pnts - 1)?;
    if u2 - ub < 0.33 * (u2 - ua) {
        let uc: f64 = (u2 + ua) * 0.5;
        sequence.set(nb_pnts - 1, uc, curve.lifted_d0(uc)?)?;
    }
    Ok(())
}

impl UniformDeflection {
    /// Whether the computation was successful (`IsDone`); `algo.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
    /// The number of points (`NbPoints`); StdFail_NotDone when not done;
    /// `algo.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("NbPoints")?;
        Ok(self.my_points.length())
    }
    /// The parameter of index `index`, from 1 (`Parameter`); StdFail_NotDone when not
    /// done, Standard_OutOfRange outside `[1, NbPoints]`; `algo.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.check_done("Parameter")?;
        self.my_points.parameter(index)
    }
    /// The point of index `index`, 2D points in the plane Z = 0 (`Value`);
    /// StdFail_NotDone (with the native message of GCPnts_UniformAbscissa) when not
    /// done; `algo.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Point3, Failure> {
        if !self.my_done {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                "GCPnts_UniformAbscissa::Parameter()",
                "",
            ));
        }
        self.my_points.point(index)
    }
    /// The deflection given to the algorithm (`Deflection`); StdFail_NotDone when not
    /// done; `algo.deflection()?`.
    pub fn deflection(&self) -> Result<f64, Failure> {
        self.check_done("Deflection")?;
        Ok(self.my_deflection)
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_done {
            return Ok(());
        }
        let message = format!("GCPnts_UniformDeflection::{member}() - not done");
        Err(Failure::with_type("StdFail_NotDone", &message, ""))
    }
}
