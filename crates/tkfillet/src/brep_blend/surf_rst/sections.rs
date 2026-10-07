//! The sections of the surface-restriction fillets: the circle at a parameter and the
//! poles at a point of a blend line, without and with their derivatives.
use super::SurfRstCore;
use crate::blend::{BlendPoint, SectionPoles};
use crate::blend_func::SectionShape;
use crate::blend_func::linalg::{matrix, point_from, vector, vget, vset};
use crate::blend_func::package::{live_curve, live_surface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill::{CircleD0, get_circle};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point2, Point3, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Svd, Vector};
use std::f64::consts::PI;

/// `#define Eps 1.e-15` of the first-order section.
pub(super) const EPS: f64 = 1.0e-15;

fn frame(origin: Point3, np: Vector3, x: Vector3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        origin,
        Direction3::from_vector(np)?,
        Direction3::from_vector(x)?,
    )
}

/// The derivatives of the projected normal of the first-order section.
struct NormalJet {
    ns: Vector3,
    ncrossns: Vector3,
    ndotns: f64,
    norm: f64,
}

impl SurfRstCore {
    /// `ns` projected in the guide plane with the native divisions:
    /// `(nplan.ns / norm) nplan - ns / norm`.
    fn projected(&self, ns: Vector3) -> Vector3 {
        let norm = self.nplan.crossed(ns).magnitude();
        let mut out = Vector3::default();
        out.set_linear_form(self.nplan.dot(ns) / norm, self.nplan, -1. / norm, ns);
        out
    }

    /// `Section(Param, U, V, W, Pdeb, Pfin, C)`: the circular section at `param`; the
    /// projection uses the plane normal of the last `Set(Param)`, as natively.
    pub(crate) fn section_circle(
        &mut self,
        param: f64,
        (u, v, w): (f64, f64, f64),
    ) -> Result<(f64, f64, Circle3), Failure> {
        (self.ptgui, self.d1gui) = live_curve(&self.tguide)?.d1(param)?;
        let mut np = self.d1gui.normalized()?;
        self.law_radius(param)?;
        let s = live_surface(&self.surf)?.d1(u, v)?;
        self.pts = s.point;
        self.ptrst = self.cons.value(w)?;
        let mut ns = self.projected(s.d1u.crossed(s.d1v));
        let center = point_from(self.pts.xyz() + *ns.xyz() * self.ray);
        if self.ray > 0. {
            ns.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
        }
        let radius = self.ray.abs();
        let mut circle = Circle3::new(frame(center, np, ns)?, radius)?;
        let mut pfin = elclib::circle_parameter(circle.position(), self.ptrst);
        // Test negative and almost null angles: special case.
        if pfin > 1.5 * PI {
            np.reverse();
            circle = Circle3::new(frame(center, np, ns)?, radius)?;
            pfin = elclib::circle_parameter(circle.position(), self.ptrst);
        }
        if pfin < precision::p_confusion() {
            pfin += precision::p_confusion();
        }
        Ok((0., pfin, circle))
    }

    /// `Section(P, Poles, Poles2d, Weigths)`: the poles of the section at `P`; a circular
    /// section reverses the member plane normal for an odd choice, as natively.
    pub(crate) fn section_d0(
        &mut self,
        point: &BlendPoint,
        values: SectionPoles<'_, '_>,
    ) -> Result<(), Failure> {
        let prm = point.parameter();
        (self.ptgui, self.d1gui) = live_curve(&self.tguide)?.d1(prm)?;
        self.law_radius(prm)?;
        self.nplan = self.d1gui.normalized()?;
        let (u1, v1) = point.parameters_on_s()?;
        let w = point.parameter_on_c()?;
        let pt2d = self.live_rst()?.value(w)?;
        let s = live_surface(&self.surf)?.d1(u1, v1)?;
        self.pts = s.point;
        self.ptrst = self.cons.value(w)?;
        self.distmin = std_min(self.distmin, self.pts.distance(&self.ptrst));
        let (low2d, upp2d) = (values.poles2d.lower(), values.poles2d.upper());
        put(values.poles2d, low2d, Point2::new(u1, v1))?;
        put(values.poles2d, upp2d, pt2d)?;
        if self.shape == SectionShape::Linear {
            return self.linear_poles(values.poles, values.weights);
        }
        let mut ns = self.projected(s.d1u.crossed(s.d1v));
        let center = point_from(self.pts.xyz() + *ns.xyz() * self.ray);
        let ns2 = Vector3::between(center, self.ptrst).normalized()?;
        if self.ray > 0. {
            ns.reverse();
        }
        if self.choix % 2 != 0 {
            self.nplan.reverse();
        }
        let circle = self.circle(ns, ns2, center);
        get_circle(self.tconv, &circle, values.poles, values.weights)
    }

    fn circle(&self, ns1: Vector3, ns2: Vector3, center: Point3) -> CircleD0 {
        CircleD0 {
            ns1,
            ns2,
            nplan: self.nplan,
            pts1: self.pts,
            pts2: self.ptrst,
            rayon: self.ray.abs(),
            center,
        }
    }

    /// The linear section: the two points with unit weights.
    fn linear_poles(
        &self,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (low, upp) = (poles.lower(), poles.upper());
        put(poles, low, self.pts)?;
        put(poles, upp, self.ptrst)?;
        put(weights, low, 1.0)?;
        put(weights, upp, 1.0)
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the poles and their
    /// derivatives along the guide; false on a tangency point (no derivative written).
    pub(crate) fn section_d1(
        &mut self,
        point: &BlendPoint,
        out: SweepD1<'_, '_>,
        values: impl FnOnce(
            &mut Self,
            &Vector<'_>,
            &mut Vector<'_>,
            &mut Matrix<'_>,
        ) -> Result<bool, Failure>,
    ) -> Result<bool, Failure> {
        let prm = point.parameter();
        let dray = self.law_d1(prm)?;
        (self.ptgui, self.d1gui, self.d2gui) = live_curve(&self.tguide)?.d2(prm)?;
        self.normtg = self.d1gui.magnitude();
        self.nplan = self.d1gui.normalized()?;
        let mut dnplan = Vector3::default();
        dnplan.set_linear_form(
            1. / self.normtg,
            self.d2gui,
            -1. / self.normtg * (self.nplan.dot(self.d2gui)),
            self.nplan,
        );
        let (u1, v1) = point.parameters_on_s()?;
        self.prmrst = point.parameter_on_c()?;
        self.pt2drst = self.live_rst()?.value(self.prmrst)?;
        let mut sol = vector(3)?;
        for (i, value) in (1..).zip([u1, v1, self.prmrst]) {
            vset(&mut sol, i, value)?;
        }
        let (mut valsol, mut gradsol) = (vector(3)?, matrix(3, 3)?);
        values(self, &sol, &mut valsol, &mut gradsol)?;
        let jet = self.section_jet(&sol, dnplan, dray)?;
        let solved = self.solve_section(&gradsol, jet.1)?;
        self.section_d1_output(point, out, (jet.0, dnplan, dray), solved)
    }
}

/// The surface jet of the first-order section and the right-hand side of its system.
pub(super) struct SectionJet {
    d1u1: Vector3,
    d1v1: Vector3,
    d2u1: Vector3,
    d2v1: Vector3,
    d2uv1: Vector3,
    d1: Vector3,
    normal: NormalJet,
    dnw: Vector3,
    resul: Vector3,
}

impl SurfRstCore {
    /// `tevol->D1(prm)` signed (the law derivative), or 0 without a law.
    fn law_d1(&mut self, prm: f64) -> Result<f64, Failure> {
        let Some(law) = &self.law else {
            return Ok(0.);
        };
        let (ray, dray) = crate::blend_func::choice::live_law(&law.tevol, self.class)?.d1(prm)?;
        self.ray = law.sg1 * ray;
        Ok(law.sg1 * dray)
    }

    fn section_jet(
        &mut self,
        sol: &Vector<'_>,
        dnplan: Vector3,
        dray: f64,
    ) -> Result<(SectionJet, Vector<'static>), Failure> {
        let s = live_surface(&self.surf)?.d2(vget(sol, 1)?, vget(sol, 2)?)?;
        self.pts = s.point;
        let (ptrst, d1) = self.cons.d1(vget(sol, 3)?)?;
        self.ptrst = ptrst;
        let nplan = self.nplan;
        let mut secmember = vector(3)?;
        vset(
            &mut secmember,
            1,
            self.normtg - dnplan.dot(Vector3::from(self.pts.xyz() - self.ptgui.xyz())),
        )?;
        vset(
            &mut secmember,
            2,
            self.normtg - dnplan.dot(Vector3::from(self.ptrst.xyz() - self.ptgui.xyz())),
        )?;
        let ns = s.d1u.crossed(s.d1v);
        let ncrossns = nplan.crossed(ns);
        let ndotns = nplan.dot(ns);
        let mut norm = ncrossns.magnitude();
        if norm < EPS {
            norm = 1.; // Not enough, but it is not necessary to stop
        }
        // Derivative of n1 corresponding to w.
        let grosterme = ncrossns.dot(dnplan.crossed(ns)) / norm / norm;
        let mut dnw = Vector3::default();
        dnw.set_linear_form_three(
            [
                (dnplan.dot(ns) - grosterme * ndotns) / norm,
                ndotns / norm,
                grosterme / norm,
            ],
            [nplan, dnplan, ns],
        );
        let mut temp = Vector3::default();
        temp.set_linear_form(ndotns / norm, nplan, -1. / norm, ns);
        let mut resul = Vector3::default();
        resul.set_scaled_sum(self.ray, temp, Vector3::between(self.ptrst, self.pts));
        let third = match self.law {
            None => -2. * self.ray * dnw.dot(resul),
            Some(_) => {
                -2. * self.ray * (dnw.dot(resul)) - 2. * dray * (temp.dot(resul))
                    + 2. * self.ray * dray
            }
        };
        vset(&mut secmember, 3, third)?;
        let normal = NormalJet {
            ns,
            ncrossns,
            ndotns,
            norm,
        };
        Ok((
            SectionJet {
                d1u1: s.d1u,
                d1v1: s.d1v,
                d2u1: s.d2u,
                d2v1: s.d2v,
                d2uv1: s.d2uv,
                d1,
                normal,
                dnw,
                resul,
            },
            secmember,
        ))
    }

    /// Gauss (pivot 1e-9 for the constant radius, 1e-20 with a law), then SVD; `None`
    /// on a tangency point.
    fn solve_section(
        &self,
        gradsol: &Matrix<'_>,
        mut secmember: Vector<'static>,
    ) -> Result<Option<[f64; 3]>, Failure> {
        let resol = match self.law {
            None => Gauss::with_min_pivot(gradsol, 1.0e-9)?,
            Some(_) => Gauss::new(gradsol)?,
        };
        if resol.is_done() {
            resol.solve(&mut secmember)?;
        } else {
            let mut sing_rs = Svd::new(gradsol)?;
            if !sing_rs.is_done() {
                return Ok(None);
            }
            let dedt = secmember.clone();
            sing_rs.solve(&dedt, &mut secmember, 1.0e-6)?;
        }
        Ok(Some([
            vget(&secmember, 1)?,
            vget(&secmember, 2)?,
            vget(&secmember, 3)?,
        ]))
    }
}

#[path = "sections_d1.rs"]
mod d1;
