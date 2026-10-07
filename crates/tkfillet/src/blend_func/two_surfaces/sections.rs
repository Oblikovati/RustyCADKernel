//! The sections of the fillet functions at a point of a blend line: poles, 2d poles and
//! weights, with their first derivatives along the guide. The caller runs
//! `ComputeValues` at the point first.
use super::super::SectionShape;
use super::super::linalg::{vector, vget, vset};
use super::super::section_solve::solve_derivatives;
use super::equations::center;
use super::solution::project_normal;
use super::{Radii, TwoSurfaceCore};
use crate::blend::{BlendPoint, SectionPoles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkgeomalgo::geom_fill::{ArcPolesD1, CircleD0, CircleD1, get_circle, get_circle_d1};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

pub(crate) use super::queries::CircleSection;

/// The parameters of `point` on both surfaces as the vector `X`.
pub(crate) fn point_parameters(point: &BlendPoint) -> Result<Vector<'static>, Failure> {
    let (u1, v1) = point.parameters_on_s1()?;
    let (u2, v2) = point.parameters_on_s2()?;
    let mut x = vector(4)?;
    for (i, value) in (1..).zip([u1, v1, u2, v2]) {
        vset(&mut x, i, value)?;
    }
    Ok(x)
}

/// The four values of `x`.
pub(super) fn values4(x: &Vector<'_>) -> Result<[f64; 4], Failure> {
    Ok([vget(x, 1)?, vget(x, 2)?, vget(x, 3)?, vget(x, 4)?])
}

/// `Poles2d(Lower) = (X1, X2); Poles2d(Upper) = (X3, X4)`.
pub(super) fn put_2d<T>(
    array: &mut Array1<'_, T>,
    values: [f64; 4],
    make: fn(f64, f64) -> T,
) -> Result<(), Failure> {
    let (low, up) = (array.lower(), array.upper());
    put(array, low, make(values[0], values[1]))?;
    put(array, up, make(values[2], values[3]))
}

/// The derivatives of the 2d poles, written at the bounds of `Poles2d` as natively.
pub(super) fn put_2d_at(
    array: &mut Array1<'_, Vector2>,
    bounds: (i32, i32),
    values: [f64; 4],
) -> Result<(), Failure> {
    put(array, bounds.0, Vector2::new(values[0], values[1]))?;
    put(array, bounds.1, Vector2::new(values[2], values[3]))
}

/// The linear section: the two points with unit weights.
pub(super) fn linear_ends(
    poles: &mut Array1<'_, Point3>,
    weights: &mut Array1<'_, f64>,
    pts: [Point3; 2],
) -> Result<(), Failure> {
    let (low, upp) = (poles.lower(), poles.upper());
    put(poles, low, pts[0])?;
    put(poles, upp, pts[1])?;
    put(weights, low, 1.0)?;
    put(weights, upp, 1.0)
}

/// The projected normals and the center of a circular section, before orientation.
pub(super) struct CircleFrame {
    pub ns1: Vector3,
    pub ns2: Vector3,
    pub center: Point3,
}

/// The law derivative of the section radius: 0 for a constant radius, `dray` (negated for a
/// negative radius, "to avoid std::abs(dray)") for an evolutive one, and the same for the
/// second derivative.
pub(super) fn radius_derivatives(radii: Radii) -> (f64, f64) {
    match radii {
        Radii::Constant { .. } => (0., 0.),
        Radii::Evolutive {
            ray, dray, d2ray, ..
        } => {
            if ray < 0. {
                (-dray, -d2ray)
            } else {
                (dray, d2ray)
            }
        }
    }
}

impl TwoSurfaceCore {
    /// The projected normals and center of the section.
    pub(super) fn circle_frame(&self, radii: Radii) -> CircleFrame {
        let nplan = self.plane.nplan;
        let ns1 = project_normal(nplan, self.s1.nsurf);
        let ns2 = project_normal(nplan, self.s2.nsurf);
        CircleFrame {
            ns1,
            ns2,
            center: center(self.s1.pts, radii.ray1(), ns1),
        }
    }

    pub(super) fn circle_d0(
        &self,
        frame: &CircleFrame,
        normals: [Vector3; 3],
        radii: Radii,
    ) -> CircleD0 {
        let [ns1, ns2, nplan] = normals;
        CircleD0 {
            ns1,
            ns2,
            nplan,
            pts1: self.s1.pts,
            pts2: self.s2.pts,
            rayon: radii.circle_radius(),
            center: frame.center,
        }
    }

    /// `Section(P, Poles, Poles2d, Weigths)` at the computed point `x`.
    pub(crate) fn section_d0(
        &mut self,
        x: &Vector<'_>,
        out: SectionPoles<'_, '_, '_, '_>,
        radii: Radii,
    ) -> Result<(), Failure> {
        self.record_distance();
        put_2d(out.poles2d, values4(x)?, Point2::new)?;
        if self.shape == SectionShape::Linear {
            return linear_ends(out.poles, out.weights, [self.s1.pts, self.s2.pts]);
        }
        let frame = self.circle_frame(radii);
        let (mut ns1, mut ns2, mut np) = (frame.ns1, frame.ns2, self.plane.nplan);
        // ns1 (resp. ns2) is oriented from the center to pts1 (resp. pts2), and the
        // trihedron ns1, ns2, nplan is made direct.
        if radii.positive(false) {
            ns1.reverse();
        }
        if radii.positive(true) {
            ns2.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
        }
        get_circle(
            self.tconv,
            &self.circle_d0(&frame, [ns1, ns2, np], radii),
            out.poles,
            out.weights,
        )
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)` at the computed
    /// point `sol`: true when the derivatives are computed.
    pub(crate) fn section_d1(
        &mut self,
        sol: &Vector<'_>,
        out: SweepD1<'_, '_>,
        radii: Radii,
    ) -> Result<bool, Failure> {
        self.record_distance();
        let near = self.s1.pts.is_equal(&self.s2.pts, 1.0e-4);
        let secmember = solve_derivatives(&self.eq.dedx, &self.eq.dedt, near)?;
        let (n1, n2) = (self.s1.normal, self.s2.normal);
        let (mut dnorm1w, mut dnorm2w) = (Vector3::default(), Vector3::default());
        if let Some(s) = secmember {
            self.tg1
                .set_linear_form(s[0], self.s1.jet.d1u, s[1], self.s1.jet.d1v);
            self.tg2
                .set_linear_form(s[2], self.s2.jet.d1u, s[3], self.s2.jet.d1v);
            dnorm1w.set_linear_form_offset(s[0], n1.dndu, s[1], n1.dndv, n1.dnw);
            dnorm2w.set_linear_form_offset(s[2], n2.dndu, s[3], n2.dndv, n2.dnw);
        }
        put_2d(out.poles2d, values4(sol)?, Point2::new)?;
        if let Some(s) = secmember {
            let bounds = (out.poles2d.lower(), out.poles2d.upper());
            put_2d_at(out.d_poles2d, bounds, s)?;
        }
        if self.shape == SectionShape::Linear {
            return self.linear_d1(out, secmember.is_some());
        }
        self.circle_d1(out, secmember.is_some(), [dnorm1w, dnorm2w], radii)
    }

    /// The linear case of the first-derivative section.
    fn linear_d1(&self, out: SweepD1<'_, '_>, derivatives: bool) -> Result<bool, Failure> {
        linear_ends(out.poles, out.weights, [self.s1.pts, self.s2.pts])?;
        if derivatives {
            let (low, upp) = (out.poles.lower(), out.poles.upper());
            put(out.d_poles, low, self.tg1)?;
            put(out.d_poles, upp, self.tg2)?;
            put(out.d_weights, low, 0.0)?;
            put(out.d_weights, upp, 0.0)?;
        }
        Ok(derivatives)
    }

    /// `tgc`: the derivative of the center along the guide.
    pub(super) fn center_derivative(
        &self,
        radii: Radii,
        dnorm1w: Vector3,
        ns1: Vector3,
    ) -> Vector3 {
        let mut tgc = Vector3::default();
        match radii {
            // = tg1.Added(ray1*dn1w)
            Radii::Constant { ray1, .. } => tgc.set_scaled_sum(ray1, dnorm1w, self.tg1),
            Radii::Evolutive { sg1, ray, dray, .. } => {
                tgc.set_linear_form_offset(sg1 * ray, dnorm1w, sg1 * dray, ns1, self.tg1)
            }
        }
        tgc
    }

    /// The circular case of the first-derivative section.
    fn circle_d1(
        &self,
        out: SweepD1<'_, '_>,
        derivatives: bool,
        dnorm: [Vector3; 2],
        radii: Radii,
    ) -> Result<bool, Failure> {
        let frame = self.circle_frame(radii);
        let (mut ns1, mut ns2, mut np, mut dnp) =
            (frame.ns1, frame.ns2, self.plane.nplan, self.plane.dnplan);
        let [mut dnorm1w, mut dnorm2w] = dnorm;
        let tgc = if derivatives {
            self.center_derivative(radii, dnorm1w, ns1)
        } else {
            Vector3::default()
        };
        // ns1 is oriented from the center to pts1, and ns2 from the center to pts2, and
        // the trihedron ns1, ns2, nplan is made direct.
        if radii.positive(false) {
            ns1.reverse();
            dnorm1w.reverse();
        }
        if radii.positive(true) {
            ns2.reverse();
            dnorm2w.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
            dnp.reverse();
        }
        let circle = self.circle_d0(&frame, [ns1, ns2, np], radii);
        if !derivatives {
            get_circle(self.tconv, &circle, out.poles, out.weights)?;
            return Ok(false);
        }
        let d_rayon = radius_derivatives(radii).0;
        let c = CircleD1 {
            circle,
            dn1w: dnorm1w,
            dn2w: dnorm2w,
            dnplan: dnp,
            tang1: self.tg1,
            tang2: self.tg2,
            d_rayon,
            d_center: tgc,
        };
        let arc = ArcPolesD1 {
            poles: out.poles,
            d_poles: out.d_poles,
            weights: out.weights,
            d_weights: out.d_weights,
        };
        get_circle_d1(self.tconv, &c, arc)
    }
}
