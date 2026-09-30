// Rust translation of Geom2dGcc_Circ2d2TanOnIter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d2TanOnIter`: a circle tangent to two arguments with its centre
//! on a line, a circle or a curve, found from start parameters.
mod on_circle;
mod on_curve;
mod on_line;

use super::circle_solutions::{circle_side, curve_side, line_side};
use super::{FunctionTanCuCuOnCu, QCurve};
use crate::gcc_ana::solutions::{circle_at, not_done};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle2, Point2, Vector2, Xy};
use rustycad_tkmath::math::{FunctionSet, FunctionSetRoot, Vector};

/// `Geom2dGcc_Circ2d2TanOnIter`: the circle centred on the locus and tangent to the
/// two arguments that math_FunctionSetRoot finds on Geom2dGcc_FunctionTanCuCuOnCu
/// from the start parameters (the start radius is the mean distance from the
/// start centre to the start points), kept when both distances agree within the
/// tolerance and the sides match the qualifiers.
///
/// The consultations answer 0 for every parameter, as natively. The point variants
/// search 4 unknowns with vectors of 3, so math_FunctionSetRoot raises
/// Standard_DimensionError on the start point, as natively.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d2TanOnIter, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let curve = |y: f64| -> Result<_, rustycad_tkernel::standard::Failure> {
///     let line: Handle<dyn Geom2dCurve> =
///         Handle::new(Geom2dLine::from_point(Point2::new(0.0, y), Direction2::new(1.0, 0.0)?)).cast();
///     unqualified(&Geom2dAdaptorCurve::with_range(&line, -5.0, 5.0)?)
/// };
/// let (q1, q2) = (curve(1.0)?, curve(-1.0)?);
/// let on = Line2::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0)?);
/// let circ = Circ2d2TanOnIter::curve_curve_on_line(&q1, &q2, on, [0.0, 0.0, 0.5], 1e-6)?;
/// assert!(circ.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanOnIter {
    well_done: bool,
    cirsol: Circle2,
    qualifier: [Position; 2],
    the_same: [bool; 2],
    pnttg: [Point2; 2],
    pntcen: Point2,
    parsol: [f64; 2],
    pararg: [f64; 2],
    parcen3: f64,
}

impl Default for Circ2d2TanOnIter {
    fn default() -> Self {
        Self {
            well_done: false,
            cirsol: Circle2::default(),
            qualifier: [Position::NoQualifier; 2],
            the_same: [false; 2],
            pnttg: [Point2::default(); 2],
            pntcen: Point2::default(),
            parsol: [0.0; 2],
            pararg: [0.0; 2],
            parcen3: 0.0,
        }
    }
}

// The bounds, start point and tolerances of one search (math_Vector(1, n) each).
struct Search {
    umin: Vec<f64>,
    umax: Vec<f64>,
    ufirst: Vec<f64>,
    tol: Vec<f64>,
}

fn vector_of(values: &[f64]) -> Result<Vector<'static>, Failure> {
    let mut v: Vector<'static> = Vector::with_bounds(1, values.len() as i32)?;
    for (i, value) in (1..).zip(values) {
        *v.change_value(i)? = *value;
    }
    Ok(v)
}

// math_FunctionSetRoot on `func` from `search`, then the native
// `Func.Value(Ufirst, Umin)` at the start point, then the root when done.
fn solve(func: &mut FunctionTanCuCuOnCu<'_>, search: &Search) -> Result<Option<Vec<f64>>, Failure> {
    let (umin, umax) = (vector_of(&search.umin)?, vector_of(&search.umax)?);
    let mut ufirst = vector_of(&search.ufirst)?;
    // The point variants hand 3 tolerances for 4 unknowns: math_FunctionSetRoot.cxx
    // (built with No_Standard_OutOfRange) reads the fourth past the end, and Perform
    // then raises Standard_DimensionError on the 3-vector start point before using it.
    let mut tol = search.tol.clone();
    tol.resize(tol.len().max(func.variables() as usize), 0.0);
    let mut root = FunctionSetRoot::new(&*func, &vector_of(&tol)?, 100)?;
    root.perform_bounded(func, &ufirst, &umin, &umax, false)?;
    func.raised()?;
    let mut values = vector_of(&search.umin)?;
    func.value(&ufirst, &mut values);
    func.raised()?;
    if !root.is_done() {
        return Ok(None);
    }
    root.copy_root(&mut ufirst)?;
    Ok(Some((1..=ufirst.length()).map(|i| ufirst[i]).collect()))
}

// `(point3.Distance(point2) + point3.Distance(point1)) / 2.`: the start radius.
fn start_radius(point1: Point2, point2: Point2, point3: Point2) -> f64 {
    (point3.distance(point2) + point3.distance(point1)) / 2.0
}

// One tangency argument of the solution as its side test sees it.
#[derive(Clone, Copy)]
enum Arg<'q> {
    Curve(&'q QCurve, Vector2),
    Circle(QualifiedCirc),
    Line(QualifiedLin),
    Point,
}

impl Arg<'_> {
    fn qualifier(&self) -> Position {
        match self {
            Arg::Curve(q, _) => q.qualifier(),
            Arg::Circle(q) => q.qualifier(),
            Arg::Line(q) => q.qualifier(),
            Arg::Point => Position::NoQualifier,
        }
    }

    // The side measure of the argument (the angle between the centre direction and
    // the tangent, 0 for short vectors), computed before any test as natively.
    fn measure(&self, point: Point2, centre: Point2) -> Result<f64, Failure> {
        let Arg::Curve(_, tan) = self else {
            return Ok(0.0);
        };
        let vec = Vector2::between(point, centre);
        if vec.magnitude() >= gp::resolution() && tan.magnitude() >= gp::resolution() {
            return vec.angle(*tan);
        }
        Ok(0.0)
    }

    fn side_ok(&self, angle: f64, cirsol: &Circle2, centre: Point2) -> bool {
        match self {
            Arg::Curve(q, _) => curve_side(q, angle),
            Arg::Circle(q) => {
                let c1: Circle2 = q.qualified();
                circle_side(
                    q.qualifier(),
                    cirsol.radius(),
                    c1.radius(),
                    c1.location().distance(centre),
                )
            }
            Arg::Line(q) => {
                let dir = q.qualified().direction();
                line_side(q.qualifier(), centre.xy().dot(Xy::new(-dir.y(), dir.x())))
            }
            Arg::Point => true,
        }
    }
}

// How the solution parameters are stored: on the solution circle, or (the
// curve-curve constructors on a circle or a curve) 0 and the chord length.
#[derive(Clone, Copy, PartialEq)]
enum ParSol {
    OnCircle,
    Chord,
}

// A candidate solution: the tangency points and arguments, the argument
// parameters, the centre and its parameter on the locus.
struct Candidate<'q> {
    points: [Point2; 2],
    args: [Arg<'q>; 2],
    pararg: [f64; 2],
    centre: Point2,
    parcen3: f64,
}

impl Circ2d2TanOnIter {
    // Accepts the candidate when its two radii agree within `tol` and both sides
    // match the qualifiers.
    fn finish(&mut self, cand: Candidate<'_>, tol: f64, mode: ParSol) -> Result<(), Failure> {
        let dist1: f64 = cand.centre.distance(cand.points[0]);
        let dist2: f64 = cand.centre.distance(cand.points[1]);
        let agree: bool = (dist1 - dist2).abs() / 2.0 <= tol;
        if !agree {
            return Ok(());
        }
        self.cirsol = circle_at(cand.centre, (dist1 + dist2) / 2.0)?;
        let angle1: f64 = cand.args[0].measure(cand.points[0], cand.centre)?;
        let angle2: f64 = cand.args[1].measure(cand.points[1], cand.centre)?;
        if !cand.args[0].side_ok(angle1, &self.cirsol, cand.centre)
            || !cand.args[1].side_ok(angle2, &self.cirsol, cand.centre)
        {
            return Ok(());
        }
        self.parsol = match mode {
            ParSol::OnCircle => [
                elclib::parameter(&self.cirsol, cand.points[0])?,
                elclib::parameter(&self.cirsol, cand.points[1])?,
            ],
            ParSol::Chord => [0.0, cand.points[1].distance(cand.points[0])],
        };
        self.qualifier = [cand.args[0].qualifier(), cand.args[1].qualifier()];
        (self.pnttg, self.pararg) = (cand.points, cand.pararg);
        (self.pntcen, self.parcen3) = (cand.centre, cand.parcen3);
        self.well_done = true;
        Ok(())
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `ThisSolution()`: the last circle of agreeing radii, even when rejected, or a
    /// default circle (no check, as natively); `circ.this_solution()`.
    pub fn this_solution(&self) -> Circle2 {
        self.cirsol
    }

    /// `WhichQualifier(Qualif1, Qualif2)`; `let (q1, q2) = circ.which_qualifier()?;`.
    pub fn which_qualifier(&self) -> Result<(Position, Position), Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::WhichQualifier")?;
        Ok((self.qualifier[0], self.qualifier[1]))
    }

    /// `Tangency1(ParSol, ParArg, PntSol)`: natively both parameters are 0;
    /// `let (par_sol, par_arg, point) = circ.tangency1()?;`.
    pub fn tangency1(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::Tangency1")?;
        if self.the_same[0] {
            return Err(not_done("Geom2dGcc_Circ2d2TanOnIter::Tangency1"));
        }
        Ok((0.0, 0.0, self.pnttg[0]))
    }

    /// `Tangency2(ParSol, ParArg, PntSol)`: natively both parameters are 0;
    /// `let (par_sol, par_arg, point) = circ.tangency2()?;`.
    pub fn tangency2(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::Tangency2")?;
        Ok((0.0, 0.0, self.pnttg[1]))
    }

    /// `CenterOn3(ParArg, PntSol)`: natively the parameter is 0;
    /// `let (par_arg, point) = circ.center_on3()?;`.
    pub fn center_on3(&self) -> Result<(f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::CenterOn3")?;
        Ok((0.0, self.pntcen))
    }

    /// `IsTheSame1()`: false; `circ.is_the_same1()?`.
    pub fn is_the_same1(&self) -> Result<bool, Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::IsTheSame1")?;
        Ok(self.the_same[0])
    }

    /// `IsTheSame2()`: false; `circ.is_the_same2()?`.
    pub fn is_the_same2(&self) -> Result<bool, Failure> {
        self.check_done("Geom2dGcc_Circ2d2TanOnIter::IsTheSame2")?;
        Ok(false)
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.well_done {
            return Err(not_done(member));
        }
        Ok(())
    }
}
