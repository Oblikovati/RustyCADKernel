//! The output of the first-order section of the surface-restriction fillets: the
//! derivatives of the section normals, the 2d poles and the circle.
use super::SectionJet;
use crate::blend::BlendPoint;
use crate::blend_func::SectionShape;
use crate::blend_func::linalg::point_from;
use crate::blend_func::package::live_surface;
use crate::brep_blend::surf_rst::{SurfRstCore, t3dto2d};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkgeomalgo::geom_fill::{ArcPolesD1, CircleD1, get_circle};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::gp::{Point2, Vector2, Vector3};

/// The normals of the section at a non-tangency point with their derivatives.
struct SectionNormals {
    ns: Vector3,
    ns2: Vector3,
    dnw: Vector3,
    dn2w: Vector3,
}

impl SurfRstCore {
    /// The derivative of the projected surface normal along a surface parameter.
    fn normal_derivative(jet: &SectionJet, nplan: Vector3, temp: Vector3) -> Vector3 {
        let n = &jet.normal;
        let grosterme = n.ncrossns.dot(nplan.crossed(temp)) / n.norm / n.norm;
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                -(grosterme * n.ndotns - nplan.dot(temp)) / n.norm,
                grosterme / n.norm,
                -1. / n.norm,
            ],
            [nplan, n.ns, temp],
        );
        out
    }

    /// The tangents and the normals at a solved section (`istgt` false).
    fn solved_normals(
        &mut self,
        jet: &SectionJet,
        s: [f64; 3],
        dray: f64,
    ) -> Result<SectionNormals, Failure> {
        let nplan = self.nplan;
        self.tgs.set_linear_form(s[0], jet.d1u1, s[1], jet.d1v1);
        self.tgrst = s[2] * jet.d1;
        let resulu = Self::normal_derivative(
            jet,
            nplan,
            jet.d2u1.crossed(jet.d1v1) + jet.d1u1.crossed(jet.d2uv1),
        );
        let resulv = Self::normal_derivative(
            jet,
            nplan,
            jet.d2uv1.crossed(jet.d1v1) + jet.d1u1.crossed(jet.d2v1),
        );
        let mut dnw = Vector3::default();
        dnw.set_linear_form_offset(s[0], resulu, s[1], resulv, jet.dnw);
        let ns = self.projected_jet(jet);
        let mut dn2w = Vector3::default();
        dn2w.set_linear_form_offset(self.ray, dnw, -1., self.tgrst, self.tgs);
        if self.law.is_some() {
            let previous = dn2w;
            dn2w.set_scaled_sum(dray, ns, previous);
        }
        dn2w /= jet.resul.magnitude();
        let ns2 = -jet.resul.normalized()?;
        let previous = dn2w;
        dn2w.set_linear_form(ns2.dot(previous), ns2, -1., previous);
        Ok(SectionNormals { ns, ns2, dnw, dn2w })
    }

    /// `ns.SetLinearForm(ndotns / norm, nplan, -1. / norm, ns)` on the jet normal.
    fn projected_jet(&self, jet: &SectionJet) -> Vector3 {
        let n = &jet.normal;
        let mut out = Vector3::default();
        out.set_linear_form(n.ndotns / n.norm, self.nplan, -1. / n.norm, n.ns);
        out
    }

    pub(super) fn section_d1_output(
        &mut self,
        point: &BlendPoint,
        out: SweepD1<'_, '_>,
        (jet, mut dnplan, dray): (SectionJet, Vector3, f64),
        solved: Option<[f64; 3]>,
    ) -> Result<bool, Failure> {
        let normals = match solved {
            Some(s) => Some(self.solved_normals(&jet, s, dray)?),
            None => None,
        };
        let (mut ns, ns2) = match &normals {
            Some(n) => (n.ns, n.ns2),
            None => (self.projected_jet(&jet), -jet.resul.normalized()?),
        };
        let (u1, v1) = point.parameters_on_s()?;
        let (low2d, upp2d) = (out.poles2d.lower(), out.poles2d.upper());
        put(out.poles2d, low2d, Point2::new(u1, v1))?;
        put(out.poles2d, upp2d, self.pt2drst)?;
        if let Some(s) = solved {
            put(out.d_poles2d, low2d, Vector2::new(s[0], s[1]))?;
            let rst = live_surface(&self.surfrst)?.d1(self.pt2drst.x(), self.pt2drst.y())?;
            let (a, b) = t3dto2d(self.tgrst, rst.d1u, rst.d1v);
            put(out.d_poles2d, upp2d, Vector2::new(a, b))?;
        }
        if self.shape == SectionShape::Linear {
            return self.linear_d1(out, solved.is_some());
        }
        let center = point_from(self.pts.xyz() + *ns.xyz() * self.ray);
        let tgct = normals
            .as_ref()
            .map(|n| self.center_tangent(n.dnw, ns, dray));
        let mut dnw = normals.as_ref().map(|n| n.dnw);
        if self.ray > 0. {
            ns.reverse();
            if let Some(dnw) = &mut dnw {
                dnw.reverse();
            }
        }
        if self.choix % 2 != 0 {
            self.nplan.reverse();
            dnplan.reverse();
        }
        let circle = self.circle(ns, ns2, center);
        let (Some(normals), Some(dnw), Some(tgct)) = (normals, dnw, tgct) else {
            get_circle(self.tconv, &circle, out.poles, out.weights)?;
            return Ok(false);
        };
        let c = CircleD1 {
            circle,
            dn1w: dnw,
            dn2w: normals.dn2w,
            dnplan,
            tang1: self.tgs,
            tang2: self.tgrst,
            d_rayon: self.rayprim(dray),
            d_center: tgct,
        };
        let arc = ArcPolesD1 {
            poles: out.poles,
            d_poles: out.d_poles,
            weights: out.weights,
            d_weights: out.d_weights,
        };
        rustycad_tkgeomalgo::geom_fill::get_circle_d1(self.tconv, &c, arc)
    }

    /// The center derivative: `tgs + ray dnw`, with a law `ray dnw + dray ns + tgs`.
    fn center_tangent(&self, dnw: Vector3, ns: Vector3, dray: f64) -> Vector3 {
        let mut tgct = Vector3::default();
        match self.law {
            None => tgct.set_scaled_sum(self.ray, dnw, self.tgs),
            Some(_) => tgct.set_linear_form_offset(self.ray, dnw, dray, ns, self.tgs),
        }
        tgct
    }

    /// The radius derivative of the circle: 0 for a constant radius, `|ray|'` with a law
    /// ("to avoid std::abs(dray)").
    fn rayprim(&self, dray: f64) -> f64 {
        match self.law {
            None => 0.,
            Some(_) if self.ray < 0. => -dray,
            Some(_) => dray,
        }
    }

    fn linear_d1(&self, out: SweepD1<'_, '_>, solved: bool) -> Result<bool, Failure> {
        let (low, upp) = (out.poles.lower(), out.poles.upper());
        self.linear_poles(out.poles, out.weights)?;
        if solved {
            put(out.d_poles, low, self.tgs)?;
            put(out.d_poles, upp, self.tgrst)?;
            put(out.d_weights, low, 0.0)?;
            put(out.d_weights, upp, 0.0)?;
        }
        Ok(solved)
    }
}
