// Rust translation of Geom2dGcc_Lin2d2TanIter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Lin2d2TanIter`: a line tangent to two arguments (curve, circle or
//! point), found from start parameters.
use super::{CurveTool, FunctionTanCirCu, FunctionTanCuCu, FunctionTanCuPnt, QCurve};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::not_done;
use crate::gcc_ent::{Position, QualifiedCirc, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2, Xy};
use rustycad_tkmath::math::{Function, FunctionRoot, FunctionSet, FunctionSetRoot, Vector};

/// `Geom2dGcc_Lin2d2TanIter`: the line tangent to a qualified curve and passing
/// through a point, tangent to a qualified circle and a curve, or tangent to two
/// curves, found by a tkmath root search from the given parameters and accepted
/// when its residuals are below `Tolang` and its sides match the qualifiers.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Lin2d2TanIter, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&circle)?)?;
/// let lin = Lin2d2TanIter::through_point(&q, Point2::new(3.0, 0.0), 1.0, 1e-9)?;
/// assert!(lin.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2d2TanIter {
    well_done: bool,
    linsol: Line2,
    qualifier1: Position,
    qualifier2: Position,
    pnttg1sol: Point2,
    pnttg2sol: Point2,
    par1sol: f64,
    par2sol: f64,
    pararg1: f64,
    pararg2: f64,
}

impl Default for Lin2d2TanIter {
    fn default() -> Self {
        Self {
            well_done: false,
            linsol: Line2::default(),
            qualifier1: Position::NoQualifier,
            qualifier2: Position::NoQualifier,
            pnttg1sol: Point2::default(),
            pnttg2sol: Point2::default(),
            par1sol: 0.0,
            par2sol: 0.0,
            pararg1: 0.0,
            pararg2: 0.0,
        }
    }
}

// The qualifier test of a tangency: `sign1` is the dot product of the tangent and
// the direction to the other element, `sign2` the cross product of the second
// derivative and that direction.
fn curve_side_ok(q: &QCurve, sign1: f64, sign2: f64) -> bool {
    q.is_unqualified()
        || (q.is_enclosing() && sign2 <= 0.0)
        || (q.is_outside() && sign1 <= 0.0 && sign2 >= 0.0)
        || (q.is_enclosed() && sign1 >= 0.0 && sign2 >= 0.0)
}

// The qualifier test of the curve-curve search: `angle` between the chord and the
// second derivative, `sign1` the dot product of the tangent and the chord.
fn chord_side_ok(q: &QCurve, angle: f64, sign1: f64) -> bool {
    q.is_unqualified()
        || (q.is_enclosing() && angle >= 0.0)
        || (q.is_outside() && angle <= 0.0 && sign1 <= 0.0)
        || (q.is_enclosed() && angle <= 0.0 && sign1 >= 0.0)
}

impl Lin2d2TanIter {
    /// `Geom2dGcc_Lin2d2TanIter(Qualified1, ThePoint, Param1, Tolang)`: the line
    /// through `the_point` tangent to the curve, from `param1`; GccEnt_BadQualifier
    /// for GccEnt_noqualifier. `Lin2d2TanIter::through_point(&q, point, 0.0, 1e-9)?`.
    pub fn through_point(
        qualified1: &QCurve,
        the_point: Point2,
        param1: f64,
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut iter = Self::default();
        check_qualifier(qualified1.qualifier(), "Geom2dGcc_Lin2d2TanIter")?;
        let cu1 = qualified1.curve();
        let u1: f64 = CurveTool::first_parameter(cu1)?;
        let u2: f64 = CurveTool::last_parameter(cu1)?;
        let mut func = FunctionTanCuPnt::new(cu1, the_point);
        let eps: f64 = CurveTool::eps_x(cu1, tolang.abs())?;
        let sol = FunctionRoot::bounded(&mut func, param1, eps, u1, u2, 100)?;
        func.raised()?;
        if !sol.is_done() {
            return Ok(iter);
        }
        let usol: f64 = sol.root()?;
        let mut norm: f64 = 0.0;
        func.value(usol, &mut norm);
        func.raised()?;
        if norm.abs() >= tolang {
            return Ok(iter);
        }
        let (origine, vect1, vect2) = CurveTool::d2(cu1, usol)?;
        let vdir = Vector2::from(the_point.xy() - origine.xy());
        let sign1: f64 = vect1.dot(vdir);
        let sign2: f64 = vect2.crossed(vdir);
        if !curve_side_ok(qualified1, sign1, sign2) {
            return Ok(iter);
        }
        iter.well_done = true;
        iter.linsol = Line2::new(origine, Direction2::from_vector(vdir)?);
        iter.qualifier1 = qualified1.qualifier();
        iter.qualifier2 = Position::NoQualifier;
        iter.pnttg1sol = origine;
        iter.pnttg2sol = the_point;
        iter.pararg1 = usol;
        iter.par1sol = 0.0;
        iter.pararg2 = the_point.distance(origine);
        iter.par2sol = 0.0;
        Ok(iter)
    }

    /// `Geom2dGcc_Lin2d2TanIter(GccEnt_QualifiedCirc Qualified1, Qualified2, Param2,
    /// Tolang)`: the line tangent to the circle and the curve, from `param2` on the
    /// curve. Natively only an enclosed circle raises GccEnt_BadQualifier.
    /// `Lin2d2TanIter::circle_curve(qc, &q, 0.0, 1e-9)?`.
    pub fn circle_curve(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        param2: f64,
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut iter = Self::default();
        if qualified1.is_enclosed() {
            return Err(bad_qualifier(
                "Geom2dGcc_Lin2d2TanIter: circle qualifier GccEnt_enclosed; expected enclosing, outside or unqualified",
            ));
        }
        let cu2 = qualified2.curve();
        let u1: f64 = CurveTool::first_parameter(cu2)?;
        let u2: f64 = CurveTool::last_parameter(cu2)?;
        let mut func = FunctionTanCirCu::new(qualified1.qualified(), cu2)?;
        let eps: f64 = CurveTool::eps_x(cu2, tolang.abs())?;
        let sol = FunctionRoot::bounded(&mut func, param2, eps, u1, u2, 100)?;
        func.raised()?;
        if !sol.is_done() {
            return Ok(iter);
        }
        let usol: f64 = sol.root()?;
        let mut norm: f64 = 0.0;
        func.value(usol, &mut norm);
        func.raised()?;
        if norm.abs() < tolang {
            iter.circle_tangency(qualified1, qualified2, usol)?;
        }
        Ok(iter)
    }

    // The solution of the circle-curve search at the curve parameter `usol`.
    fn circle_tangency(
        &mut self,
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        usol: f64,
    ) -> Result<(), Failure> {
        let c1 = qualified1.qualified();
        let (origine, mut vect1, vect2) = CurveTool::d2(qualified2.curve(), usol)?;
        let vdir = Vector2::from(c1.location().xy() - origine.xy());
        let sign1: f64 = vect1.dot(vdir);
        if sign1 <= 0.0 {
            vect1.reverse();
        }
        let sign2: f64 = vect2.crossed(vect1);
        if !curve_side_ok(qualified2, sign1, sign2) {
            return Ok(());
        }
        let circle_ok: bool = qualified1.is_unqualified()
            || (qualified1.is_outside() && vect1.angle(vdir)? <= 0.0)
            || (qualified1.is_enclosing() && vect1.angle(vdir)? >= 0.0);
        if !circle_ok {
            return Ok(());
        }
        let direc = Direction2::from_vector(vect1)?;
        let r1: f64 = c1.radius();
        let normal = Xy::new(-r1 * direc.y(), r1 * direc.x());
        let side: f64 = vect1.crossed(vdir);
        let inner: bool = qualified1.is_enclosing() || (!qualified1.is_outside() && side >= 0.0);
        self.pnttg1sol = if inner {
            Point2::from(c1.location().xy() - normal)
        } else {
            Point2::from(c1.location().xy() + normal)
        };
        self.pnttg2sol = origine;
        self.linsol = Line2::new(self.pnttg1sol, direc);
        self.well_done = true;
        self.qualifier1 = qualified1.qualifier();
        self.qualifier2 = qualified2.qualifier();
        self.pararg2 = usol;
        self.par1sol = 0.0;
        self.par2sol = self.pnttg2sol.distance(self.pnttg1sol);
        self.pararg1 = 0.0;
        Ok(())
    }

    /// `Geom2dGcc_Lin2d2TanIter(Qualified1, Qualified2, Param1, Param2, Tolang)`: the
    /// line tangent to both curves, found by math_FunctionSetRoot from
    /// (`param1`, `param2`). Natively the side tests take the angle between the
    /// chord and each second derivative, so a line argument (null second derivative)
    /// raises gp_VectorWithNullMagnitude once a root is found.
    /// `Lin2d2TanIter::new(&q1, &q2, 0.0, 0.0, 1e-9)?`.
    pub fn new(
        qualified1: &QCurve,
        qualified2: &QCurve,
        param1: f64,
        param2: f64,
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut iter = Self::default();
        check_qualifier(qualified1.qualifier(), "Geom2dGcc_Lin2d2TanIter")?;
        check_qualifier(qualified2.qualifier(), "Geom2dGcc_Lin2d2TanIter")?;
        let (cu1, cu2) = (qualified1.curve(), qualified2.curve());
        let mut func = FunctionTanCuCu::new(cu1, cu2);
        let umin = pair(
            CurveTool::first_parameter(cu1)?,
            CurveTool::first_parameter(cu2)?,
        )?;
        let umax = pair(
            CurveTool::last_parameter(cu1)?,
            CurveTool::last_parameter(cu2)?,
        )?;
        let mut ufirst = pair(param1, param2)?;
        let tol = pair(
            CurveTool::eps_x(cu1, tolang.abs())?,
            CurveTool::eps_x(cu2, tolang.abs())?,
        )?;
        let mut root = FunctionSetRoot::new(&func, &tol, 100)?;
        root.perform_bounded(&mut func, &ufirst, &umin, &umax, false)?;
        func.raised()?;
        if !root.is_done() {
            return Ok(iter);
        }
        root.copy_root(&mut ufirst)?;
        let mut norm: Vector<'static> = Vector::with_bounds(1, 2)?;
        func.value(&ufirst, &mut norm);
        func.raised()?;
        if norm[1].abs() < tolang && norm[2].abs() < tolang {
            iter.curves_tangency(qualified1, qualified2, *ufirst.value(1)?, *ufirst.value(2)?)?;
        }
        Ok(iter)
    }

    // The solution of the curve-curve search at the parameters (`u1`, `u2`).
    fn curves_tangency(
        &mut self,
        qualified1: &QCurve,
        qualified2: &QCurve,
        u1: f64,
        u2: f64,
    ) -> Result<(), Failure> {
        let (point1, vect11, vect12) = CurveTool::d2(qualified1.curve(), u1)?;
        let (point2, vect21, vect22) = CurveTool::d2(qualified2.curve(), u2)?;
        let vec = Vector2::from(point2.xy() - point1.xy());
        let angle1: f64 = vec.angle(vect12)?;
        if !chord_side_ok(qualified1, angle1, vect11.dot(vec)) {
            return Ok(());
        }
        let angle2: f64 = vec.angle(vect22)?;
        if !chord_side_ok(qualified2, angle2, vect21.dot(vec)) {
            return Ok(());
        }
        self.qualifier1 = qualified1.qualifier();
        self.qualifier2 = qualified2.qualifier();
        self.pararg1 = u1;
        self.par1sol = 0.0;
        self.pnttg1sol = point1;
        self.pararg2 = u2;
        self.pnttg2sol = point2;
        self.par2sol = self.pnttg2sol.distance(self.pnttg1sol);
        let dir = Direction2::new(
            self.pnttg2sol.x() - self.pnttg1sol.x(),
            self.pnttg2sol.y() - self.pnttg1sol.y(),
        )?;
        self.linsol = Line2::new(self.pnttg1sol, dir);
        self.well_done = true;
        Ok(())
    }

    /// `IsDone()`; `lin.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `ThisSolution()`: StdFail_NotDone when the search failed; `lin.this_solution()?`.
    pub fn this_solution(&self) -> Result<Line2, Failure> {
        self.check_done("Geom2dGcc_Lin2d2TanIter::ThisSolution")?;
        Ok(self.linsol)
    }

    /// `WhichQualifier(Qualif1, Qualif2)`; `let (q1, q2) = lin.which_qualifier()?;`.
    pub fn which_qualifier(&self) -> Result<(Position, Position), Failure> {
        self.check_done("Geom2dGcc_Lin2d2TanIter::WhichQualifier")?;
        Ok((self.qualifier1, self.qualifier2))
    }

    /// `Tangency1(ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = lin.tangency1()?;`.
    pub fn tangency1(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Lin2d2TanIter::Tangency1")?;
        Ok((self.par1sol, self.pararg1, self.pnttg1sol))
    }

    /// `Tangency2(ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = lin.tangency2()?;`.
    pub fn tangency2(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Lin2d2TanIter::Tangency2")?;
        Ok((self.par2sol, self.pararg2, self.pnttg2sol))
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.well_done {
            return Err(not_done(member));
        }
        Ok(())
    }
}

// `math_Vector v(1, 2)` holding (a, b).
fn pair(a: f64, b: f64) -> Result<Vector<'static>, Failure> {
    let mut v: Vector<'static> = Vector::with_bounds(1, 2)?;
    (v[1], v[2]) = (a, b);
    Ok(v)
}
