//! The common parts of `IsSolution`: the residual test, the tangents of the section ends
//! and the angle bookkeeping of the approximation.
use super::super::linalg::vget;
use super::super::normal_terms::EPS;
use super::{Radii, TwoSurfaceCore};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::gp::{Vector2, Vector3};
use std::f64::consts::PI;

/// `ns.SetLinearForm(nplan.Dot(ns) / norm, nplan, -1. / norm, ns)`: the projected normal
/// with the norm `|nplan ^ ns|` (1 when below `Eps`).
pub(crate) fn project_normal(nplan: Vector3, ns: Vector3) -> Vector3 {
    let mut norm = nplan.crossed(ns).magnitude();
    if norm < EPS {
        norm = 1.; // Unsatisfactory, but it is not necessary to stop
    }
    let mut out = Vector3::default();
    out.set_linear_form(nplan.dot(ns) / norm, nplan, -1. / norm, ns);
    out
}

/// The angle from `ns1` to `ns2` around `np` in `]-pi/2, 3pi/2]`, `Sina` negated for an
/// odd choice (the plane normal is reversed then).
pub(crate) fn section_angle(ns1: Vector3, ns2: Vector3, np: Vector3, choix: i32) -> f64 {
    let mut cosa = ns1.dot(ns2);
    let mut sina = np.dot(ns1.crossed(ns2));
    if choix % 2 != 0 {
        sina = -sina; // nplan is changed in -nplan
    }
    if cosa > 1. {
        cosa = 1.;
        sina = 0.;
    }
    let mut angle = cosa.acos();
    // Reframing on ]-pi/2, 3pi/2]
    if sina < 0. {
        angle = if cosa > 0. { -angle } else { 2. * PI - angle };
    }
    angle
}

impl TwoSurfaceCore {
    /// Whether `|E(1)| <= tol` and `|E(2..4)|^2 <= tol^2`.
    pub(crate) fn residual_within(&self, tol: f64) -> Result<bool, Failure> {
        let e = &self.eq.e;
        let (e1, e2, e3, e4) = (vget(e, 1)?, vget(e, 2)?, vget(e, 3)?, vget(e, 4)?);
        Ok(e1.abs() <= tol && e2 * e2 + e3 * e3 + e4 * e4 <= tol * tol)
    }

    /// The tangents of a solution `s` (dX/dt), or a tangency point for `None`.
    pub(crate) fn apply_tangents(&mut self, solution: Option<[f64; 4]>) {
        self.istangent = solution.is_none();
        let Some(s) = solution else {
            return;
        };
        let (j1, j2) = (self.s1.jet, self.s2.jet);
        self.tg1.set_linear_form(s[0], j1.d1u, s[1], j1.d1v);
        self.tg2.set_linear_form(s[2], j2.d1u, s[3], j2.d1v);
        self.tg12d = Vector2::new(s[0], s[1]);
        self.tg22d = Vector2::new(s[2], s[3]);
    }

    /// The angle of the current section (ns1, ns2 projected and oriented by the radii).
    pub(crate) fn current_angle(&self, radii: Radii) -> f64 {
        let np = self.plane.nplan;
        let mut ns1 = project_normal(np, self.s1.nsurf);
        let mut ns2 = project_normal(np, self.s2.nsurf);
        if radii.positive(false) {
            ns1.reverse();
        }
        if radii.positive(true) {
            ns2.reverse();
        }
        section_angle(ns1, ns2, np, self.choix)
    }

    /// The update of `maxang` and `minang`.
    pub(crate) fn record_angle(&mut self, angle: f64) {
        if angle.abs() > self.maxang {
            self.maxang = angle.abs();
        }
        if angle.abs() < self.minang {
            self.minang = angle.abs();
        }
    }

    /// The update of `distmin` with the current points.
    pub(crate) fn record_distance(&mut self) {
        self.distmin = std_min(self.distmin, self.s1.pts.distance(&self.s2.pts));
    }
}
