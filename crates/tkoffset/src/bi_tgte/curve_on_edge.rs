// Rust translation of ModelingAlgorithms/TKOffset/BiTgte/BiTgte_CurveOnEdge.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BiTgte_CurveOnEdge`: the projection on an edge of the points of an edge of a face,
//! as a 3d curve adaptor.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// Standard_NotImplemented of the members the native class does not implement.
fn not_implemented() -> Failure {
    Failure::with_message("Standard_NotImplemented", "BiTgte_CurveOnEdge")
}

/// The fields of `BiTgte_CurveOnEdge`.
#[derive(Clone)]
struct CurveOnEdgeState {
    my_edge: Edge,
    my_eon_f: Edge,
    my_curv: Handle<dyn GeomCurve>,
    my_con_f: Handle<dyn GeomCurve>,
    my_type: CurveType,
    my_circ: Circle3,
}

impl Default for CurveOnEdgeState {
    fn default() -> Self {
        Self {
            my_edge: Edge::default(),
            my_eon_f: Edge::default(),
            my_curv: Handle::default(),
            my_con_f: Handle::default(),
            my_type: CurveType::OtherCurve,
            my_circ: Circle3::default(),
        }
    }
}

/// `BiTgte_CurveOnEdge`: the curve of an edge of a face (`EonF`) projected on another
/// edge, a circle of null radius when the edge is the axis of the circular `EonF`.
///
/// ```
/// use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
/// use rustycad_tkmath::geom_abs::CurveType;
/// use rustycad_tkoffset::bi_tgte::BiTgteCurveOnEdge;
/// let curve = BiTgteCurveOnEdge::new();
/// assert_eq!(curve.get_type()?, CurveType::OtherCurve);
/// assert!(curve.continuity().is_err());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BiTgteCurveOnEdge {
    parent: StandardTransient,
    state: SharedState<CurveOnEdgeState>,
}

impl_transient!(
    BiTgteCurveOnEdge,
    "BiTgte_CurveOnEdge",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl Default for BiTgteCurveOnEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl BiTgteCurveOnEdge {
    /// `BiTgte_CurveOnEdge()`; `BiTgteCurveOnEdge::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(CurveOnEdgeState::default()),
        }
    }

    /// `BiTgte_CurveOnEdge(EonF, Edge)`; `BiTgteCurveOnEdge::with_edges(&eonf, &edge)?`.
    pub fn with_edges(eon_f: &Edge, edge: &Edge) -> Result<Self, Failure> {
        let curve = Self::new();
        curve.init(eon_f, edge)?;
        Ok(curve)
    }

    /// `Init(EonF, Edge)`: the trimmed curves of both edges; the type is a circle when
    /// the edge is a line coaxial to the circle of `EonF`; `curve.init(&eonf, &edge)?`.
    pub fn init(&self, eon_f: &Edge, edge: &Edge) -> Result<(), Failure> {
        let mut s = self.state.write();
        s.my_edge = edge.clone();
        let (c, f, l) = BRepTool::curve(&s.my_edge)?;
        s.my_curv = Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast();

        s.my_eon_f = eon_f.clone();
        let (c, f, l) = BRepTool::curve(&s.my_eon_f)?;
        s.my_con_f = Handle::new(GeomTrimmedCurve::new(&c, f, l, true, true)?).cast();

        // peut on generer un cercle de rayon nul
        let curv = GeomAdaptorCurve::with_curve(&s.my_curv)?;
        let con_f = GeomAdaptorCurve::with_curve(&s.my_con_f)?;

        s.my_type = CurveType::OtherCurve;
        if curv.get_type()? == CurveType::Line && con_f.get_type()? == CurveType::Circle {
            let a1 = curv.line()?.position();
            let a2 = con_f.circle()?.axis();
            if a1.is_coaxial(a2, precision::angular(), precision::confusion()) {
                s.my_type = CurveType::Circle;
                s.my_circ = Circle3::new(con_f.circle()?.position(), 0.)?;
            }
        }
        Ok(())
    }

    fn con_f(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let c = self.state.read().my_con_f.clone();
        if c.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "BiTgte_CurveOnEdge: no curve",
            ));
        }
        Ok(c)
    }
}

impl Adaptor3dCurve for BiTgteCurveOnEdge {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let copy = Self {
            parent: StandardTransient::default(),
            state: SharedState::new(self.state.snapshot()),
        };
        Ok(Handle::new(copy).cast())
    }

    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.con_f()?.get().map_or(0., |c| c.first_parameter()))
    }

    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.con_f()?.get().map_or(0., |c| c.last_parameter()))
    }

    fn continuity(&self) -> Result<Continuity, Failure> {
        Err(not_implemented())
    }

    fn nb_intervals(&self, _shape: Continuity) -> Result<i32, Failure> {
        Err(not_implemented())
    }

    fn intervals(&self, _bounds: &mut Array1<'_, f64>, _shape: Continuity) -> Result<(), Failure> {
        Err(not_implemented())
    }

    fn trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Err(not_implemented())
    }

    fn is_closed(&self) -> Result<bool, Failure> {
        Err(not_implemented())
    }

    fn is_periodic(&self) -> Result<bool, Failure> {
        Err(not_implemented())
    }

    fn period(&self) -> Result<f64, Failure> {
        Err(not_implemented())
    }

    /// `EvalD0(U)`: the nearest point on the edge of the point of `EonF`.
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let (con_f, curv) = {
            let s = self.state.read();
            (s.my_con_f.clone(), s.my_curv.clone())
        };
        let a_p = con_f
            .get()
            .ok_or_else(|| {
                Failure::with_message("Standard_NullObject", "BiTgte_CurveOnEdge: no curve")
            })?
            .value(u)?;
        let mut a_projector = ProjectPointOnCurve::new();
        a_projector.init(a_p, &curv)?;
        a_projector.nearest_point()
    }

    fn eval_d1(&self, _u: f64) -> Result<GeomCurveD1, Failure> {
        Err(not_implemented())
    }

    fn eval_d2(&self, _u: f64) -> Result<GeomCurveD2, Failure> {
        Err(not_implemented())
    }

    fn eval_d3(&self, _u: f64) -> Result<GeomCurveD3, Failure> {
        Err(not_implemented())
    }

    fn eval_dn(&self, _u: f64, _n: i32) -> Result<Vector3, Failure> {
        Err(not_implemented())
    }

    fn resolution(&self, _r3d: f64) -> Result<f64, Failure> {
        Err(not_implemented())
    }

    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(self.state.read().my_type)
    }

    fn line(&self) -> Result<Line3, Failure> {
        Err(not_implemented())
    }

    /// `Circle()`: the circle of null radius; Standard_NoSuchObject for another type.
    fn circle(&self) -> Result<Circle3, Failure> {
        let s = self.state.read();
        if s.my_type != CurveType::Circle {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "BiTgte_CurveOnEdge::Circle",
            ));
        }
        Ok(s.my_circ)
    }

    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        Err(not_implemented())
    }

    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        Err(not_implemented())
    }

    fn parabola(&self) -> Result<Parabola3, Failure> {
        Err(not_implemented())
    }

    fn degree(&self) -> Result<i32, Failure> {
        Err(not_implemented())
    }

    fn is_rational(&self) -> Result<bool, Failure> {
        Err(not_implemented())
    }

    fn nb_poles(&self) -> Result<i32, Failure> {
        Err(not_implemented())
    }

    fn nb_knots(&self) -> Result<i32, Failure> {
        Err(not_implemented())
    }

    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        Err(not_implemented())
    }

    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        Err(not_implemented())
    }
}
