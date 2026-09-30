// Rust translation of IntCurveSurface_IntersectionPoint.hxx/.lxx/.cxx and
// IntCurveSurface_TransitionOnCurve.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_IntersectionPoint`: one point where a curve meets a surface.
use super::dump::general;
use rustycad_tkmath::gp::Point3;
use std::fmt;

/// `IntCurveSurface_TransitionOnCurve`: how the curve crosses the surface, by the
/// sign of the curve tangent along the surface normal (both C1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TransitionOnCurve {
    /// `IntCurveSurface_Tangent`: the curve touches the surface (the default).
    #[default]
    Tangent,
    /// `IntCurveSurface_In`: the curve enters, against the surface normal.
    In,
    /// `IntCurveSurface_Out`: the curve leaves, along the surface normal.
    Out,
}

/// `IntCurveSurface_IntersectionPoint`: the point, its (u, v) on the surface, its
/// parameter w on the curve and the transition of the curve.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::{IntersectionPoint, TransitionOnCurve};
/// use rustycad_tkmath::gp::Point3;
/// let p = IntersectionPoint::new(Point3::new(1.0, 2.0, 3.0), 0.5, 0.6, 0.7, TransitionOnCurve::In);
/// assert_eq!((p.u(), p.v(), p.w()), (0.5, 0.6, 0.7));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntersectionPoint {
    my_p: Point3,
    my_u_surf: f64,
    my_v_surf: f64,
    my_u_curv: f64,
    my_tr_on_curv: TransitionOnCurve,
}

impl IntersectionPoint {
    /// `IntCurveSurface_IntersectionPoint(P, USurf, VSurf, UCurv, TrCurv)`;
    /// `IntersectionPoint::new(p, 0.5, 0.6, 0.7, TransitionOnCurve::In)`.
    pub fn new(p: Point3, u_surf: f64, v_surf: f64, u_curv: f64, tr: TransitionOnCurve) -> Self {
        Self {
            my_p: p,
            my_u_surf: u_surf,
            my_v_surf: v_surf,
            my_u_curv: u_curv,
            my_tr_on_curv: tr,
        }
    }
    /// `SetValues(P, USurf, VSurf, UCurv, TrCurv)`: replaces every field;
    /// `point.set_values(p, 1.0, 2.0, 3.0, TransitionOnCurve::Out)`.
    pub fn set_values(
        &mut self,
        p: Point3,
        u_surf: f64,
        v_surf: f64,
        u_curv: f64,
        tr: TransitionOnCurve,
    ) {
        *self = Self::new(p, u_surf, v_surf, u_curv, tr);
    }
    /// `Values(P, USurf, VSurf, UCurv, TrCurv)`: every field;
    /// `let (p, u, v, w, tr) = point.values();`.
    pub fn values(&self) -> (Point3, f64, f64, f64, TransitionOnCurve) {
        (
            self.my_p,
            self.my_u_surf,
            self.my_v_surf,
            self.my_u_curv,
            self.my_tr_on_curv,
        )
    }
    /// `Pnt()`: the geometric point; `point.pnt()`.
    pub fn pnt(&self) -> Point3 {
        self.my_p
    }
    /// `U()`: the u parameter on the surface; `point.u()`.
    pub fn u(&self) -> f64 {
        self.my_u_surf
    }
    /// `V()`: the v parameter on the surface; `point.v()`.
    pub fn v(&self) -> f64 {
        self.my_v_surf
    }
    /// `W()`: the parameter on the curve; `point.w()`.
    pub fn w(&self) -> f64 {
        self.my_u_curv
    }
    /// `Transition()`: the transition of the curve; `point.transition()`.
    pub fn transition(&self) -> TransitionOnCurve {
        self.my_tr_on_curv
    }
    /// `Dump()`: the three native lines (written to std::cout natively);
    /// `point.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        let p = self.my_p;
        writeln!(
            out,
            "IntersectionPoint: P({},{},{})",
            general(p.x()),
            general(p.y()),
            general(p.z())
        )?;
        let (u, v, w) = (
            general(self.my_u_surf),
            general(self.my_v_surf),
            general(self.my_u_curv),
        );
        writeln!(
            out,
            "                 : USurf({u})  VSurf({v})   UCurve({w})"
        )?;
        let name: &str = match self.my_tr_on_curv {
            TransitionOnCurve::Tangent => " Tangent ",
            TransitionOnCurve::In => " In ",
            TransitionOnCurve::Out => " Out ",
        };
        writeln!(out, "                 : TransitionOnSurf {name}")
    }
}
