//! The derivatives of the section normals and the output of the first-order section of
//! the restriction-restriction fillets.
use super::RstRstCircle;
use crate::blend_func::SectionShape;
use crate::blend_func::package::live_surface;
use crate::brep_blend::rst_rst::RstRstCore;
use crate::brep_blend::surf_rst::t3dto2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkgeomalgo::geom_fill::{ArcPolesD1, CircleD1, get_circle, get_circle_d1};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::gp::{Vector2, Vector3};

/// The derivatives of the normals along the guide at a solved section.
pub(super) struct NormalDerivatives {
    d1n1: Vector3,
    d1n2: Vector3,
}

impl RstRstCore {
    /// The tangents on the restrictions and the derivatives of the normals from the center.
    pub(super) fn normal_derivatives(
        &mut self,
        c: &RstRstCircle,
        s: [f64; 2],
        (d11, d21): (Vector3, Vector3),
        (dnplan, dray): (Vector3, f64),
    ) -> Result<NormalDerivatives, Failure> {
        self.tgrst1 = s[0] * d11;
        self.tgrst2 = s[1] * d21;
        let rst1rst2 = Vector3::between(self.ptrst1, self.ptrst2);
        let norm2 = rst1rst2.square_magnitude();
        let mut d1rst1rst2 = self.tgrst2 - self.tgrst1;
        let ray = self.ray;
        let dist = ray * ray - 0.25 * norm2;
        let invdray = dray / ray;
        let mut dmed = Vector3::default();
        if dist > 1.0e-7 {
            dmed = self.bisector_derivative(c, (rst1rst2, d1rst1rst2), dnplan, (dist, dray));
        }
        d1rst1rst2 *= 0.5;
        let law = self.law.is_some();
        let (d1n1, d1n2) = match (dist > 1.0e-7, law) {
            (true, false) => (-(dmed + d1rst1rst2) / ray, (d1rst1rst2 - dmed) / ray),
            (true, true) => (
                -(d1rst1rst2 + dmed + invdray * c.n1) / ray,
                (d1rst1rst2 - dmed - invdray * c.n2) / ray,
            ),
            (false, false) => (-d1rst1rst2 / ray, d1rst1rst2 / ray),
            (false, true) => (
                -(d1rst1rst2 + invdray * c.n1) / ray,
                (d1rst1rst2 - invdray * c.n2) / ray,
            ),
        };
        Ok(NormalDerivatives { d1n1, d1n2 })
    }

    /// The derivative of the bisector with the derivative of the distance to the center
    /// (`dmed.SetLinearForm(Dist, dmed, d1Dist, med)`).
    fn bisector_derivative(
        &self,
        c: &RstRstCircle,
        (rst1rst2, d1rst1rst2): (Vector3, Vector3),
        dnplan: Vector3,
        (dist, dray): (f64, f64),
    ) -> Vector3 {
        let d1p1p2_cros_np = d1rst1rst2.crossed(self.nplan) + rst1rst2.crossed(dnplan);
        let mut dmed = d1p1p2_cros_np - c.med.dot(d1p1p2_cros_np) * c.med;
        dmed /= c.normmed;
        let dist = dist.sqrt();
        let d1_dist = match self.law {
            None => -(0.25 / dist) * rst1rst2.dot(d1rst1rst2),
            Some(_) => (self.ray * dray - 0.25 * rst1rst2.dot(d1rst1rst2)) / dist,
        };
        if self.choix > 2 {
            dmed.reverse();
        }
        let previous = dmed;
        dmed.set_linear_form(dist, previous, d1_dist, c.med);
        dmed
    }

    pub(super) fn section_d1_output(
        &mut self,
        out: SweepD1<'_, '_>,
        mut c: RstRstCircle,
        derivatives: Option<NormalDerivatives>,
        (dnplan, dray): (&mut Vector3, f64),
    ) -> Result<bool, Failure> {
        if self.law.is_some() {
            c.n1 = c.n1.normalized()?;
            c.n2 = c.n2.normalized()?;
        }
        let (low2d, upp2d) = (out.poles2d.lower(), out.poles2d.upper());
        put(out.poles2d, low2d, self.pt2drst1)?;
        put(out.poles2d, upp2d, self.pt2drst2)?;
        if derivatives.is_some() {
            self.tangents_2d(out.d_poles2d, (low2d, upp2d))?;
        }
        if self.shape == SectionShape::Linear {
            return self.linear_d1(out, derivatives.is_some());
        }
        let tgct = derivatives
            .as_ref()
            .map(|d| self.center_tangent(d.d1n1, c.n1, dray));
        if self.choix % 2 != 0 {
            self.nplan.reverse();
            dnplan.reverse();
        }
        let circle = self.circle(c.n1, c.n2, c.center);
        let (Some(d), Some(tgct)) = (derivatives, tgct) else {
            get_circle(self.tconv, &circle, out.poles, out.weights)?;
            return Ok(false);
        };
        let data = CircleD1 {
            circle,
            dn1w: d.d1n1,
            dn2w: d.d1n2,
            dnplan: *dnplan,
            tang1: self.tgrst1,
            tang2: self.tgrst2,
            d_rayon: if self.law.is_some() { dray } else { 0. },
            d_center: tgct,
        };
        let arc = ArcPolesD1 {
            poles: out.poles,
            d_poles: out.d_poles,
            weights: out.weights,
            d_weights: out.d_weights,
        };
        get_circle_d1(self.tconv, &data, arc)
    }

    /// `DPoles2d`: the 3d tangents in the parameters of each restriction surface.
    fn tangents_2d(
        &self,
        d_poles2d: &mut rustycad_tkernel::ncollection::Array1<'_, Vector2>,
        (low, upp): (i32, i32),
    ) -> Result<(), Failure> {
        let a = live_surface(&self.r1.surf)?.d1(self.pt2drst1.x(), self.pt2drst1.y())?;
        let (u, v) = t3dto2d(self.tgrst1, a.d1u, a.d1v);
        put(d_poles2d, low, Vector2::new(u, v))?;
        let b = live_surface(&self.r2.surf)?.d1(self.pt2drst2.x(), self.pt2drst2.y())?;
        let (u, v) = t3dto2d(self.tgrst2, b.d1u, b.d1v);
        put(d_poles2d, upp, Vector2::new(u, v))
    }

    /// The center tangent: `-ray d1n1 + tgrst1`, with a law `-ray d1n1 - dray n1 + tgrst1`.
    fn center_tangent(&self, d1n1: Vector3, n1: Vector3, dray: f64) -> Vector3 {
        let mut tgct = Vector3::default();
        match self.law {
            None => tgct.set_scaled_sum(-self.ray, d1n1, self.tgrst1),
            Some(_) => tgct.set_linear_form_offset(-self.ray, d1n1, -dray, n1, self.tgrst1),
        }
        tgct
    }

    fn linear_d1(&self, out: SweepD1<'_, '_>, solved: bool) -> Result<bool, Failure> {
        let (low, upp) = (out.poles.lower(), out.poles.upper());
        self.linear_poles(out.poles, out.weights)?;
        if solved {
            put(out.d_poles, low, self.tgrst1)?;
            put(out.d_poles, upp, self.tgrst2)?;
            put(out.d_weights, low, 0.0)?;
            put(out.d_weights, upp, 0.0)?;
        }
        Ok(solved)
    }
}
