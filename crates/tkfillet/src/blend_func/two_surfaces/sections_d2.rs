//! The second-derivative section of the fillet functions.
use super::super::SectionShape;
use super::super::section_solve::solve_second_derivatives;
use super::sections::{put_2d, put_2d_at, radius_derivatives, values4};
use super::{NormalDerivatives, Radii, TwoSurfaceCore};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkgeomalgo::geom_fill::{
    ArcPolesD1, ArcPolesD2, CircleD1, CircleD2, get_circle, get_circle_d2,
};
use rustycad_tkgeombase::approx::SweepD2;
use rustycad_tkmath::gp::{Point2, Vector3};
use rustycad_tkmath::math::Vector;

/// The derivatives along the guide of the section data at second order.
#[derive(Clone, Copy, Default)]
struct SecondOrderTangents {
    dnorm1w: Vector3,
    dnorm2w: Vector3,
    d2norm1w: Vector3,
    d2norm2w: Vector3,
    dtg1: Vector3,
    dtg2: Vector3,
}

/// `temp.SetLinearForm(s1 s1, d2u, 2 s1 s2, d2uv, s2 s2, d2v)`.
fn quadratic(a: f64, b: f64, uu: Vector3, uv: Vector3, vv: Vector3) -> Vector3 {
    let mut temp = Vector3::default();
    temp.set_linear_form_three([a * a, 2. * a * b, b * b], [uu, uv, vv]);
    temp
}

/// `d2norm1w`: from the normal second derivatives along the surface and the guide.
fn second_normal(a: f64, b: f64, sa: f64, sb: f64, n: &NormalDerivatives) -> Vector3 {
    let mut temp = quadratic(a, b, n.d2ndu, n.d2nduv, n.d2ndv);
    let mut tempbis = Vector3::default();
    tempbis.set_linear_form_offset(2. * a, n.d2ndtu, 2. * b, n.d2ndtv, n.d2nw);
    temp += tempbis;
    let mut out = Vector3::default();
    out.set_linear_form_offset(sa, n.dndu, sb, n.dndv, temp);
    out
}

impl TwoSurfaceCore {
    /// `Section(P, ..., D2Weigths)` at the computed point `x`: true when the derivatives
    /// are computed.
    pub(crate) fn section_d2(
        &mut self,
        x: &Vector<'_>,
        out: SweepD2<'_, '_>,
        radii: Radii,
    ) -> Result<bool, Failure> {
        self.record_distance();
        let near = self.s1.pts.is_equal(&self.s2.pts, 1.0e-4);
        let solved = solve_second_derivatives(&self.eq, near)?;
        let derivatives = solved.map(|(sol, secmember)| self.second_order_tangents(sol, secmember));
        let SweepD2 {
            d1,
            d2_poles,
            d2_poles2d,
            d2_weights,
        } = out;
        put_2d(d1.poles2d, values4(x)?, Point2::new)?;
        if let Some((sol, secmember)) = solved {
            let bounds = (d1.poles2d.lower(), d1.poles2d.upper());
            put_2d_at(d1.d_poles2d, bounds, sol)?;
            put_2d_at(d2_poles2d, bounds, secmember)?;
        }
        let out = SweepD2 {
            d1,
            d2_poles,
            d2_poles2d,
            d2_weights,
        };
        if self.shape == SectionShape::Linear {
            return self.linear_d2(out, derivatives);
        }
        self.circle_d2(out, derivatives, radii)
    }

    /// The tangents and normal derivatives from the solutions `sol` (first order) and
    /// `secmember` (second order); sets `tg1` and `tg2`.
    fn second_order_tangents(&mut self, sol: [f64; 4], secmember: [f64; 4]) -> SecondOrderTangents {
        let (j1, j2, n1, n2) = (self.s1.jet, self.s2.jet, self.s1.normal, self.s2.normal);
        self.tg1.set_linear_form(sol[0], j1.d1u, sol[1], j1.d1v);
        self.tg2.set_linear_form(sol[2], j2.d1u, sol[3], j2.d1v);
        let mut out = SecondOrderTangents::default();
        out.dnorm1w
            .set_linear_form_offset(sol[0], n1.dndu, sol[1], n1.dndv, n1.dnw);
        out.dnorm2w
            .set_linear_form_offset(sol[2], n2.dndu, sol[3], n2.dndv, n2.dnw);
        let temp = quadratic(sol[0], sol[1], j1.d2u, j1.d2uv, j1.d2v);
        out.dtg1
            .set_linear_form_offset(secmember[0], j1.d1u, secmember[1], j1.d1v, temp);
        let temp = quadratic(sol[2], sol[3], j2.d2u, j2.d2uv, j2.d2v);
        out.dtg2
            .set_linear_form_offset(secmember[2], j2.d1u, secmember[3], j2.d1v, temp);
        out.d2norm1w = second_normal(sol[0], sol[1], secmember[0], secmember[1], &n1);
        out.d2norm2w = second_normal(sol[2], sol[3], secmember[2], secmember[3], &n2);
        out
    }

    /// The linear case: as natively, the second derivatives of the ends overwrite the
    /// first derivatives in `DPoles` (`D2Poles` is left unwritten).
    fn linear_d2(
        &self,
        out: SweepD2<'_, '_>,
        derivatives: Option<SecondOrderTangents>,
    ) -> Result<bool, Failure> {
        let SweepD2 { d1, d2_weights, .. } = out;
        let (low, upp) = (d1.poles.lower(), d1.poles.upper());
        put(d1.poles, low, self.s1.pts)?;
        put(d1.poles, upp, self.s2.pts)?;
        put(d1.weights, low, 1.0)?;
        put(d1.weights, upp, 1.0)?;
        let Some(t) = derivatives else {
            return Ok(false);
        };
        put(d1.d_poles, low, self.tg1)?;
        put(d1.d_poles, upp, self.tg2)?;
        put(d1.d_poles, low, t.dtg1)?;
        put(d1.d_poles, upp, t.dtg2)?;
        put(d1.d_weights, low, 0.0)?;
        put(d1.d_weights, upp, 0.0)?;
        put(d2_weights, low, 0.0)?;
        put(d2_weights, upp, 0.0)?;
        Ok(true)
    }

    /// `dtgc`: the second derivative of the center along the guide.
    fn center_second_derivative(radii: Radii, t: &SecondOrderTangents, ns1: Vector3) -> Vector3 {
        let mut dtgc = Vector3::default();
        match radii {
            Radii::Constant { ray1, .. } => dtgc.set_scaled_sum(ray1, t.d2norm1w, t.dtg1),
            Radii::Evolutive {
                sg1,
                ray,
                dray,
                d2ray,
                ..
            } => {
                dtgc.set_linear_form_three(
                    [sg1 * ray, 2. * sg1 * dray, sg1 * d2ray],
                    [t.d2norm1w, t.dnorm1w, ns1],
                );
                dtgc += t.dtg1;
            }
        }
        dtgc
    }

    /// The circular case of the second-derivative section.
    fn circle_d2(
        &self,
        out: SweepD2<'_, '_>,
        derivatives: Option<SecondOrderTangents>,
        radii: Radii,
    ) -> Result<bool, Failure> {
        let frame = self.circle_frame(radii);
        let (mut ns1, mut ns2) = (frame.ns1, frame.ns2);
        let (mut np, mut dnp, mut d2np) = (self.plane.nplan, self.plane.dnplan, self.plane.d2nplan);
        let mut t = derivatives.unwrap_or_default();
        let (mut tgc, mut dtgc) = (Vector3::default(), Vector3::default());
        if derivatives.is_some() {
            tgc = self.center_derivative(radii, t.dnorm1w, ns1);
            dtgc = Self::center_second_derivative(radii, &t, ns1);
        }
        // ns1 is oriented from the center to pts1 and ns2 from the center to pts2;
        // the trihedron ns1, ns2, nplan is made direct.
        if radii.positive(false) {
            ns1.reverse();
            t.dnorm1w.reverse();
            t.d2norm1w.reverse();
        }
        if radii.positive(true) {
            ns2.reverse();
            t.dnorm2w.reverse();
            t.d2norm2w.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
            dnp.reverse();
            d2np.reverse();
        }
        let SweepD2 {
            d1,
            d2_poles,
            d2_weights,
            ..
        } = out;
        if derivatives.is_none() {
            // As natively, the fallback circle uses the field nplan, not the oriented np.
            let circle = self.circle_d0(&frame, [ns1, ns2, self.plane.nplan], radii);
            get_circle(self.tconv, &circle, d1.poles, d1.weights)?;
            return Ok(false);
        }
        let (d_rayon, d2_rayon) = radius_derivatives(radii);
        let circle = self.circle_d0(&frame, [ns1, ns2, np], radii);
        let c1 = CircleD1 {
            circle,
            dn1w: t.dnorm1w,
            dn2w: t.dnorm2w,
            dnplan: dnp,
            tang1: self.tg1,
            tang2: self.tg2,
            d_rayon,
            d_center: tgc,
        };
        let c = CircleD2 {
            d1: c1,
            d2n1w: t.d2norm1w,
            d2n2w: t.d2norm2w,
            d2nplan: d2np,
            dtang1: t.dtg1,
            dtang2: t.dtg2,
            d2_rayon,
            d2_center: dtgc,
        };
        let arc = ArcPolesD2 {
            d1: ArcPolesD1 {
                poles: d1.poles,
                d_poles: d1.d_poles,
                weights: d1.weights,
                d_weights: d1.d_weights,
            },
            d2_poles,
            d2_weights,
        };
        get_circle_d2(self.tconv, &c, arc)
    }
}
