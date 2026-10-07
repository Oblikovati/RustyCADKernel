//! `IsSolution` (the tangents along the guide and the angle range) and `Decroch` of the
//! surface-restriction fillets.
use super::{SurfRstCore, t3dto2d};
use crate::blend_func::linalg::{matrix, point_from, vector, vget, vset};
use crate::blend_func::package::live_surface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point2, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Svd, Vector};
use std::f64::consts::PI;

/// The terms of the tangent system at a solution.
struct SolutionFrame {
    d1u1: Vector3,
    d1v1: Vector3,
    d1: Vector3,
    ns: Vector3,
    resul: Vector3,
}

impl SurfRstCore {
    /// `IsSolution(Sol, Tol)`: `Values` (of the class, through `values`), the residual
    /// test, then the tangents and the angle range; a refused point is a tangency point.
    pub(crate) fn is_solution(
        &mut self,
        sol: &Vector<'_>,
        tol: f64,
        values: impl FnOnce(
            &mut Self,
            &Vector<'_>,
            &mut Vector<'_>,
            &mut Matrix<'_>,
        ) -> Result<bool, Failure>,
    ) -> Result<bool, Failure> {
        let (mut valsol, mut gradsol) = (vector(3)?, matrix(3, 3)?);
        values(self, sol, &mut valsol, &mut gradsol)?;
        let within = vget(&valsol, 1)?.abs() <= tol
            && vget(&valsol, 2)?.abs() <= tol
            && vget(&valsol, 3)?.abs() <= 2. * tol * self.ray.abs();
        if !within {
            self.istangent = true;
            return Ok(false);
        }
        let (frame, secmember) = self.tangent_system(sol)?;
        self.solve_tangents(&frame, &gradsol, secmember)?;
        self.update_angles(frame.ns, frame.resul)?;
        self.distmin = std_min(self.distmin, self.pts.distance(&self.ptrst));
        Ok(true)
    }

    /// The points at `sol` and the right-hand side of the tangent system.
    fn tangent_system(
        &mut self,
        sol: &Vector<'_>,
    ) -> Result<(SolutionFrame, Vector<'static>), Failure> {
        self.pt2ds = Point2::new(vget(sol, 1)?, vget(sol, 2)?);
        self.prmrst = vget(sol, 3)?;
        self.pt2drst = self.live_rst()?.value(self.prmrst)?;
        let s = live_surface(&self.surf)?.d1(vget(sol, 1)?, vget(sol, 2)?)?;
        self.pts = s.point;
        let (ptrst, d1) = self.cons.d1(vget(sol, 3)?)?;
        self.ptrst = ptrst;
        let (nplan, normtg) = (self.nplan, self.normtg);
        let mut dnplan = Vector3::default();
        dnplan.set_linear_form(
            1. / normtg,
            self.d2gui,
            -1. / normtg * (nplan.dot(self.d2gui)),
            nplan,
        );
        let mut secmember = vector(3)?;
        vset(
            &mut secmember,
            1,
            normtg - dnplan.dot(Vector3::from(self.pts.xyz() - self.ptgui.xyz())),
        )?;
        vset(
            &mut secmember,
            2,
            normtg - dnplan.dot(Vector3::from(self.ptrst.xyz() - self.ptgui.xyz())),
        )?;
        let raw = s.d1u.crossed(s.d1v);
        let (ns, resul, third) = self.guide_term(raw, dnplan);
        vset(&mut secmember, 3, third)?;
        Ok((
            SolutionFrame {
                d1u1: s.d1u,
                d1v1: s.d1v,
                d1,
                ns,
                resul,
            },
            secmember,
        ))
    }

    /// The projected normal, `resul` (restriction point to center) and `secmember(3)`:
    /// `-2 temp.resul` with a constant radius, with a law
    /// `-2 ray dnw.resul - 2 dray ns.resul + 2 ray dray`.
    fn guide_term(&self, ns: Vector3, dnplan: Vector3) -> (Vector3, Vector3, f64) {
        let (nplan, ray) = (self.nplan, self.ray);
        let ncrossns = nplan.crossed(ns);
        let ndotns = nplan.dot(ns);
        let norm = ncrossns.magnitude();
        let grosterme = ncrossns.dot(dnplan.crossed(ns)) / norm / norm;
        let mut derivative = Vector3::default();
        let weights = match self.law {
            None => [
                ray / norm * (dnplan.dot(ns) - grosterme * ndotns),
                ray * ndotns / norm,
                ray * grosterme / norm,
            ],
            Some(_) => [
                (dnplan.dot(ns) - grosterme * ndotns) / norm,
                ndotns / norm,
                grosterme / norm,
            ],
        };
        derivative.set_linear_form_three(weights, [nplan, dnplan, ns]);
        let mut projected = Vector3::default();
        projected.set_linear_form(ndotns / norm, nplan, -1. / norm, ns);
        let mut resul = Vector3::default();
        resul.set_scaled_sum(ray, projected, Vector3::between(self.ptrst, self.pts));
        let third = match &self.law {
            None => -2. * (derivative.dot(resul)),
            Some(law) => {
                -2. * ray * (derivative.dot(resul)) - 2. * law.dray * (projected.dot(resul))
                    + 2. * ray * law.dray
            }
        };
        (projected, resul, third)
    }

    /// Gauss (pivot 1e-20), then SVD (cutoff 1e-6); off a tangency point the 3d and 2d
    /// tangents on the surface and on the restriction.
    fn solve_tangents(
        &mut self,
        frame: &SolutionFrame,
        gradsol: &Matrix<'_>,
        mut secmember: Vector<'static>,
    ) -> Result<(), Failure> {
        let resol = Gauss::new(gradsol)?;
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
        if self.istangent {
            return Ok(());
        }
        let (s1, s2, s3) = (
            vget(&secmember, 1)?,
            vget(&secmember, 2)?,
            vget(&secmember, 3)?,
        );
        self.tgs.set_linear_form(s1, frame.d1u1, s2, frame.d1v1);
        self.tgrst = s3 * frame.d1;
        self.tg2ds = Vector2::new(s1, s2);
        let rst = live_surface(&self.surfrst)?.d1(self.pt2drst.x(), self.pt2drst.y())?;
        let (a, b) = t3dto2d(self.tgrst, rst.d1u, rst.d1v);
        self.tg2drst = Vector2::new(a, b);
        Ok(())
    }

    /// The update of `maxang` and `minang` with the angle of the section.
    fn update_angles(&mut self, mut ns: Vector3, resul: Vector3) -> Result<(), Failure> {
        if self.ray > 0. {
            ns.reverse();
        }
        let ns2 = -resul.normalized()?;
        let cosa = ns.dot(ns2);
        let mut sina = self.nplan.dot(ns.crossed(ns2));
        if self.choix % 2 != 0 {
            sina = -sina; // nplan is changed to -nplan
        }
        let mut angle = cosa.acos();
        if sina < 0. {
            angle = 2. * PI - angle;
        }
        if angle > self.maxang {
            self.maxang = angle;
        }
        if angle < self.minang {
            self.minang = angle;
        }
        Ok(())
    }

    /// `Decroch(Sol, NS, TgS)`: whether the section leaves the reference restriction; the
    /// surface normal and the tangent to the section at the surface point.
    pub(crate) fn decroch(&self, sol: &Vector<'_>) -> Result<(bool, Vector3, Vector3), Failure> {
        let s = live_surface(&self.surf)?.d1(vget(sol, 1)?, vget(sol, 2)?)?;
        let ns = s.d1u.crossed(s.d1v);
        let center = point_from(s.point.xyz() + *self.in_plane(ns).xyz() * self.ray);
        let mut tgs = self.nplan.crossed(Vector3::between(center, s.point));
        if self.choix % 2 == 1 {
            tgs.reverse();
        }
        let ref_rst =
            super::super::live_curve2d_of(&self.rstref, self.class)?.value(vget(sol, 3)?)?;
        let r = live_surface(&self.surfref)?.d1(ref_rst.x(), ref_rst.y())?;
        let mut nrst_in_plane = self.in_plane(r.d1u.crossed(r.d1v));
        let centptrst = Vector3::between(center, r.point);
        if centptrst.dot(nrst_in_plane) < 0. {
            nrst_in_plane.reverse();
        }
        let mut tgrst = self.nplan.crossed(centptrst);
        if self.choix % 2 == 1 {
            tgrst.reverse();
        }
        let nt = nrst_in_plane.magnitude() * tgrst.magnitude();
        if nt.abs() < 1.0e-7 {
            return Ok((false, ns, tgs)); // Singularity or Incoherence.
        }
        let dot = nrst_in_plane.dot(tgrst) / nt;
        Ok((dot < 1.0e-10, ns, tgs))
    }

    /// The normal projected in the guide plane: `(nplan.n / |nplan ^ n|) nplan - n / |..|`
    /// with the inverse norm multiplied.
    fn in_plane(&self, n: Vector3) -> Vector3 {
        let unsurnorm = 1. / self.nplan.crossed(n).magnitude();
        let mut out = Vector3::default();
        out.set_linear_form(self.nplan.dot(n) * unsurnorm, self.nplan, -unsurnorm, n);
        out
    }
}
