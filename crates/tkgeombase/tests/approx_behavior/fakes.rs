//! Named fakes: a MultiLine with scripted Approx members, a continuous function and
//! a sweep section.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkgeombase::app_cont::{Function, FunctionBase};
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkgeombase::approx::{Status, SweepD1, SweepD2, SweepFunction};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::sync::Mutex;

/// What the fake line answers to the Approx members.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineScript {
    /// AppDef_MyLineTool: no point added, the same line, no extra point.
    AppDef,
    /// PointsAdded while the line has fewer than `cap` points; MakeMLBetween doubles
    /// the sampling of the range (on the parabola) and MakeMLOneMorePoint inserts a
    /// midpoint.
    Refine { cap: i32 },
    /// Always NoApproximation.
    Refuse,
}

/// A MultiLine of one 3D point per multipoint, sampled on `(t, t^2, 0)` at the
/// stored parameters, numbered from `first`.
#[derive(Clone, Debug)]
pub struct ParabolaLine {
    pub first: i32,
    pub params: Vec<f64>,
    pub script: LineScript,
    pub tangents: bool,
}

impl ParabolaLine {
    pub fn uniform(n: i32, script: LineScript) -> Self {
        let params = (0..n).map(|i| f64::from(i) / f64::from(n - 1)).collect();
        Self {
            first: 1,
            params,
            script,
            tangents: false,
        }
    }

    fn param(&self, index: i32) -> Result<f64, Failure> {
        usize::try_from(index - self.first)
            .ok()
            .and_then(|slot| self.params.get(slot))
            .copied()
            .ok_or_else(|| Failure::out_of_range("ParabolaLine"))
    }

    fn point(t: f64) -> Point3 {
        Point3::new(t, t * t, 0.0)
    }
}

impl LineTool for ParabolaLine {
    fn first_point(&self) -> i32 {
        self.first
    }
    fn last_point(&self) -> Result<i32, Failure> {
        Ok(self.first + self.params.len() as i32 - 1)
    }
    fn nb_p2d(&self) -> Result<i32, Failure> {
        Ok(0)
    }
    fn nb_p3d(&self) -> Result<i32, Failure> {
        Ok(1)
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        let low = points.lower();
        points[low] = Self::point(self.param(index)?);
        Ok(())
    }
    fn value_2d(&self, index: i32, _: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        self.param(index).map(|_| ())
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        _: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        self.value_3d(index, points)
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        let t = self.param(index)?;
        if !self.tangents {
            return Ok(false);
        }
        let low = tangents.lower();
        tangents[low] = Vector3::new(1.0, 2.0 * t, 0.0);
        Ok(true)
    }
    fn tangency_2d(&self, index: i32, _: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        self.param(index).map(|_| false)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        _: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        self.tangency_3d(index, tangents)
    }
    fn curvature_3d(&self, _: i32, _: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature_2d(&self, _: i32, _: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature(
        &self,
        _: i32,
        _: &mut Array1<'_, Vector3>,
        _: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn what_status(&self, _: i32, _: i32) -> Result<Status, Failure> {
        Ok(match self.script {
            LineScript::AppDef => Status::NoPointsAdded,
            LineScript::Refine { cap } if (self.params.len() as i32) < cap => Status::PointsAdded,
            LineScript::Refine { .. } => Status::NoPointsAdded,
            LineScript::Refuse => Status::NoApproximation,
        })
    }
    fn make_ml_between(&self, i1: i32, i2: i32, nb_p_min: i32) -> Result<Self, Failure> {
        if self.script == LineScript::AppDef {
            return Ok(self.clone());
        }
        let n: i32 = (2 * (i2 - i1) + 1).max(nb_p_min);
        let (t1, t2) = (self.param(i1)?, self.param(i2)?);
        let params = (0..n)
            .map(|k| t1 + (t2 - t1) * f64::from(k) / f64::from(n - 1))
            .collect();
        Ok(Self {
            params,
            ..self.clone()
        })
    }
    fn make_ml_one_more_point(
        &self,
        i1: i32,
        i2: i32,
        index_bad: i32,
    ) -> Result<Option<Self>, Failure> {
        if !matches!(self.script, LineScript::Refine { .. }) || index_bad <= i1 || index_bad > i2 {
            return Ok(None);
        }
        let mut params = Vec::new();
        for i in i1..=i2 {
            if i == index_bad {
                params.push(0.5 * (self.param(i - 1)? + self.param(i)?));
            }
            params.push(self.param(i)?);
        }
        Ok(Some(Self {
            params,
            ..self.clone()
        }))
    }
}

/// A continuous function: one 3D point `(u, u^2, u^3)` on `[first, last]`.
pub struct CubicFunction {
    base: FunctionBase,
    range: [f64; 2],
}

impl CubicFunction {
    pub fn new(first: f64, last: f64) -> Self {
        let mut base = FunctionBase::new();
        base.set_number_of_points(1, 0);
        Self {
            base,
            range: [first, last],
        }
    }
}

impl Function for CubicFunction {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    fn first_parameter(&self) -> f64 {
        self.range[0]
    }
    fn last_parameter(&self) -> f64 {
        self.range[1]
    }
    fn value(&self, u: f64, _: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
        pnt[1] = Point3::new(u, u * u, u * u * u);
        true
    }
    fn d1(&self, u: f64, _: &mut Array1<'_, Vector2>, vec: &mut Array1<'_, Vector3>) -> bool {
        vec[1] = Vector3::new(1.0, 2.0 * u, 3.0 * u * u);
        true
    }
}

/// A sweep section of two poles moving along `(t, t^2, 0)` and `(t, t^2, 1)`, with
/// an optional pcurve `(t, 0)`; D1, D2 and Resolution are left to the defaults
/// unless `derivatives` is set.
pub struct TwoPoleSweep {
    parent: StandardTransient,
    pub nb2d: i32,
    pub derivatives: bool,
    pub intervals_set: Mutex<i32>,
}
impl_transient!(
    TwoPoleSweep,
    "TwoPoleSweep",
    dyn SweepFunction,
    parent,
    interfaces: [dyn SweepFunction]
);

impl TwoPoleSweep {
    pub fn new(nb2d: i32, derivatives: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            nb2d,
            derivatives,
            intervals_set: Mutex::new(0),
        }
    }
}

impl SweepFunction for TwoPoleSweep {
    fn d0(
        &self,
        t: f64,
        _: f64,
        _: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        for k in 1..=2 {
            poles[k] = Point3::new(t, t * t, f64::from(k - 1));
            weights[k] = 1.0;
        }
        for j in 1..=self.nb2d {
            poles2d[j] = Point2::new(t, 0.0);
        }
        Ok(true)
    }
    fn d1(&self, t: f64, first: f64, last: f64, d: SweepD1<'_, '_>) -> Result<bool, Failure> {
        if !self.derivatives {
            return Err(Failure::with_type("Standard_NotImplemented", "D1", ""));
        }
        self.d0(t, first, last, d.poles, d.poles2d, d.weights)?;
        for k in 1..=2 {
            d.d_poles[k] = Vector3::new(1.0, 2.0 * t, 0.0);
            d.d_weights[k] = 0.0;
        }
        for j in 1..=self.nb2d {
            d.d_poles2d[j] = Vector2::new(1.0, 0.0);
        }
        Ok(true)
    }
    fn d2(&self, t: f64, first: f64, last: f64, d: SweepD2<'_, '_>) -> Result<bool, Failure> {
        if !self.derivatives {
            return Err(Failure::with_type("Standard_NotImplemented", "D2", ""));
        }
        let nb2d = self.nb2d;
        self.d1(t, first, last, d.d1)?;
        for k in 1..=2 {
            d.d2_poles[k] = Vector3::new(0.0, 2.0, 0.0);
            d.d2_weights[k] = 0.0;
        }
        for j in 1..=nb2d {
            d.d2_poles2d[j] = Vector2::new(0.0, 0.0);
        }
        Ok(true)
    }
    fn nb_2d_curves(&self) -> Result<i32, Failure> {
        Ok(self.nb2d)
    }
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok((2, 2, 1))
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        (knots[1], knots[2]) = (0.0, 1.0);
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        (mults[1], mults[2]) = (2, 2);
        Ok(())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn nb_intervals(&self, _: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn intervals(&self, t: &mut Array1<'_, f64>, _: Shape) -> Result<(), Failure> {
        (t[1], t[2]) = (0.0, 1.0);
        Ok(())
    }
    fn set_interval(&self, _: f64, _: f64) -> Result<(), Failure> {
        *self.intervals_set.lock().unwrap() += 1;
        Ok(())
    }
    fn resolution(&self, _: i32, tol: f64) -> Result<(f64, f64), Failure> {
        Ok((tol, tol))
    }
    fn get_tolerance(
        &self,
        _: f64,
        surf: f64,
        _: f64,
        t: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        t.init(surf);
        Ok(())
    }
    fn set_tolerance(&self, _: f64, _: f64) -> Result<(), Failure> {
        Ok(())
    }
}
