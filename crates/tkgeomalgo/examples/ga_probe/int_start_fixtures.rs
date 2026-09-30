//! Probe ports of the IntPatch classes that instantiate the IntStart templates
//! (IntPatch is not ported yet), the twins of the classes used by
//! tools/oracle/tkgeomalgo_probe_intstart.cxx: IntPatch_ArcFunction, IntPatch_HInterTool
//! (the members the templates and the function call), the IntPatch path point and
//! segment, the probe domain with an edge and the probe tool of seen arcs.
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_start::{
    TheArc, TheArcFunction, ThePathPoint, TheSOBTool, TheSegment, TheTopolTool, TheVertex,
};
use rustycad_tkgeomalgo::int_surf::IntSurfQuadric;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use std::cell::RefCell;

fn out_of_range() -> Failure {
    Failure::with_type("Standard_OutOfRange", "", "")
}

/// IntPatch_ArcFunction: the signed distance of the quadric along an arc of a surface.
#[derive(Default)]
pub struct ProbeArcFunction {
    arc: TheArc,
    surf: Handle<dyn Adaptor3dSurface>,
    quad: IntSurfQuadric,
    ptsol: Point3,
    seqpt: Vec<Point3>,
    capture: FailureCapture,
}

impl ProbeArcFunction {
    pub fn set_quadric(&mut self, q: &IntSurfQuadric) {
        self.quad = *q;
        self.seqpt.clear();
    }
    pub fn set_surface(&mut self, s: &Handle<dyn Adaptor3dSurface>) {
        self.surf = s.clone();
        self.seqpt.clear();
    }
    fn eval(&mut self, x: f64) -> Result<f64, Failure> {
        let p2d: Point2 = self.arc.value(x)?;
        self.ptsol = self.surf.d0(p2d.x(), p2d.y())?;
        Ok(self.quad.distance(self.ptsol))
    }
    // `v.SetLinearForm(d2d.X(), d1u, d2d.Y(), d1v)`: the tangent of the arc on the surface.
    fn tangent(&mut self, x: f64) -> Result<Vector3, Failure> {
        let (p2d, d2d) = self.arc.d1(x)?;
        let d1 = self.surf.d1(p2d.x(), p2d.y())?;
        self.ptsol = d1.point;
        let mut v = Vector3::default();
        v.set_linear_form(d2d.x(), d1.d1u, d2d.y(), d1.d1v);
        Ok(v)
    }
    fn eval_derivative(&mut self, x: f64) -> Result<f64, Failure> {
        let v: Vector3 = self.tangent(x)?;
        Ok(v.dot(self.quad.gradient(self.ptsol)))
    }
    fn eval_values(&mut self, x: f64) -> Result<(f64, f64), Failure> {
        let v1: Vector3 = self.tangent(x)?;
        // The probe quadrics are real ones, whose ValAndGrad always writes.
        let (f, v2) = self.quad.val_and_grad(self.ptsol).unwrap_or_default();
        Ok((f, v1.dot(v2)))
    }
}

impl Function for ProbeArcFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = self.eval(x);
        self.capture.keep(value).map(|f| *result = f).is_some()
    }
    fn state_number(&mut self) -> i32 {
        self.get_state_number()
    }
}

impl FunctionWithDerivative for ProbeArcFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let value = self.eval_derivative(x);
        self.capture.keep(value).map(|d| *derivative = d).is_some()
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let values = self.eval_values(x);
        self.capture
            .keep(values)
            .map(|(f, d)| (*value, *derivative) = (f, d))
            .is_some()
    }
}

impl TheArcFunction for ProbeArcFunction {
    fn set_arc(&mut self, arc: &TheArc) {
        self.arc = arc.clone();
        self.seqpt.clear();
    }
    fn checked_value(&mut self, x: f64) -> Result<Option<f64>, Failure> {
        Ok(Some(self.eval(x)?))
    }
    fn checked_values(&mut self, x: f64) -> Result<Option<(f64, f64)>, Failure> {
        Ok(Some(self.eval_values(x)?))
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        let su: i32 = ProbeHInterTool::nb_samples_u(&self.surf)?;
        let sv: i32 = ProbeHInterTool::nb_samples_v(&self.surf)?;
        Ok(su
            .max(sv)
            .max(ProbeHInterTool::nb_samples_on_arc(&self.arc)?))
    }
    fn get_state_number(&mut self) -> i32 {
        self.seqpt.push(self.ptsol);
        self.seqpt.len() as i32
    }
    fn valpoint(&self, index: i32) -> Result<Point3, Failure> {
        let slot = usize::try_from(index).ok().and_then(|i| i.checked_sub(1));
        slot.and_then(|i| self.seqpt.get(i))
            .copied()
            .ok_or_else(out_of_range)
    }
    fn quadric(&self) -> &IntSurfQuadric {
        &self.quad
    }
    fn surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.surf.clone()
    }
    fn last_computed_point(&self) -> Point3 {
        self.ptsol
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
}

/// IntPatch_HInterTool: the sample counts, and the members of TheSOBTool on arcs that
/// are never seen.
pub struct ProbeHInterTool;

impl ProbeHInterTool {
    pub fn nb_samples_v(s: &Handle<dyn Adaptor3dSurface>) -> Result<i32, Failure> {
        Ok(match s.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_v_poles()?,
            SurfaceType::BSplineSurface => {
                spline_samples(s.nb_v_knots()?, s.v_degree()?, s.is_v_rational()?)
            }
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion => 15,
            _ => 10,
        })
    }
    pub fn nb_samples_u(s: &Handle<dyn Adaptor3dSurface>) -> Result<i32, Failure> {
        Ok(match s.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + s.nb_u_poles()?,
            SurfaceType::BSplineSurface => {
                spline_samples(s.nb_u_knots()?, s.u_degree()?, s.is_u_rational()?)
            }
            SurfaceType::Torus => 20,
            _ => 10,
        })
    }
    pub fn nb_samples_on_arc(a: &TheArc) -> Result<i32, Failure> {
        Ok(match a.get_type()? {
            CurveType::Line => 2,
            CurveType::Circle | CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola => {
                10
            }
            CurveType::BezierCurve => a.nb_poles()?,
            CurveType::BSplineCurve => 2 + a.nb_knots()? * a.degree()?,
            _ => 10,
        })
    }
}

fn spline_samples(nb_knots: i32, degree: i32, rational: bool) -> i32 {
    let mut nbs: i32 = nb_knots * degree;
    if !rational {
        nbs *= 2;
    }
    nbs.max(4)
}

impl TheSOBTool for ProbeHInterTool {
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        vertex.parameter(arc)
    }
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        Ok(vertex.resolution(arc))
    }
    fn has_been_seen(_: &TheArc) -> Result<bool, Failure> {
        Ok(false)
    }
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure> {
        Ok((arc.first_parameter()?, arc.last_parameter()?))
    }
    fn nb_points(_: &TheArc) -> Result<i32, Failure> {
        Ok(0)
    }
    fn value(_: &TheArc, _: i32) -> Result<(Point3, f64, f64), Failure> {
        Err(out_of_range())
    }
    fn is_vertex(_: &TheArc, _: i32) -> Result<bool, Failure> {
        Ok(false)
    }
    fn vertex(_: &TheArc, _: i32) -> Result<TheVertex, Failure> {
        Err(out_of_range())
    }
    fn nb_segments(_: &TheArc) -> Result<i32, Failure> {
        Ok(0)
    }
    fn has_first_point(_: &TheArc, _: i32) -> Result<Option<i32>, Failure> {
        Err(out_of_range())
    }
    fn has_last_point(_: &TheArc, _: i32) -> Result<Option<i32>, Failure> {
        Err(out_of_range())
    }
    fn is_all_solution(_: &TheArc) -> Result<bool, Failure> {
        Ok(false)
    }
}

/// The known points `(x y z tol prm isvertex)` and segments of the seen arcs.
#[derive(Default)]
pub struct SeenTable {
    pub pts: Vec<[f64; 6]>,
    pub segs: Vec<(i32, i32)>,
    pub all: bool,
}

thread_local! {
    pub static SEEN: RefCell<SeenTable> = RefCell::new(SeenTable::default());
}

/// GaProbeSeenTool: TheSOBTool whose every arc has been seen.
pub struct ProbeSeenTool;

fn known(i: i32) -> Result<[f64; 6], Failure> {
    SEEN.with(|t| {
        let t = t.borrow();
        let slot = usize::try_from(i).ok().and_then(|i| i.checked_sub(1));
        slot.and_then(|k| t.pts.get(k))
            .copied()
            .ok_or_else(out_of_range)
    })
}

fn seg(i: i32) -> Result<(i32, i32), Failure> {
    SEEN.with(|t| {
        let t = t.borrow();
        let slot = usize::try_from(i).ok().and_then(|i| i.checked_sub(1));
        slot.and_then(|k| t.segs.get(k))
            .copied()
            .ok_or_else(out_of_range)
    })
}

impl TheSOBTool for ProbeSeenTool {
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        ProbeHInterTool::parameter(vertex, arc)
    }
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        ProbeHInterTool::tolerance(vertex, arc)
    }
    fn has_been_seen(_: &TheArc) -> Result<bool, Failure> {
        Ok(true)
    }
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure> {
        ProbeHInterTool::bounds(arc)
    }
    fn nb_points(_: &TheArc) -> Result<i32, Failure> {
        Ok(SEEN.with(|t| t.borrow().pts.len() as i32))
    }
    fn value(_: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        let k = known(index)?;
        Ok((Point3::new(k[0], k[1], k[2]), k[3], k[4]))
    }
    fn is_vertex(_: &TheArc, index: i32) -> Result<bool, Failure> {
        Ok(known(index)?[5] != 0.0)
    }
    fn vertex(_: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        let k = known(index)?;
        let v = Adaptor3dHVertex::new(Point2::new(k[4], 0.0), Orientation::Forward, k[3]);
        Ok(Handle::new(v))
    }
    fn nb_segments(_: &TheArc) -> Result<i32, Failure> {
        Ok(SEEN.with(|t| t.borrow().segs.len() as i32))
    }
    fn has_first_point(_: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        let first: i32 = seg(index)?.0;
        Ok((first != 0).then_some(first))
    }
    fn has_last_point(_: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        let last: i32 = seg(index)?.1;
        Ok((last != 0).then_some(last))
    }
    fn is_all_solution(_: &TheArc) -> Result<bool, Failure> {
        Ok(SEEN.with(|t| t.borrow().all))
    }
}

/// IntPatch_ThePathPointOfTheSOnBounds.
#[derive(Clone)]
pub struct ProbePathPoint {
    point: Point3,
    tol: f64,
    isnew: bool,
    vtx: TheVertex,
    arc: TheArc,
    param: f64,
}

impl Default for ProbePathPoint {
    fn default() -> Self {
        Self {
            point: Point3::default(),
            tol: 0.0,
            isnew: true,
            vtx: Handle::default(),
            arc: Handle::default(),
            param: 0.0,
        }
    }
}

impl ProbePathPoint {
    pub fn value(&self) -> Point3 {
        self.point
    }
    pub fn tolerance(&self) -> f64 {
        self.tol
    }
}

impl ThePathPoint for ProbePathPoint {
    fn on_vertex(p: Point3, tol: f64, vertex: &TheVertex, arc: &TheArc, param: f64) -> Self {
        let mut point = Self::default();
        point.set_value_on_vertex(p, tol, vertex, arc, param);
        point
    }
    fn on_arc(p: Point3, tol: f64, arc: &TheArc, param: f64) -> Self {
        let mut point = Self::default();
        point.set_value_on_arc(p, tol, arc, param);
        point
    }
    fn set_value_on_vertex(&mut self, p: Point3, tol: f64, v: &TheVertex, a: &TheArc, param: f64) {
        (self.isnew, self.point, self.tol, self.param) = (false, p, tol, param);
        (self.vtx, self.arc) = (v.clone(), a.clone());
    }
    fn set_value_on_arc(&mut self, p: Point3, tol: f64, arc: &TheArc, param: f64) {
        (self.isnew, self.point, self.tol, self.param) = (true, p, tol, param);
        self.arc = arc.clone();
    }
    fn is_new(&self) -> bool {
        self.isnew
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        if self.isnew {
            return Err(Failure::with_type("Standard_DomainError", "", ""));
        }
        Ok(self.vtx.clone())
    }
    fn arc(&self) -> TheArc {
        self.arc.clone()
    }
    fn parameter(&self) -> f64 {
        self.param
    }
}

/// IntPatch_TheSegmentOfTheSOnBounds.
#[derive(Clone, Default)]
pub struct ProbeSegment {
    pub arc: TheArc,
    pub first: Option<ProbePathPoint>,
    pub last: Option<ProbePathPoint>,
}

impl TheSegment for ProbeSegment {
    type PathPoint = ProbePathPoint;
    fn set_value(&mut self, arc: &TheArc) {
        (self.first, self.last, self.arc) = (None, None, arc.clone());
    }
    fn set_limit_point(&mut self, point: &ProbePathPoint, first: bool) {
        if first {
            self.first = Some(point.clone());
        } else {
            self.last = Some(point.clone());
        }
    }
}

/// GaProbeEdgeTopolTool: Adaptor3d_TopolTool whose Edge() is a real edge.
pub struct ProbeDomain {
    pub tool: Adaptor3dTopolTool,
    pub edge: Option<Edge>,
}

impl TheTopolTool for ProbeDomain {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_u(&self.tool)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_v(&self.tool)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples(&self.tool)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        TheTopolTool::sample_point(&self.tool, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        TheTopolTool::classify(&self.tool, point, tol, reframe)
    }
    fn init(&self) {
        TheTopolTool::init(&self.tool);
    }
    fn more(&self) -> bool {
        TheTopolTool::more(&self.tool)
    }
    fn value(&self) -> Result<TheArc, Failure> {
        TheTopolTool::value(&self.tool)
    }
    fn next(&self) {
        TheTopolTool::next(&self.tool);
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.tool.initialize_arc(arc)
    }
    fn init_vertex_iterator(&self) {
        TheTopolTool::init_vertex_iterator(&self.tool);
    }
    fn more_vertex(&self) -> bool {
        TheTopolTool::more_vertex(&self.tool)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        TheTopolTool::vertex(&self.tool)
    }
    fn next_vertex(&self) {
        TheTopolTool::next_vertex(&self.tool);
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        TheTopolTool::identical(&self.tool, first, second)
    }
    fn edge(&self) -> Option<Edge> {
        self.edge.clone()
    }
}
