// Rust translation of BRepBlend_BlendTool.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_BlendTool`: the queries of the walking on the restrictions of a face: the
//! projection of a point on an arc, the intersection of a segment with an arc, and the
//! vertices on the arcs.
use super::{BRepBlendHCurve2dTool, live_curve2d_of};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::extrema::EPCOfExtPC2d;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// `BRepBlend_BlendTool`: static members; the surface arguments are unused natively.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkfillet::brep_blend::BRepBlendBlendTool;
/// assert_eq!(BRepBlendBlendTool::nb_samples_u(&Handle::default(), 0.0, 1.0), 10);
/// ```
pub struct BRepBlendBlendTool;

type Arc = Handle<dyn Adaptor2dCurve2d>;
type Surface = Handle<dyn Adaptor3dSurface>;
type Vertex = Handle<dyn Adaptor3dHVertex>;

fn vertex<'a>(v: &'a Vertex, member: &str) -> Result<&'a dyn Adaptor3dHVertex, Failure> {
    v.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("BRepBlend_BlendTool::{member}: null vertex; expected an Adaptor3d_HVertex"),
        )
    })
}

impl BRepBlendBlendTool {
    /// `Project(P, S, C, Paramproj, Dist)`: the nearest of the arc ends and of the extrema
    /// (20 samples) to `p`; always a distance ("pmn 8/10/98: a distance is always returned",
    /// BUC60360). `let (param, dist) = BRepBlendBlendTool::project(p, &s, &arc)?;`.
    pub fn project(p: Point2, _s: &Surface, c: &Arc) -> Result<(f64, f64), Failure> {
        let mut paramproj = BRepBlendHCurve2dTool::first_parameter(c)?;
        let mut dist = BRepBlendHCurve2dTool::d0(c, paramproj)?.distance(p);
        let t = BRepBlendHCurve2dTool::last_parameter(c)?;
        let p2d = BRepBlendHCurve2dTool::d0(c, t)?;
        if p2d.distance(p) < dist {
            paramproj = t;
            dist = p2d.distance(p);
        }
        let (eps_x, nbu, tol) = (1.0e-8, 20, 1.0e-5);
        let arc = live_curve2d_of(c, "BRepBlend_BlendTool::Project")?;
        let extrema = EPCOfExtPC2d::with_curve(p, arc, nbu, eps_x, tol)?;
        if !extrema.is_done() {
            return Ok((paramproj, dist));
        }
        let mut dist2 = dist * dist;
        for i in 1..=extrema.nb_ext()? {
            if extrema.square_distance(i)? < dist2 {
                dist2 = extrema.square_distance(i)?;
                paramproj = extrema.point(i)?.parameter();
            }
        }
        Ok((paramproj, dist2.sqrt()))
    }

    /// `Inters(P1, P2, S, C, Param, Dist)`: the first intersection of the segment
    /// `[P1, P2]` (extended by 1% on both sides) with the arc: its parameter on the arc and
    /// its distance to `P1`; `None` for a segment shorter than 1e-8 or no intersection.
    /// `BRepBlendBlendTool::inters(p1, p2, &s, &arc)?`.
    pub fn inters(
        p1: Point2,
        p2: Point2,
        _s: &Surface,
        c: &Arc,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let tol = 1.0e-8;
        let v = Vector2::between(p1, p2);
        let mag = v.magnitude();
        if mag < tol {
            return Ok(None);
        }
        let d = Direction2::from_vector(v)?;
        let bid: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p1, d)).cast();
        let seg = Geom2dAdaptorCurve::with_range(&bid, -0.01 * mag, 1.01 * mag)?;
        let arc = live_curve2d_of(c, "BRepBlend_BlendTool::Inters")?;
        let inter = GInter::between(&seg, arc, tol, tol)?;
        if !inter.is_done() || inter.nb_points()? == 0 {
            return Ok(None);
        }
        let ip = inter.point(1)?;
        Ok(Some((ip.param_on_second(), p1.distance(ip.value()))))
    }

    /// `SingularOnUMin(S)`: false.
    pub fn singular_on_u_min(_s: &Surface) -> bool {
        false
    }

    /// `SingularOnUMax(S)`: false.
    pub fn singular_on_u_max(_s: &Surface) -> bool {
        false
    }

    /// `SingularOnVMin(S)`: false.
    pub fn singular_on_v_min(_s: &Surface) -> bool {
        false
    }

    /// `SingularOnVMax(S)`: false.
    pub fn singular_on_v_max(_s: &Surface) -> bool {
        false
    }

    /// `NbSamplesU(S, U1, U2)`: 10.
    pub fn nb_samples_u(_s: &Surface, _u1: f64, _u2: f64) -> i32 {
        10
    }

    /// `NbSamplesV(S, V1, V2)`: 10.
    pub fn nb_samples_v(_s: &Surface, _v1: f64, _v2: f64) -> i32 {
        10
    }

    /// `Tolerance(V, A)`: the resolution of the vertex on the arc.
    /// `BRepBlendBlendTool::tolerance(&vertex, &arc)?`.
    pub fn tolerance(v: &Vertex, a: &Arc) -> Result<f64, Failure> {
        vertex(v, "Tolerance")?.resolution(a)
    }

    /// `Parameter(V, C)`: the parameter of the vertex on the arc.
    /// `BRepBlendBlendTool::parameter(&vertex, &arc)?`.
    pub fn parameter(v: &Vertex, c: &Arc) -> Result<f64, Failure> {
        vertex(v, "Parameter")?.parameter(c)
    }

    /// `Bounds(A, Ufirst, Ulast)`: the parametric range of the arc.
    /// `let (first, last) = BRepBlendBlendTool::bounds(&arc)?;`.
    pub fn bounds(a: &Arc) -> Result<(f64, f64), Failure> {
        Ok((
            BRepBlendHCurve2dTool::first_parameter(a)?,
            BRepBlendHCurve2dTool::last_parameter(a)?,
        ))
    }

    /// `CurveOnSurf(C, S)`: the arc itself.
    pub fn curve_on_surf(c: &Arc, _s: &Surface) -> Arc {
        c.clone()
    }
}
