//! `IsSolution` (the tangents along the guide and the angle range) and `Decroch` of the
//! restriction-restriction fillets.
use super::RstRstCore;
use crate::blend::DecrochStatus;
use crate::blend_func::linalg::{copy_vector, matrix, vector, vget, vset};
use crate::blend_func::package::live_surface;
use crate::brep_blend::live_curve2d_of;
use crate::brep_blend::surf_rst::t3dto2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Svd, Vector};
use std::f64::consts::PI;

impl RstRstCore {
    /// The guide-plane derivative and the right-hand side of the tangent system at the
    /// current restriction points.
    pub(super) fn guide_second_member(&self, dnplan: Vector3) -> Result<Vector<'static>, Failure> {
        let mut secmember = vector(2)?;
        vset(
            &mut secmember,
            1,
            self.normtg - dnplan.dot(Vector3::from(self.ptrst1.xyz() - self.ptgui.xyz())),
        )?;
        vset(
            &mut secmember,
            2,
            self.normtg - dnplan.dot(Vector3::from(self.ptrst2.xyz() - self.ptgui.xyz())),
        )?;
        Ok(secmember)
    }

    /// `dnplan` of the members after `Set(Param)`.
    pub(super) fn dnplan(&self) -> Vector3 {
        let mut dnplan = Vector3::default();
        dnplan.set_linear_form(
            1. / self.normtg,
            self.d2gui,
            -1. / self.normtg * (self.nplan.dot(self.d2gui)),
            self.nplan,
        );
        dnplan
    }

    /// Gauss (pivot `pivot`), then SVD; `None` on a tangency point. The SVD branch of
    /// `IsSolution` copies the 2-vector into a 3-vector, a Standard_DimensionError, kept.
    pub(super) fn solve_guide(
        &self,
        gradsol: &Matrix<'_>,
        mut secmember: Vector<'static>,
        (pivot, dedt_len): (f64, i32),
    ) -> Result<Option<[f64; 2]>, Failure> {
        let resol = Gauss::with_min_pivot(gradsol, pivot)?;
        if resol.is_done() {
            resol.solve(&mut secmember)?;
            return Ok(Some([vget(&secmember, 1)?, vget(&secmember, 2)?]));
        }
        let mut sing_rs = Svd::new(gradsol)?;
        if !sing_rs.is_done() {
            return Ok(None);
        }
        let mut dedt = vector(dedt_len)?;
        copy_vector(&mut dedt, &secmember)?;
        sing_rs.solve(&dedt, &mut secmember, 1.0e-6)?;
        Ok(Some([vget(&secmember, 1)?, vget(&secmember, 2)?]))
    }

    /// `IsSolution(Sol, Tol)`: `Values`, the residual test, then the tangents and the
    /// angle range; false without a center (the tangency flag kept).
    pub(crate) fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let (mut valsol, mut gradsol) = (vector(2)?, matrix(2, 2)?);
        self.values(sol, &mut valsol, &mut gradsol)?;
        if vget(&valsol, 1)?.abs() > tol || vget(&valsol, 2)?.abs() > tol {
            self.istangent = true;
            return Ok(false);
        }
        let (d11, d21) = self.solution_points(sol)?;
        let secmember = self.guide_second_member(self.dnplan())?;
        let solved = self.solve_guide(&gradsol, secmember, (1.0e-20, 3))?;
        self.istangent = solved.is_none();
        if let Some(s) = solved {
            self.solution_tangents(s, d11, d21)?;
        }
        let (is_center, center, _) =
            self.center_circle_rst1_rst2(self.ptrst1, self.ptrst2, self.nplan)?;
        if !is_center {
            return Ok(false);
        }
        self.update_angles(center)?;
        self.distmin = std_min(self.distmin, self.ptrst1.distance(&self.ptrst2));
        Ok(true)
    }

    /// The restriction parameters, points and derivatives at `sol`.
    fn solution_points(&mut self, sol: &Vector<'_>) -> Result<(Vector3, Vector3), Failure> {
        self.prmrst1 = vget(sol, 1)?;
        self.pt2drst1 = live_curve2d_of(&self.r1.rst, self.class)?.value(self.prmrst1)?;
        self.prmrst2 = vget(sol, 2)?;
        self.pt2drst2 = live_curve2d_of(&self.r2.rst, self.class)?.value(self.prmrst2)?;
        let (p1, d11) = self.r1.cons.d1(self.prmrst1)?;
        let (p2, d21) = self.r2.cons.d1(self.prmrst2)?;
        (self.ptrst1, self.ptrst2) = (p1, p2);
        Ok((d11, d21))
    }

    /// The 3d and 2d tangents on both restrictions; the second 2d tangent projects the
    /// first 3d tangent, as natively.
    fn solution_tangents(
        &mut self,
        s: [f64; 2],
        d11: Vector3,
        d21: Vector3,
    ) -> Result<(), Failure> {
        self.tgrst1 = s[0] * d11;
        self.tgrst2 = s[1] * d21;
        let a = live_surface(&self.r1.surf)?.d1(self.pt2drst1.x(), self.pt2drst1.y())?;
        let (u, v) = t3dto2d(self.tgrst1, a.d1u, a.d1v);
        self.tg2drst1 = Vector2::new(u, v);
        let b = live_surface(&self.r2.surf)?.d1(self.pt2drst2.x(), self.pt2drst2.y())?;
        let (u, v) = t3dto2d(self.tgrst1, b.d1u, b.d1v);
        self.tg2drst2 = Vector2::new(u, v);
        Ok(())
    }

    fn update_angles(&mut self, center: Point3) -> Result<(), Failure> {
        let n1 = Vector3::between(center, self.ptrst1).normalized()?;
        let n2 = Vector3::between(center, self.ptrst2).normalized()?;
        let cosa = n1.dot(n2);
        let mut sina = self.nplan.dot(n1.crossed(n2));
        if self.choix % 2 != 0 {
            sina = -sina; // nplan is changed into -nplan
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

    /// `Decroch(Sol, NRst1, TgRst1, NRst2, TgRst2)`: which restrictions the section leaves
    /// at `sol`, with the normals of the reference surfaces and the section tangents.
    pub(crate) fn decroch(
        &self,
        sol: &Vector<'_>,
    ) -> Result<(DecrochStatus, [Vector3; 4]), Failure> {
        let (pt1, nrst1) = self.reference_point(0, vget(sol, 1)?)?;
        let (pt2, nrst2) = self.reference_point(1, vget(sol, 2)?)?;
        let (_, center, _) = self.center_circle_rst1_rst2(pt1, pt2, self.nplan)?;
        let (in_plane1, mut tg1) = self.reference_tangent(nrst1, pt1, center);
        let (in_plane2, mut tg2) = self.reference_tangent(nrst2, pt2, center);
        if self.choix % 2 != 0 {
            tg1.reverse();
            tg2.reverse();
        }
        let leaves1 = in_plane1.dot(tg1) > -1.0e-10;
        let leaves2 = in_plane2.dot(tg2) < 1.0e-10;
        let status = match (leaves1, leaves2) {
            (true, true) => DecrochStatus::DecrochBoth,
            (true, false) => DecrochStatus::DecrochRst1,
            (false, true) => DecrochStatus::DecrochRst2,
            (false, false) => DecrochStatus::NoDecroch,
        };
        Ok((status, [nrst1, tg1, nrst2, tg2]))
    }

    /// The point of a reference restriction at `w` and the normal of its surface there.
    fn reference_point(&self, index: usize, w: f64) -> Result<(Point3, Vector3), Failure> {
        let (surface, rst) = &self.refs[index];
        let uv = live_curve2d_of(rst, self.class)?.value(w)?;
        let d1 = live_surface(surface)?.d1(uv.x(), uv.y())?;
        Ok((d1.point, d1.d1u.crossed(d1.d1v)))
    }

    /// The reference normal in the guide plane, oriented away from the center, and the
    /// section tangent `nplan ^ (P - Center)`.
    fn reference_tangent(
        &self,
        nrst: Vector3,
        point: Point3,
        center: Point3,
    ) -> (Vector3, Vector3) {
        let unsurnorm = 1. / self.nplan.crossed(nrst).magnitude();
        let mut in_plane = Vector3::default();
        in_plane.set_linear_form(
            self.nplan.dot(nrst) * unsurnorm,
            self.nplan,
            -unsurnorm,
            nrst,
        );
        let centptrst = Vector3::from(point.xyz() - center.xyz());
        if centptrst.dot(in_plane) < 0. {
            in_plane.reverse();
        }
        (in_plane, self.nplan.crossed(centptrst))
    }
}
