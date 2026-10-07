//! The sections of `BlendFunc_ConstRad` at a point of a blend line: poles, 2d poles and
//! weights, with their first and second derivatives along the guide.
use super::super::SectionShape;
use super::super::linalg::{vector, vget, vset};
use super::super::section_solve::solve_derivatives;
use super::ConstRad;
use super::solution::project_normal;
use crate::blend::{BlendPoint, SectionPoles};
use crate::blend_func::linalg::point_from;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkgeomalgo::geom_fill::{ArcPolesD1, CircleD0, CircleD1, get_circle, get_circle_d1};
use rustycad_tkgeombase::approx::{SweepD1, SweepD2};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

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

/// `Poles2d(Lower) = (X1, X2); Poles2d(Upper) = (X3, X4)` and the same for derivatives.
pub(crate) fn put_2d<T>(
    array: &mut Array1<'_, T>,
    values: [f64; 4],
    make: fn(f64, f64) -> T,
) -> Result<(), Failure> {
    let (low, up) = (array.lower(), array.upper());
    put(array, low, make(values[0], values[1]))?;
    put(array, up, make(values[2], values[3]))
}

/// The linear section: the two points with unit weights.
fn linear_ends(
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

/// The normals of a circular section: `ns1`, `ns2` projected, oriented from the center
/// to the points, and the center.
struct CircleFrame {
    ns1: Vector3,
    ns2: Vector3,
    center: Point3,
}

impl ConstRad {
    /// The projected normals and center of the section, before orientation.
    fn circle_frame(&self) -> CircleFrame {
        let nplan = self.plane.nplan;
        let ns1 = project_normal(nplan, self.s1.nsurf);
        let ns2 = project_normal(nplan, self.s2.nsurf);
        let center = point_from(self.s1.pts.xyz() + self.ray1 * *ns1.xyz());
        CircleFrame { ns1, ns2, center }
    }

    fn circle_d0(&self, frame: &CircleFrame, ns1: Vector3, ns2: Vector3, np: Vector3) -> CircleD0 {
        CircleD0 {
            ns1,
            ns2,
            nplan: np,
            pts1: self.s1.pts,
            pts2: self.s2.pts,
            rayon: self.ray1.abs(),
            center: frame.center,
        }
    }

    /// `Section(P, Poles, Poles2d, Weigths)`.
    pub(super) fn section_impl(
        &mut self,
        point: &BlendPoint,
        out: SectionPoles<'_, '_>,
    ) -> Result<(), Failure> {
        let x = point_parameters(point)?;
        self.compute_values(&x, 0, true, point.parameter())?;
        self.distmin = std_min(self.distmin, self.s1.pts.distance(&self.s2.pts));
        let xs = [vget(&x, 1)?, vget(&x, 2)?, vget(&x, 3)?, vget(&x, 4)?];
        put_2d(out.poles2d, xs, Point2::new)?;
        if self.shape == SectionShape::Linear {
            return linear_ends(out.poles, out.weights, [self.s1.pts, self.s2.pts]);
        }
        let frame = self.circle_frame();
        let (mut ns1, mut ns2, mut np) = (frame.ns1, frame.ns2, self.plane.nplan);
        // ns1 (resp. ns2) is oriented from the center to pts1 (resp. pts2), and the
        // trihedron ns1, ns2, nplan is made direct.
        if self.ray1 > 0. {
            ns1.reverse();
        }
        if self.ray2 > 0. {
            ns2.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
        }
        get_circle(
            self.tconv,
            &self.circle_d0(&frame, ns1, ns2, np),
            out.poles,
            out.weights,
        )
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: true when the
    /// derivatives are computed.
    pub(super) fn section_d1_impl(
        &mut self,
        point: &BlendPoint,
        out: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let sol = point_parameters(point)?;
        self.compute_values(&sol, 1, true, point.parameter())?;
        self.distmin = std_min(self.distmin, self.s1.pts.distance(&self.s2.pts));
        let near = self.s1.pts.is_equal(&self.s2.pts, 1.0e-4);
        let secmember = solve_derivatives(&self.eq.dedx, &self.eq.dedt, near)?;
        let n1 = self.s1.normal;
        let n2 = self.s2.normal;
        let (mut dnorm1w, mut dnorm2w) = (Vector3::default(), Vector3::default());
        if let Some(s) = secmember {
            self.tg1
                .set_linear_form(s[0], self.s1.jet.d1u, s[1], self.s1.jet.d1v);
            self.tg2
                .set_linear_form(s[2], self.s2.jet.d1u, s[3], self.s2.jet.d1v);
            dnorm1w.set_linear_form_offset(s[0], n1.dndu, s[1], n1.dndv, n1.dnw);
            dnorm2w.set_linear_form_offset(s[2], n2.dndu, s[3], n2.dndv, n2.dnw);
        }
        let xs = [
            vget(&sol, 1)?,
            vget(&sol, 2)?,
            vget(&sol, 3)?,
            vget(&sol, 4)?,
        ];
        put_2d(out.poles2d, xs, Point2::new)?;
        if let Some(s) = secmember {
            // The native code writes DPoles2d at the bounds of Poles2d.
            let (low, up) = (out.poles2d.lower(), out.poles2d.upper());
            put(out.d_poles2d, low, Vector2::new(s[0], s[1]))?;
            put(out.d_poles2d, up, Vector2::new(s[2], s[3]))?;
        }
        if self.shape == SectionShape::Linear {
            return self.linear_d1(out, secmember.is_some());
        }
        self.circle_d1(out, secmember.is_some(), [dnorm1w, dnorm2w])
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

    /// The circular case of the first-derivative section.
    fn circle_d1(
        &self,
        out: SweepD1<'_, '_>,
        derivatives: bool,
        dnorm: [Vector3; 2],
    ) -> Result<bool, Failure> {
        let frame = self.circle_frame();
        let (mut ns1, mut ns2, mut np, mut dnp) =
            (frame.ns1, frame.ns2, self.plane.nplan, self.plane.dnplan);
        let [mut dnorm1w, mut dnorm2w] = dnorm;
        let mut tgc = Vector3::default();
        if derivatives {
            tgc.set_scaled_sum(self.ray1, dnorm1w, self.tg1); // = tg1.Added(ray1*dn1w)
        }
        // ns1 is oriented from the center to pts1, and ns2 from the center to pts2, and
        // the trihedron ns1, ns2, nplan is made direct.
        if self.ray1 > 0. {
            ns1.reverse();
            dnorm1w.reverse();
        }
        if self.ray2 > 0. {
            ns2.reverse();
            dnorm2w.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
            dnp.reverse();
        }
        let circle = self.circle_d0(&frame, ns1, ns2, np);
        if !derivatives {
            get_circle(self.tconv, &circle, out.poles, out.weights)?;
            return Ok(false);
        }
        let c = CircleD1 {
            circle,
            dn1w: dnorm1w,
            dn2w: dnorm2w,
            dnplan: dnp,
            tang1: self.tg1,
            tang2: self.tg2,
            d_rayon: 0.,
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

    /// `Section(P, ..., D2Weigths)`: true when the derivatives are computed.
    pub(super) fn section_d2_impl(
        &mut self,
        point: &BlendPoint,
        out: SweepD2<'_, '_>,
    ) -> Result<bool, Failure> {
        let x = point_parameters(point)?;
        self.compute_values(&x, 2, true, point.parameter())?;
        self.distmin = std_min(self.distmin, self.s1.pts.distance(&self.s2.pts));
        let near = self.s1.pts.is_equal(&self.s2.pts, 1.0e-4);
        let solved = super::super::section_solve::solve_second_derivatives(&self.eq, near)?;
        let derivatives = solved.map(|(sol, secmember)| self.second_order_tangents(sol, secmember));
        let xs = [vget(&x, 1)?, vget(&x, 2)?, vget(&x, 3)?, vget(&x, 4)?];
        let SweepD2 {
            d1,
            d2_poles,
            d2_poles2d,
            d2_weights,
        } = out;
        put_2d(d1.poles2d, xs, Point2::new)?;
        if let Some((sol, secmember)) = solved {
            let (low, up) = (d1.poles2d.lower(), d1.poles2d.upper());
            put(d1.d_poles2d, low, Vector2::new(sol[0], sol[1]))?;
            put(d1.d_poles2d, up, Vector2::new(sol[2], sol[3]))?;
            put(d2_poles2d, low, Vector2::new(secmember[0], secmember[1]))?;
            put(d2_poles2d, up, Vector2::new(secmember[2], secmember[3]))?;
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
        self.circle_d2(out, derivatives)
    }
}

/// The derivatives along the guide of the section data at second order.
#[derive(Clone, Copy)]
pub(super) struct SecondOrderTangents {
    pub dnorm1w: Vector3,
    pub dnorm2w: Vector3,
    pub d2norm1w: Vector3,
    pub d2norm2w: Vector3,
    pub dtg1: Vector3,
    pub dtg2: Vector3,
}

#[path = "sections_d2.rs"]
mod sections_d2;
