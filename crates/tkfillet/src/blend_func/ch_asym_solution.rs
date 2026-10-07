//! `BlendFunc_ChAsym::IsSolution` and the first-order `Section`: the tangents of the
//! section ends along the guide, solved by Gauss (pivot 1e-9) and, when that fails, SVD.
use super::ChAsym;
use crate::blend::{BlendFunction, BlendPoint};
use crate::blend_func::gen_chamfer::{ChamferEquations, put_tangents_2d, put_tangents_3d, uv_of};
use crate::blend_func::linalg::{matrix, vector, vset};
use crate::blend_func::package::{live_curve, live_surface};
use crate::blend_func::two_surfaces::point_parameters;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::gp::{Point2, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Svd, Vector};

/// The guide and surface terms both members compute before `Values`.
struct GuideTerms {
    d1gui: Vector3,
    np: Vector3,
    dnp: Vector3,
    normg: f64,
    dwtsurf1: Vector3,
    d1u1: Vector3,
    d1v1: Vector3,
    d1u2: Vector3,
    d1v2: Vector3,
    pguis1: Vector3,
    pguis2: Vector3,
    s1s2: Vector3,
    psca_inv: f64,
    temp: f64,
}

impl ChAsym {
    /// The guide frame at the current parameter and both surfaces at `sol`; sets the
    /// points and `tsurf1` as natively.
    fn guide_terms(&mut self, sol: &Vector<'_>) -> Result<GuideTerms, Failure> {
        let (ptgui, d1gui, d2gui) = live_curve(&self.tcurv)?.d2(self.param)?;
        let mut normg = d1gui.magnitude();
        let mut np = d1gui.normalized()?;
        let mut dnp = (d2gui - np.dot(d2gui) * np) / normg;
        if self.core.choix % 2 != 0 {
            np.reverse();
            dnp.reverse();
            normg = -normg;
        }
        let [u1, v1, u2, v2] = uv_of(sol)?;
        let d1 = live_surface(&self.core.surf1)?.d1(u1, v1)?;
        self.pt1 = d1.point;
        let nsurf1 = d1.d1u.crossed(d1.d1v);
        self.tsurf1 = nsurf1.crossed(np);
        let dwtsurf1 = nsurf1.crossed(dnp);
        let d2 = live_surface(&self.core.surf2)?.d1(u2, v2)?;
        self.pt2 = d2.point;
        let (pguis1, pguis2) = (
            Vector3::between(ptgui, self.pt1),
            Vector3::between(ptgui, self.pt2),
        );
        let s1s2 = Vector3::between(self.pt1, self.pt2);
        let psca_inv = 1. / self.tsurf1.dot(s1s2);
        let (nordu1, nordv1) = (d1.d1u.magnitude(), d1.d1v.magnitude());
        let temp = 2. * (nordu1 + nordv1) * s1s2.magnitude() + 2. * nordu1 * nordv1;
        Ok(GuideTerms {
            d1gui,
            np,
            dnp,
            normg,
            dwtsurf1,
            d1u1: d1.d1u,
            d1v1: d1.d1v,
            d1u2: d2.d1u,
            d1v2: d2.d1v,
            pguis1,
            pguis2,
            s1s2,
            psca_inv,
            temp,
        })
    }

    /// The derivative of the equations along the guide; `tsurf1` is the one of `Values`.
    fn second_member(&self, g: &GuideTerms) -> Result<Vector<'static>, Failure> {
        let mut secmember = vector(4)?;
        vset(&mut secmember, 1, g.normg - g.dnp.dot(g.pguis1))?;
        vset(&mut secmember, 2, g.normg - g.dnp.dot(g.pguis2))?;
        vset(&mut secmember, 3, -2. * g.d1gui.dot(g.pguis1))?;
        let cross_vec = self.tsurf1.crossed(g.s1s2);
        let f4 = g.np.dot(cross_vec) * g.psca_inv;
        let mut temp = g.dnp.dot(cross_vec) + g.np.dot(g.dwtsurf1.crossed(g.s1s2));
        temp -= f4 * g.dwtsurf1.dot(g.s1s2);
        vset(&mut secmember, 4, g.psca_inv * temp)?;
        Ok(secmember)
    }

    /// Solves `gradsol * T = secmember` (Gauss, then SVD with cutoff 1e-6), sets the
    /// tangents and the tangency flag, and the minimal distance.
    fn solve_tangents(
        &mut self,
        g: &GuideTerms,
        gradsol: &Matrix<'_>,
        mut secmember: Vector<'static>,
    ) -> Result<(), Failure> {
        let resol = Gauss::with_min_pivot(gradsol, 1.0e-9)?;
        if resol.is_done() {
            resol.solve(&mut secmember)?;
            self.istangent = false;
        } else {
            let mut sing_rs = Svd::new(gradsol)?;
            self.istangent = !sing_rs.is_done();
            if sing_rs.is_done() {
                let dedt = secmember.clone();
                sing_rs.solve(&dedt, &mut secmember, 1.0e-6)?;
            }
        }
        if !self.istangent {
            let [s1, s2, s3, s4] = uv_of(&secmember)?;
            self.tg1.set_linear_form(s1, g.d1u1, s2, g.d1v1);
            self.tg2.set_linear_form(s3, g.d1u2, s4, g.d1v2);
            self.tg12d = Vector2::new(s1, s2);
            self.tg22d = Vector2::new(s3, s4);
        }
        self.core.distmin = std_min(self.core.distmin, self.pt1.distance(&self.pt2));
        Ok(())
    }

    /// `IsSolution(Sol, Tol)`: the residual test (the angle equation scaled by the
    /// surface speeds), then the tangents; a refused point is a tangency point.
    pub(super) fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let g = self.guide_terms(sol)?;
        let (mut valsol, mut gradsol) = (vector(4)?, matrix(4, 4)?);
        self.try_values(sol, &mut valsol, &mut gradsol)?;
        let [f1, f2, f3, f4] = uv_of(&valsol)?;
        if f1.abs() < tol
            && f2.abs() < tol
            && f3.abs() < 2. * self.dist1 * tol
            && f4.abs() < tol * (1. + self.tgang) * g.psca_inv.abs() * g.temp
        {
            let secmember = self.second_member(&g)?;
            self.solve_tangents(&g, &gradsol, secmember)?;
            return Ok(true);
        }
        self.istangent = true;
        Ok(false)
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the points (of
    /// the last evaluation, as natively) and the tangents at `P` without a residual test.
    pub(super) fn section_with_tangents(
        &mut self,
        point: &BlendPoint,
        values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let sol = point_parameters(point)?;
        self.param = point.parameter();
        let [u1, v1, u2, v2] = uv_of(&sol)?;
        let (low2d, upp2d) = (values.poles2d.lower(), values.poles2d.upper());
        put(values.poles2d, low2d, Point2::new(u1, v1))?;
        put(values.poles2d, upp2d, Point2::new(u2, v2))?;
        let (low, upp) = (values.poles.lower(), values.poles.upper());
        put(values.poles, low, self.point_on_s1())?;
        put(values.poles, upp, self.point_on_s2())?;
        put(values.weights, low, 1.0)?;
        put(values.weights, upp, 1.0)?;
        let g = self.guide_terms(&sol)?;
        let (mut valsol, mut gradsol) = (vector(4)?, matrix(4, 4)?);
        self.try_values(&sol, &mut valsol, &mut gradsol)?;
        let secmember = self.second_member(&g)?;
        self.solve_tangents(&g, &gradsol, secmember)?;
        if !self.istangent {
            put_tangents_2d(self, values.d_poles2d, low2d, upp2d)?;
            put_tangents_3d(self, values.d_poles, values.d_weights, (low, upp))?;
        }
        Ok(!self.istangent)
    }
}
