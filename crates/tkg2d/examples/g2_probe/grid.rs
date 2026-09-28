//! The Geom2dGridEval grid evaluators; the counterpart of tools/oracle/tkg2d_probe_grid.cxx
//! (g2_grid, g2_grid_adaptor, g2_grid_evalrep, g2_grid_null).
use super::adaptor::built as built_adaptor;
use super::adaptor_cases::set_rep;
use super::bspl::desc;
use super::common::{Reader, attempt, built, curve, end, flag, int, pnt, vec};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalBSplineCurve, Geom2dGridEvalBezierCurve, Geom2dGridEvalCircle,
    Geom2dGridEvalCurve, Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3,
    Geom2dGridEvalEllipse, Geom2dGridEvalHyperbola, Geom2dGridEvalLine, Geom2dGridEvalOffsetCurve,
    Geom2dGridEvalOtherCurve, Geom2dGridEvalParabola,
};
use rustycad_tkmath::gp::{Point2, Vector2};

/// `lower n u*n`, as g2_grid_params reads it.
fn params(r: &mut Reader) -> Array1<'static, f64> {
    let lower = r.integer();
    let u = r.reals();
    if u.is_empty() {
        return Array1::default();
    }
    let mut p: Array1<'static, f64> =
        Array1::with_bounds(lower, lower + u.len() as i32 - 1).unwrap();
    p.as_mut_slice().copy_from_slice(&u);
    p
}

/// One grid element, as the g2_grid_value overloads print it.
trait GridValue {
    fn print(&self);
}
impl GridValue for Point2 {
    fn print(&self) {
        pnt(*self);
    }
}
impl GridValue for Vector2 {
    fn print(&self) {
        vec(*self);
    }
}
impl GridValue for Geom2dGridEvalCurveD1 {
    fn print(&self) {
        pnt(self.point);
        vec(self.d1);
    }
}
impl GridValue for Geom2dGridEvalCurveD2 {
    fn print(&self) {
        pnt(self.point);
        [self.d1, self.d2].into_iter().for_each(vec);
    }
}
impl GridValue for Geom2dGridEvalCurveD3 {
    fn print(&self) {
        pnt(self.point);
        [self.d1, self.d2, self.d3].into_iter().for_each(vec);
    }
}

fn array<T: GridValue>(grid: Result<Array1<'static, T>, Failure>) {
    attempt(|| {
        let grid = grid?;
        int(grid.lower());
        int(grid.len() as i32);
        grid.iter().for_each(GridValue::print);
        Ok(())
    });
}

/// The five members of every evaluator, with `Result` for the infallible line ones.
trait GridEvaluator {
    fn grid(&self, p: &Array1<'_, f64>) -> Result<Array1<'static, Point2>, Failure>;
    fn grid_d1(
        &self,
        p: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure>;
    fn grid_d2(
        &self,
        p: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure>;
    fn grid_d3(
        &self,
        p: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure>;
    fn grid_dn(&self, p: &Array1<'_, f64>, n: i32) -> Result<Array1<'static, Vector2>, Failure>;
}
macro_rules! grid_evaluator {
    ($class:ty, $wrap:path) => {
        impl GridEvaluator for $class {
            fn grid(&self, p: &Array1<'_, f64>) -> Result<Array1<'static, Point2>, Failure> {
                $wrap(self.evaluate_grid(p))
            }
            fn grid_d1(
                &self,
                p: &Array1<'_, f64>,
            ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
                $wrap(self.evaluate_grid_d1(p))
            }
            fn grid_d2(
                &self,
                p: &Array1<'_, f64>,
            ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
                $wrap(self.evaluate_grid_d2(p))
            }
            fn grid_d3(
                &self,
                p: &Array1<'_, f64>,
            ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
                $wrap(self.evaluate_grid_d3(p))
            }
            fn grid_dn(
                &self,
                p: &Array1<'_, f64>,
                n: i32,
            ) -> Result<Array1<'static, Vector2>, Failure> {
                $wrap(self.evaluate_grid_dn(p, n))
            }
        }
    };
}
fn same<T>(value: Result<T, Failure>) -> Result<T, Failure> {
    value
}
grid_evaluator!(Geom2dGridEvalLine, Ok);
grid_evaluator!(Geom2dGridEvalCircle, same);
grid_evaluator!(Geom2dGridEvalEllipse, same);
grid_evaluator!(Geom2dGridEvalHyperbola, same);
grid_evaluator!(Geom2dGridEvalParabola, same);
grid_evaluator!(Geom2dGridEvalBezierCurve, same);
grid_evaluator!(Geom2dGridEvalBSplineCurve, same);
grid_evaluator!(Geom2dGridEvalOffsetCurve, same);
grid_evaluator!(Geom2dGridEvalOtherCurve<'_>, same);
grid_evaluator!(Geom2dGridEvalCurve<'_>, same);

/// g2_grid_all: EvaluateGrid, D1, D2, D3, then DN for N = -1..7.
fn all(e: &dyn GridEvaluator, p: &Array1<'_, f64>) {
    array(e.grid(p));
    array(e.grid_d1(p));
    array(e.grid_d2(p));
    array(e.grid_d3(p));
    for n in -1..=7 {
        array(e.grid_dn(p, n));
    }
}

fn dispatch(e: &Geom2dGridEvalCurve<'_>, p: &Array1<'_, f64>) {
    int(e.get_type() as i32);
    all(e, p);
}

/// The specialized evaluators that take a typed handle and expose `geometry()`.
trait Specialized<G: RuntimeTyped>: GridEvaluator + Sized {
    fn build(geometry: Handle<G>) -> Self;
    fn is_null(&self) -> bool;
}
macro_rules! specialized {
    ($class:ty, $geom:ty) => {
        impl Specialized<$geom> for $class {
            fn build(geometry: Handle<$geom>) -> Self {
                <$class>::new(geometry)
            }
            fn is_null(&self) -> bool {
                self.geometry().is_null()
            }
        }
    };
}
specialized!(Geom2dGridEvalLine, Geom2dLine);
specialized!(Geom2dGridEvalCircle, Geom2dCircle);
specialized!(Geom2dGridEvalEllipse, Geom2dEllipse);
specialized!(Geom2dGridEvalHyperbola, Geom2dHyperbola);
specialized!(Geom2dGridEvalParabola, Geom2dParabola);
specialized!(Geom2dGridEvalBezierCurve, Geom2dBezierCurve);
specialized!(Geom2dGridEvalBSplineCurve, Geom2dBSplineCurve);
specialized!(Geom2dGridEvalOffsetCurve, Geom2dOffsetCurve);

fn direct_as<G: RuntimeTyped, E: Specialized<G>>(
    c: &Handle<dyn Geom2dCurve>,
    p: &Array1<'_, f64>,
) -> bool {
    let g: Handle<G> = c.cast::<G>();
    if g.is_null() {
        return false;
    }
    let e = E::build(g);
    flag(e.is_null());
    all(&e, p);
    true
}

/// g2_grid_direct: the specialized evaluator of the curve's own class.
fn direct(c: &Handle<dyn Geom2dCurve>, p: &Array1<'_, f64>) {
    let _ = direct_as::<Geom2dLine, Geom2dGridEvalLine>(c, p)
        || direct_as::<Geom2dCircle, Geom2dGridEvalCircle>(c, p)
        || direct_as::<Geom2dEllipse, Geom2dGridEvalEllipse>(c, p)
        || direct_as::<Geom2dHyperbola, Geom2dGridEvalHyperbola>(c, p)
        || direct_as::<Geom2dParabola, Geom2dGridEvalParabola>(c, p)
        || direct_as::<Geom2dBezierCurve, Geom2dGridEvalBezierCurve>(c, p)
        || direct_as::<Geom2dBSplineCurve, Geom2dGridEvalBSplineCurve>(c, p)
        || direct_as::<Geom2dOffsetCurve, Geom2dGridEvalOffsetCurve>(c, p);
}

/// g2_grid: the dispatcher on the handle and on an adaptor, the specialized evaluator
/// and the fallback.
pub fn grid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let p = params(&mut r);
    dispatch(&Geom2dGridEvalCurve::from_curve(&c), &p);
    let a = Geom2dAdaptorCurve::with_curve(&c).unwrap();
    dispatch(&Geom2dGridEvalCurve::from_adaptor(&a).unwrap(), &p);
    direct(&c, &p);
    all(&Geom2dGridEvalOtherCurve::new(&a), &p);
    end();
}

/// g2_grid_adaptor: the dispatcher on any adaptor, then the fallback.
pub fn grid_adaptor_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(a) = built_adaptor(&mut r) else {
        return;
    };
    let p = params(&mut r);
    let adaptor: &dyn Adaptor2dCurve2d = &*a;
    attempt(|| {
        dispatch(&Geom2dGridEvalCurve::from_adaptor(adaptor)?, &p);
        Ok(())
    });
    all(&Geom2dGridEvalOtherCurve::new(adaptor), &p);
    end();
}

/// g2_grid_evalrep: grids of a curve that carries an evaluation representation.
pub fn grid_evalrep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = built(&mut r) else { return };
    let rep = curve(&mut r).unwrap();
    let d = desc(&mut r, rep);
    let p = params(&mut r);
    attempt(|| set_rep(&c, &d));
    dispatch(&Geom2dGridEvalCurve::from_curve(&c), &p);
    let a = Geom2dAdaptorCurve::with_curve(&c).unwrap();
    dispatch(&Geom2dGridEvalCurve::from_adaptor(&a).unwrap(), &p);
    end();
}

fn null_as<G: RuntimeTyped, E: Specialized<G>>(p: &Array1<'_, f64>) {
    let e = E::build(Handle::default());
    flag(e.is_null());
    all(&e, p);
}

/// g2_grid_null: every evaluator on a null curve, the dispatcher on a null handle and
/// on an empty adaptor.
pub fn grid_null_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = params(&mut r);
    null_as::<Geom2dLine, Geom2dGridEvalLine>(&p);
    null_as::<Geom2dCircle, Geom2dGridEvalCircle>(&p);
    null_as::<Geom2dEllipse, Geom2dGridEvalEllipse>(&p);
    null_as::<Geom2dHyperbola, Geom2dGridEvalHyperbola>(&p);
    null_as::<Geom2dParabola, Geom2dGridEvalParabola>(&p);
    null_as::<Geom2dBezierCurve, Geom2dGridEvalBezierCurve>(&p);
    null_as::<Geom2dBSplineCurve, Geom2dGridEvalBSplineCurve>(&p);
    null_as::<Geom2dOffsetCurve, Geom2dGridEvalOffsetCurve>(&p);
    dispatch(&Geom2dGridEvalCurve::from_curve(&Handle::default()), &p);
    let a = Geom2dAdaptorCurve::default();
    dispatch(&Geom2dGridEvalCurve::from_adaptor(&a).unwrap(), &p);
    end();
}
