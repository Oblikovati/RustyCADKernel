// Rust translation of ChFiDS_ElSpine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! An elementary spine: the C2 guideline curve of one run of fillet computation.
use super::errors::standard_failure;
use super::surf_data::SurfDataHandle;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Axis1, Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3,
};

/// The mutable fields of an elementary spine, behind the adaptor's lock.
#[derive(Clone)]
struct ElSpineFields {
    first_point: Point3,
    last_point: Point3,
    first_tangent: Vector3,
    last_tangent: Vector3,
    vertices_with_tangents: Vec<Axis1>,
    previous: Option<SurfDataHandle>,
    next: Option<SurfDataHandle>,
    first: f64,
    last: f64,
    period: f64,
    periodic: bool,
    saved_first: f64,
    saved_last: f64,
}

impl Default for ElSpineFields {
    fn default() -> Self {
        Self {
            first_point: Point3::default(),
            last_point: Point3::default(),
            first_tangent: Vector3::default(),
            last_tangent: Vector3::default(),
            vertices_with_tangents: Vec::new(),
            previous: None,
            next: None,
            first: 0.0,
            last: 0.0,
            period: 0.0,
            periodic: false,
            saved_first: precision::infinite(),
            saved_last: precision::infinite(),
        }
    }
}

/// An elementary spine (`ChFiDS_ElSpine`): a curve adaptor whose geometry is a loaded
/// `GeomAdaptor_Curve`, whose bounds and periodicity are its own, with the extremity points
/// and tangents, the vertices with tangents met on the way, and the fillet surfaces
/// computed before and after it.
///
/// As every adaptor, its members take `&self` and the state sits behind a lock. The
/// `Adaptor3d_Curve` members it does not override (IsClosed, Degree, NbPoles, NbKnots,
/// IsRational, OffsetCurve, EvalDN) raise Standard_NotImplemented, as natively.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::ElSpine;
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// let spine = ElSpine::new();
/// spine.set_first_parameter(1.0);
/// spine.set_last_parameter(3.0);
/// spine.set_periodic(true);
/// assert_eq!(spine.period()?, 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct ElSpine {
    parent: StandardTransient,
    curve: GeomAdaptorCurve,
    fields: SharedState<ElSpineFields>,
}
impl_transient!(
    ElSpine,
    "ChFiDS_ElSpine",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl ElSpine {
    /// An empty spine: no curve, range `[0, 0]`, not periodic, saved bounds infinite;
    /// `ElSpine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The ShallowCopy body: a spine sharing the curve, with copies of every field.
    pub fn shallow_spine(&self) -> Result<Self, Failure> {
        Ok(Self {
            parent: StandardTransient::default(),
            curve: self.curve.shallow_adaptor()?,
            fields: SharedState::new(self.fields.snapshot()),
        })
    }

    /// `GetSavedFirstParameter()`; `spine.saved_first_parameter()`.
    pub fn saved_first_parameter(&self) -> f64 {
        self.fields.read().saved_first
    }

    /// `GetSavedLastParameter()`; `spine.saved_last_parameter()`.
    pub fn saved_last_parameter(&self) -> f64 {
        self.fields.read().saved_last
    }

    /// `SetPeriodic(I)`: the periodicity, with the period `last - first` of the current
    /// bounds; `spine.set_periodic(true)`.
    pub fn set_periodic(&self, periodic: bool) {
        let mut fields = self.fields.write();
        fields.periodic = periodic;
        fields.period = fields.last - fields.first;
    }

    /// `FirstParameter(P)`; `spine.set_first_parameter(0.0)`.
    pub fn set_first_parameter(&self, parameter: f64) {
        self.fields.write().first = parameter;
    }

    /// `LastParameter(P)`; `spine.set_last_parameter(1.0)`.
    pub fn set_last_parameter(&self, parameter: f64) {
        self.fields.write().last = parameter;
    }

    /// `SaveFirstParameter()`: keeps the current first bound; `spine.save_first_parameter()`.
    pub fn save_first_parameter(&self) {
        let mut fields = self.fields.write();
        fields.saved_first = fields.first;
    }

    /// `SaveLastParameter()`: keeps the current last bound; `spine.save_last_parameter()`.
    pub fn save_last_parameter(&self) {
        let mut fields = self.fields.write();
        fields.saved_last = fields.last;
    }

    /// `SetOrigin(O)`: Standard_Failure "Elspine non periodique" when not periodic; on a
    /// B-spline curve, moves its origin to `origin` (`SetOrigin(O, PConfusion)`) and reloads
    /// it; another curve kind is left unchanged; `spine.set_origin(0.5)?`.
    pub fn set_origin(&self, origin: f64) -> Result<(), Failure> {
        if !self.fields.read().periodic {
            return Err(standard_failure("Elspine non periodique"));
        }
        let bspline: Handle<GeomBSplineCurve> = self.curve.curve().cast();
        if bspline.is_null() {
            return Ok(());
        }
        let curve: &GeomBSplineCurve = bspline.get().expect("checked non-null handle");
        curve.set_origin_parameter(origin, precision::p_confusion())?;
        self.curve.load(&bspline.cast())
    }

    /// `FirstPointAndTgt(P, T)`; `let (point, tangent) = spine.first_point_and_tgt();`.
    pub fn first_point_and_tgt(&self) -> (Point3, Vector3) {
        let fields = self.fields.read();
        (fields.first_point, fields.first_tangent)
    }

    /// `LastPointAndTgt(P, T)`; `let (point, tangent) = spine.last_point_and_tgt();`.
    pub fn last_point_and_tgt(&self) -> (Point3, Vector3) {
        let fields = self.fields.read();
        (fields.last_point, fields.last_tangent)
    }

    /// `NbVertices()`: the number of vertices with tangents; `spine.nb_vertices()`.
    pub fn nb_vertices(&self) -> i32 {
        self.fields.read().vertices_with_tangents.len() as i32
    }

    /// `VertexWithTangent(Index)`: the 1-based `index`-th vertex with its tangent;
    /// Standard_OutOfRange outside `[1, NbVertices]`; `spine.vertex_with_tangent(1)?`.
    pub fn vertex_with_tangent(&self, index: i32) -> Result<Axis1, Failure> {
        let fields = self.fields.read();
        let count = fields.vertices_with_tangents.len() as i32;
        if index < 1 || index > count {
            return Err(super::errors::out_of_range(
                "ChFiDS_ElSpine::VertexWithTangent",
                index,
                count,
            ));
        }
        Ok(fields.vertices_with_tangents[(index - 1) as usize])
    }

    /// `SetFirstPointAndTgt(P, T)`; `spine.set_first_point_and_tgt(p, t)`.
    pub fn set_first_point_and_tgt(&self, point: Point3, tangent: Vector3) {
        let mut fields = self.fields.write();
        fields.first_point = point;
        fields.first_tangent = tangent;
    }

    /// `SetLastPointAndTgt(P, T)`; `spine.set_last_point_and_tgt(p, t)`.
    pub fn set_last_point_and_tgt(&self, point: Point3, tangent: Vector3) {
        let mut fields = self.fields.write();
        fields.last_point = point;
        fields.last_tangent = tangent;
    }

    /// `AddVertexWithTangent(anAx1)`; `spine.add_vertex_with_tangent(axis)`.
    pub fn add_vertex_with_tangent(&self, axis: Axis1) {
        self.fields.write().vertices_with_tangents.push(axis);
    }

    /// `SetCurve(C)`: loads `curve` over its own bounds into the adaptor (the spine's own
    /// bounds are unchanged); Standard_NullObject for a null curve; `spine.set_curve(&c)?`.
    pub fn set_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.curve.load(curve)
    }

    /// `Previous()`: the fillet surface before the spine, `None` for a null handle;
    /// `spine.previous()`.
    pub fn previous(&self) -> Option<SurfDataHandle> {
        self.fields.read().previous.clone()
    }

    /// `ChangePrevious()`: replaces the fillet surface before the spine;
    /// `spine.change_previous(Some(data))`.
    pub fn change_previous(&self, previous: Option<SurfDataHandle>) {
        self.fields.write().previous = previous;
    }

    /// `Next()`: the fillet surface after the spine, `None` for a null handle; `spine.next()`.
    pub fn next(&self) -> Option<SurfDataHandle> {
        self.fields.read().next.clone()
    }

    /// `ChangeNext()`: replaces the fillet surface after the spine;
    /// `spine.change_next(Some(data))`.
    pub fn change_next(&self, next: Option<SurfDataHandle>) {
        self.fields.write().next = next;
    }
}

impl Adaptor3dCurve for ElSpine {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Ok(Handle::new(self.shallow_spine()?).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.fields.read().first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.fields.read().last)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        self.curve.continuity()
    }
    fn nb_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        self.curve.nb_intervals(continuity)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        self.curve.intervals(bounds, continuity)
    }
    fn trim(&self, first: f64, last: f64, tol: f64) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        self.curve.trim(first, last, tol)
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        Ok(self.fields.read().periodic)
    }
    fn period(&self) -> Result<f64, Failure> {
        let fields = self.fields.read();
        if !fields.periodic {
            return Err(standard_failure("ElSpine non periodique"));
        }
        Ok(fields.period)
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.curve.resolution(r3d)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        self.curve.get_type()
    }
    fn line(&self) -> Result<Line3, Failure> {
        self.curve.line()
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        self.curve.circle()
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        self.curve.ellipse()
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        self.curve.hyperbola()
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        self.curve.parabola()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        self.curve.bezier()
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        self.curve.bspline()
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.curve.eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        self.curve.eval_d1(u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        self.curve.eval_d2(u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        self.curve.eval_d3(u)
    }
}

#[cfg(test)]
#[path = "el_spine_tests.rs"]
mod tests;
