// Rust translation of ModelingData/TKG2d/Adaptor2d/Adaptor2d_Line2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor2d_Line2d`: a bounded line seen as an adapted curve.
use super::curve2d::Adaptor2dCurve2d;
use crate::geom2d::errors::{array_bounds, no_such_object, out_of_range};
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Axis2d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};

#[derive(Clone, Copy, Default)]
struct Line2dState {
    u_first: f64,
    u_last: f64,
    ax2d: Axis2d,
}

/// A line on `[UFirst, ULast]` (`Adaptor2d_Line2d`), used by the topology tools to
/// trim surfaces; `Adaptor2dLine2d::new(Point2::new(0.0, 0.0), direction, 0.0, 10.0)`.
#[derive(Default)]
pub struct Adaptor2dLine2d {
    parent: StandardTransient,
    state: SharedState<Line2dState>,
}
impl_transient!(
    Adaptor2dLine2d,
    "Adaptor2d_Line2d",
    dyn Adaptor2dCurve2d,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

// Standard_NoSuchObject of the members that only other curve kinds answer.
fn not_a_line_member<T>(member: &str) -> Result<T, Failure> {
    Err(no_such_object(&format!(
        "Adaptor2d_Line2d::{member}: the adaptor is a line; expected the matching curve type"
    )))
}

impl Adaptor2dLine2d {
    /// Line through `point` along `direction` on `[u_first, u_last]`
    /// (`Adaptor2d_Line2d(P, D, UFirst, ULast)`);
    /// `Adaptor2dLine2d::new(p, d, 0.0, 10.0)`.
    pub fn new(point: Point2, direction: Direction2, u_first: f64, u_last: f64) -> Self {
        let state = Line2dState {
            u_first,
            u_last,
            ax2d: Axis2d::new(point, direction),
        };
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Load an unbounded line: the bounds become `-/+Precision::Infinite()`
    /// (`Load(gp_Lin2d)`); `adaptor.load(line)`.
    pub fn load(&self, line: Line2) {
        self.load_bounded(line, -precision::infinite(), precision::infinite());
    }
    /// Load a line on `[first, last]` (`Load(gp_Lin2d, UFirst, ULast)`);
    /// `adaptor.load_bounded(line, 2.0, 8.0)`.
    pub fn load_bounded(&self, line: Line2, first: f64, last: f64) {
        let mut state = self.state.write();
        state.ax2d = line.position();
        state.u_first = first;
        state.u_last = last;
    }
    fn axis(&self) -> Axis2d {
        self.state.read().ax2d
    }
}

impl Adaptor2dCurve2d for Adaptor2dLine2d {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let copy = Adaptor2dLine2d::default();
        *copy.state.write() = *self.state.read();
        Ok(Handle::new(copy).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().u_first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().u_last)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let state: Line2dState = *self.state.read();
        let lower: i32 = bounds.lower();
        bounds
            .set_value(lower, state.u_first)
            .map_err(array_bounds)?;
        bounds
            .set_value(lower + 1, state.u_last)
            .map_err(array_bounds)
    }
    fn trim(
        &self,
        first: f64,
        last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let trimmed = Adaptor2dLine2d::default();
        trimmed.load_bounded(Line2::from(self.axis()), first, last);
        Ok(Handle::new(trimmed).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn period(&self) -> Result<f64, Failure> {
        not_a_line_member("Period")
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        Ok(r3d)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(CurveType::Line)
    }
    fn line(&self) -> Result<Line2, Failure> {
        Ok(Line2::from(self.axis()))
    }
    fn circle(&self) -> Result<Circle2, Failure> {
        not_a_line_member("Circle")
    }
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        not_a_line_member("Ellipse")
    }
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        not_a_line_member("Hyperbola")
    }
    fn parabola(&self) -> Result<Parabola2, Failure> {
        not_a_line_member("Parabola")
    }
    fn degree(&self) -> Result<i32, Failure> {
        not_a_line_member("Degree")
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        not_a_line_member("IsRational")
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        not_a_line_member("NbPoles")
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        not_a_line_member("NbKnots")
    }
    fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        not_a_line_member("Bezier")
    }
    fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        not_a_line_member("BSpline")
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        Ok(elclib::line_value_2d(u, self.axis()))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.axis());
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.axis());
        let d2 = Vector2::new(0.0, 0.0);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1) = elclib::line_d1_2d(u, self.axis());
        let zero = Vector2::new(0.0, 0.0);
        Ok(Geom2dCurveD3 {
            point,
            d1,
            d2: zero,
            d3: zero,
        })
    }
    fn eval_dn(&self, _u: f64, n: i32) -> Result<Vector2, Failure> {
        if n <= 0 {
            return Err(out_of_range(&format!(
                "Adaptor2d_Line2d::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        if n == 1 {
            return Ok(Vector2::from(self.axis().direction()));
        }
        Ok(Vector2::new(0.0, 0.0))
    }
}
