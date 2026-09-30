// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (the Perform overloads with points) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytic bisectors of a circle or a line and a point, and of two points.
use super::BisecAna;
use super::distance::{DistanceQuery, DistanceResult, distance};
use super::solutions::{
    Trimmed, degenerate, half_line_like, line_solution, line_to_center, trim_conic,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dParabola, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkgeomalgo::gcc_ana::{CircPnt2dBisec, LinPnt2dBisec, Pnt2dBisec};
use rustycad_tkgeomalgo::gcc_int::{Bisec, IType};
use rustycad_tkmath::gp::{Circle2, Line2, Point2, Vector2};

// The query of the point overloads: no reference vector.
fn point_query(apoint: Point2, v1: Vector2, v2: Vector2, adirection: f64) -> DistanceQuery {
    DistanceQuery {
        apoint,
        first: v1,
        second: v2,
        vec_ref: Vector2::new(0., 0.),
        adirection,
    }
}

impl BisecAna {
    /// `Perform(Cu, Pnt, P, V1, V2, Sense, Tolerance, oncurve)`: the bisector of a
    /// circle or a line (or a trimmed one) and a point, through `p`. Other curve
    /// types leave the object unchanged (natively after printing "Not yet
    /// implemented" on std::cout, which the port does not print);
    /// `bisector.perform_curve_point(&curve, &point, p, v1, v2, 1.0, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_curve_point(
        &self,
        cu: &Handle<dyn Geom2dCurve>,
        pnt: &Handle<dyn Geom2dPoint>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let curve: Handle<dyn Geom2dCurve> = match cu.cast::<Geom2dTrimmedCurve>().get() {
            Some(trimmed) => trimmed.basis_curve(),
            None => cu.clone(),
        };
        let point: Point2 = pnt.pnt2d();
        let query: DistanceQuery = point_query(p, v1, v2, sense);
        let result: Option<Trimmed> = if let Some(circle) = curve.cast::<Geom2dCircle>().get() {
            circle_point(circle.circ2d()?, point, query, tolerance)?
        } else if let Some(line) = curve.cast::<Geom2dLine>().get() {
            Some(line_point(line.lin2d(), point, query, tolerance, oncurve)?)
        } else {
            None
        };
        self.store(result);
        Ok(())
    }
    /// `Perform(Pnt, Cu, P, V1, V2, Sense, Tolerance, oncurve)`: the curve-point
    /// bisector with the vectors swapped and the sense reversed; the tolerance is
    /// ignored (0 is passed on, as natively);
    /// `bisector.perform_point_curve(&point, &curve, p, v1, v2, 1.0, 1e-7, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_point_curve(
        &self,
        pnt: &Handle<dyn Geom2dPoint>,
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        _tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        // skv, Feb 2005: the tolerance passed on is 0.
        self.perform_curve_point(cu, pnt, p, v2, v1, -sense, 0., oncurve)
    }
    /// `Perform(Pnt1, Pnt2, P, V1, V2, Sense, Tolerance, oncurve)`: the mediator of two
    /// points from `p`; left unchanged when its status is wrong on the curve.
    /// Standard_ConstructionError for equal points (no mediator);
    /// `bisector.perform_points(&p1, &p2, p, v1, v2, 1.0, 0.0, true)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_points(
        &self,
        pnt1: &Handle<dyn Geom2dPoint>,
        pnt2: &Handle<dyn Geom2dPoint>,
        p: Point2,
        v1: Vector2,
        v2: Vector2,
        sense: f64,
        _tolerance: f64,
        oncurve: bool,
    ) -> Result<(), Failure> {
        let line: Line2 = Pnt2dBisec::new(pnt1.pnt2d(), pnt2.pnt2d())?.this_solution()?;
        let result: DistanceResult = distance(
            point_query(p, v1, v2, sense),
            &*line_solution(line),
            false,
            false,
        )?;
        if !(result.ok || !oncurve) {
            return Ok(());
        }
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
        self.store(Some(half_line_like(curve, result.parameter, result.sense)?));
        Ok(())
    }
}

// Case 1 of the curve-point Perform: a circle and a point; the solution closest to
// the point of passage.
fn circle_point(
    circle: Circle2,
    point: Point2,
    query: DistanceQuery,
    tolerance: f64,
) -> Result<Option<Trimmed>, Failure> {
    let bisector = CircPnt2dBisec::with_tolerance(circle, point, tolerance);
    if !bisector.is_done() {
        return Ok(None);
    }
    let mut chosen: Option<(Handle<dyn Bisec>, DistanceResult)> = None;
    let mut distancemini: f64 = precision::infinite();
    for i in 1..=bisector.nb_solutions() {
        let solution = degenerate(bisector.this_solution(i)?, tolerance)?;
        let result: DistanceResult = distance(query, &*solution, false, false)?;
        if result.distance <= distancemini {
            distancemini = result.distance;
            chosen = Some((solution, result));
        }
    }
    let Some((solution, result)) = chosen else {
        return Ok(None);
    };
    if solution.arc_type() == IType::Lin {
        let line: Line2 = solution.line()?;
        return line_to_center(line, circle.location(), result.parameter, result.sense).map(Some);
    }
    trim_conic(&*solution, result.parameter, result.sense)
}

// Case 2 of the curve-point Perform: a line and a point, a parabola or a line.
fn line_point(
    line: Line2,
    point: Point2,
    query: DistanceQuery,
    tolerance: f64,
    oncurve: bool,
) -> Result<Trimmed, Failure> {
    let bisector = LinPnt2dBisec::new(line, point)?;
    let solution = degenerate(bisector.this_solution()?, tolerance)?;
    let curve: Handle<dyn Geom2dCurve> = match solution.arc_type() {
        IType::Lin => Handle::new(Geom2dLine::from_lin2d(solution.line()?)).cast(),
        IType::Par => Handle::new(Geom2dParabola::from_parab2d(solution.parabola()?)).cast(),
        _ => Handle::default(),
    };
    let result: DistanceResult = distance(query, &*solution, false, false)?;
    // The first parameter and the sense stay 0 and false when the status is wrong
    // on the curve.
    let (first, sense) = if result.ok || !oncurve {
        (result.parameter, result.sense)
    } else {
        (0., false)
    };
    half_line_like(curve, first, sense)
}
