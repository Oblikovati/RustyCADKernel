// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_GuideTrihedronPlan.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_GuideTrihedronPlan`: a frame whose normal points to the intersection
//! of the guide with the normal plane of the path.
use super::frenet::GeomFillFrenet;
use super::guide_plan_init::init_poles;
use super::pipe_error::GeomFillPipeError;
use super::plan_func::GeomFillPlanFunc;
use super::sweep_failures::live;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use super::trihedron_with_guide::{GeomFillTrihedronWithGuide, GeomFillTrihedronWithGuideBase};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkmath::math::{FunctionRoot, FunctionWithDerivative};
use std::f64::consts::PI;

/// The fields `Init`, `SetInterval` and the evaluators write: the class's own
/// `myCurve` and `myTrimmed` (hiding the base members), the guide parameters of the
/// sampled path points (`Pole`), the start `X(1)`, the bounds `Inf(1)`, `Sup(1)`
/// and `myStatus`.
#[derive(Clone, Default)]
pub(crate) struct PlanFields {
    pub my_curve: Handle<dyn Adaptor3dCurve>,
    pub my_trimmed: Handle<dyn Adaptor3dCurve>,
    pub pole: Vec<Point2>,
    pub x: f64,
    pub inf: f64,
    pub sup: f64,
    pub status: GeomFillPipeError,
}

/// `GeomFill_GuideTrihedronPlan`: the guided law by normal planes.
///
/// The class declares its own `myCurve` and `myTrimmed`, hiding the base members,
/// which stay null (so the base `GetInterval` meets a null curve, a crash natively);
/// its `D2` always answers false, and it never sets the current guide point.
/// `let law = GeomFillGuideTrihedronPlan::new(&guide)?; law.set_curve(&path)?;`.
pub struct GeomFillGuideTrihedronPlan {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    guide_base: GeomFillTrihedronWithGuideBase,
    x_tol: f64,
    pub(crate) frenet: Handle<GeomFillFrenet>,
    pub(crate) fields: SharedState<PlanFields>,
}
impl_transient!(
    GeomFillGuideTrihedronPlan,
    "GeomFill_GuideTrihedronPlan",
    dyn GeomFillTrihedronWithGuide,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw, dyn GeomFillTrihedronWithGuide]
);

/// The number of sampled path points (`myNbPts`).
pub(crate) const NB_PTS: usize = 20;

impl GeomFillGuideTrihedronPlan {
    /// `GeomFill_GuideTrihedronPlan(theGuide)`: `XTol(1)` is the guide resolution of
    /// 1e-6.
    pub fn new(guide: &Handle<dyn Adaptor3dCurve>) -> Result<Self, Failure> {
        let x_tol = live(guide, "GeomFill_GuideTrihedronPlan", "theGuide")?.resolution(1.0e-6)?;
        let law = Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            guide_base: GeomFillTrihedronWithGuideBase::default(),
            x_tol,
            frenet: Handle::new(GeomFillFrenet::new()),
            fields: SharedState::new(PlanFields {
                pole: vec![Point2::default(); NB_PTS],
                ..PlanFields::default()
            }),
        };
        law.guide_base.edit(|g| {
            g.guide = guide.clone();
            g.trim_g = guide.clone();
        });
        Ok(law)
    }

    pub(crate) fn frenet(&self) -> &GeomFillFrenet {
        self.frenet
            .get()
            .expect("the constructor creates the Frenet law")
    }

    pub(crate) fn guide_curve(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let guide = self.guide_base.fields().guide;
        live(&guide, member, "myGuide")?;
        Ok(guide)
    }

    pub(crate) fn set_trim_g(&self, trim_g: Handle<dyn Adaptor3dCurve>) {
        self.guide_base.edit(|g| g.trim_g = trim_g);
    }

    fn my_curve(&self, member: &str) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let curve = self.fields.read().my_curve.clone();
        live(&curve, member, "myCurve")?;
        Ok(curve)
    }

    /// `InitX(Param)`: the start of the guide parameter search, interpolated in the
    /// sampled poles and brought into the guide period.
    fn init_x(&self, param: f64) -> Result<(), Failure> {
        let pole = self.fields.read().pole.clone();
        let mut x = interpolated_guide_parameter(&pole, param);
        let guide = self.guide_curve("GeomFill_GuideTrihedronPlan::InitX")?;
        let g = guide.get().expect("checked");
        if g.is_periodic()? {
            x = elclib::in_period(x, g.first_parameter()?, g.last_parameter()?);
        }
        self.fields.write().x = x;
        Ok(())
    }

    /// The root of the plane function from `X(1)` within `[Inf(1), Sup(1)]` (50
    /// iterations), or `None` after recording PlaneNotIntersectGuide.
    fn guide_root(&self, func: &mut GeomFillPlanFunc) -> Result<Option<f64>, Failure> {
        let f = self.fields.read().clone();
        let result = FunctionRoot::bounded(func, f.x, self.x_tol, f.inf, f.sup, 50)?;
        func.take_failure()?;
        if !result.is_done() {
            self.fields.write().status = GeomFillPipeError::PlaneNotIntersectGuide;
            return Ok(None);
        }
        Ok(Some(result.root()?))
    }
}

/// The continuity of the path that gives continuity `shape` to the law: one order
/// more up to C2, CN beyond.
fn plan_shape(shape: Shape) -> Shape {
    match shape {
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        _ => Shape::CN,
    }
}

/// The guide parameter at `param`, linear between the two poles around it.
fn interpolated_guide_parameter(pole: &[Point2], param: f64) -> f64 {
    let at = |i: usize| pole[i - 1];
    let (mut ideb, mut ifin) = (1, pole.len());
    if param == at(ideb).x() {
        ifin = ideb + 1;
    }
    if param == at(ifin).x() {
        ideb = ifin - 1;
    }
    while ideb + 1 != ifin {
        let idemi = (ideb + ifin) / 2;
        let valeur = at(idemi).x();
        if valeur < param {
            ideb = idemi;
        } else if valeur > param {
            ifin = idemi;
        } else {
            ideb = idemi;
            ifin = ideb + 1;
        }
    }
    let (t1, t2) = (at(ideb).x(), at(ifin).x());
    let diff = t2 - t1;
    if diff > 1.0e-7 {
        let (b, a) = ((param - t1) / diff, (t2 - param) / diff);
        return at(ideb).y() * a + at(ifin).y() * b;
    }
    (at(ideb).y() + at(ifin).y()) / 2.0
}

impl GeomFillTrihedronLaw for GeomFillGuideTrihedronPlan {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    /// Loads the path and samples the guide intersections; always true.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.fields.write().my_curve = curve.clone();
        if curve.is_some() {
            init_poles(self)?;
        }
        Ok(true)
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillGuideTrihedronPlan::new(&self.guide_base.fields().guide)?;
        copy.set_curve(&self.fields.read().my_curve.clone())?;
        Ok(Handle::new(copy).cast())
    }
    fn error_status(&self) -> GeomFillPipeError {
        self.fields.read().status
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::D0")?;
        let p = curve.get().expect("checked").d0(param)?;
        self.frenet().d0(param, frame)?;
        self.init_x(param)?;
        let guide = self.guide_curve("GeomFill_GuideTrihedronPlan::D0")?;
        let mut e = GeomFillPlanFunc::new(p, frame.tangent, &guide);
        let Some(res) = self.guide_root(&mut e)? else {
            return Ok(false);
        };
        let trim_g = self.guide_base.fields().trim_g;
        let pprime = live(&trim_g, "GeomFill_GuideTrihedronPlan::D0", "myTrimG")?.value(res)?;
        frame.normal = Vector3::between(p, pprime).normalized()?;
        frame.binormal = frame.tangent.crossed(frame.normal);
        frame.binormal.normalize()?;
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::D1")?;
        let (p, to) = curve.get().expect("checked").d1(param)?;
        self.frenet().d1(param, frame)?;
        self.init_x(param)?;
        let guide = self.guide_curve("GeomFill_GuideTrihedronPlan::D1")?;
        let mut e = GeomFillPlanFunc::new(p, frame.tangent, &guide);
        let Some(res) = self.guide_root(&mut e)? else {
            return Ok(false);
        };
        let trim_g = self.guide_base.fields().trim_g;
        let (pg, tg) = live(&trim_g, "GeomFill_GuideTrihedronPlan::D1", "myTrimG")?.d1(res)?;
        let mut n = Vector3::between(p, pg);
        let mut norm = n.magnitude();
        if norm < 1.0e-12 {
            norm = 1.0;
        }
        n /= norm;
        frame.normal = n;
        frame.binormal = frame.tangent.crossed(frame.normal);
        let mut dedx = 0.0;
        e.derivative(res, &mut dedx);
        e.take_failure()?;
        let dedt = e.dedt(res, to, frame.d_tangent)?;
        let dtg_dt = -dedt / dedx;
        let mut dn = Vector3::default();
        dn.set_linear_form(dtg_dt, tg, -1.0, to);
        frame.d_normal.set_scaled_sum(-(n * dn), n, dn);
        frame.d_normal /= norm;
        frame.d_binormal.set_sum(
            frame.tangent.crossed(frame.d_normal),
            frame.d_tangent.crossed(frame.normal),
        );
        Ok(true)
    }
    /// The Frenet D2 of the path; always false, as natively.
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::D2")?;
        curve.get().expect("checked").d2(param)?;
        self.frenet().d2(param, frame)?;
        Ok(false)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::NbIntervals")?;
        curve
            .get()
            .expect("checked")
            .nb_intervals(plan_shape(shape))
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::Intervals")?;
        curve
            .get()
            .expect("checked")
            .intervals(bounds, plan_shape(shape))
    }
    /// Trims the class's own `myTrimmed` (never read afterwards).
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::SetInterval")?;
        let trimmed = curve
            .get()
            .expect("checked")
            .trim(first, last, precision::confusion())?;
        self.fields.write().my_trimmed = trimmed;
        Ok(())
    }
    /// The mean of 20 frames over `[First, Last - Delta]`, not normalized.
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::GetAverageLaw")?;
        let c = curve.get().expect("checked");
        let delta = (c.last_parameter()? - c.first_parameter()?) / 20.001;
        let (mut sum, mut frame) = (TrihedronD0::default(), TrihedronD0::default());
        for ii in 1..=20 {
            let t = c.first_parameter()? + f64::from(ii - 1) * delta;
            self.d0(t, &mut frame)?;
            sum.tangent += frame.tangent;
            sum.normal += frame.normal;
            sum.binormal += frame.binormal;
        }
        sum.tangent /= 20.0;
        sum.normal /= 20.0;
        sum.binormal /= 20.0;
        Ok(sum)
    }
    /// True for a line path along a line guide.
    fn is_constant(&self) -> Result<bool, Failure> {
        let curve = self.my_curve("GeomFill_GuideTrihedronPlan::IsConstant")?;
        let guide = self.guide_curve("GeomFill_GuideTrihedronPlan::IsConstant")?;
        let (c, g) = (curve.get().expect("checked"), guide.get().expect("checked"));
        if c.get_type()? == CurveType::Line && g.get_type()? == CurveType::Line {
            let angle = c.line()?.angle(g.line()?);
            if angle < 1.0e-12 || (2.0 * PI - angle) < 1.0e-12 {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl GeomFillTrihedronWithGuide for GeomFillGuideTrihedronPlan {
    fn guide_base(&self) -> &GeomFillTrihedronWithGuideBase {
        &self.guide_base
    }
    /// `Origine`: nothing, as natively.
    fn origine(&self, _param1: f64, _param2: f64) -> Result<(), Failure> {
        Ok(())
    }
}
