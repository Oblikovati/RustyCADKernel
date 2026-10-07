//! `Derivatives` and `Values` of the inverse fillet functions: the Jacobian of the
//! inverse equations. `BlendFunc_ConstRadInv::Derivatives` derives the plane normal with
//! `theD` where every other member uses `-nplan.d2cur` (a native slip kept as is), hence
//! the [`PlaneDerivative`] choice.
use super::{InverseCore, InverseRadius, plane_residual, repaired};
use crate::blend_func::linalg::{mset, vset};
use crate::blend_func::normal_terms::EPS;
use crate::blend_func::package::{live_curve, live_surface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The coefficient of `nplan` in the derivative of the plane normal along the guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaneDerivative {
    /// `dnplan.SetLinearForm(theD, nplan, d2cur)`: `BlendFunc_ConstRadInv::Derivatives`.
    FromTheD,
    /// `dnplan.SetLinearForm(-nplan.Dot(d2cur), nplan, d2cur)`.
    Exact,
}

/// The guide frame at `w`: the point, the plane normal, its derivative and the speed.
struct GuideFrame {
    ptcur: Point3,
    nplan: Vector3,
    dnplan: Vector3,
    normtgcur: f64,
    the_d: f64,
}

/// The evaluations both members share.
struct Jet {
    guide: GuideFrame,
    v2d: Vector2,
    s1: GeomSurfaceD2,
    s2: GeomSurfaceD2,
    ns1: Vector3,
    ns2: Vector3,
}

impl InverseCore {
    fn guide_frame(&self, w: f64, plane: PlaneDerivative) -> Result<GuideFrame, Failure> {
        let (ptcur, d1cur, d2cur) = live_curve(&self.curv)?.d2(w)?;
        let normtgcur = d1cur.magnitude();
        let nplan = d1cur.normalized()?;
        let the_d = -nplan.xyz().dot(ptcur.xyz());
        let mut dnplan = Vector3::default();
        let coefficient = match plane {
            PlaneDerivative::FromTheD => the_d,
            PlaneDerivative::Exact => -nplan.dot(d2cur),
        };
        dnplan.set_scaled_sum(coefficient, nplan, d2cur);
        dnplan /= normtgcur;
        Ok(GuideFrame {
            ptcur,
            nplan,
            dnplan,
            normtgcur,
            the_d,
        })
    }

    /// Positions on the guide, the curve on surface and both surfaces; writes `D(1, j)`.
    fn jet(
        &self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
        plane: PlaneDerivative,
    ) -> Result<Jet, Failure> {
        let x = [
            x.value(1).copied()?,
            x.value(2).copied()?,
            x.value(3).copied()?,
            x.value(4).copied()?,
        ];
        let guide = self.guide_frame(x[1], plane)?;
        let (p2d, v2d) = self.live_csurf()?.d1(x[0])?;
        let (uv1, uv2) = self.surface_parameters(p2d, Point2::new(x[2], x[3]));
        let s1 = live_surface(&self.surf1)?.d2(uv1.x(), uv1.y())?;
        let s2 = live_surface(&self.surf2)?.d2(uv2.x(), uv2.y())?;
        let (on_curve, other_surface) = if self.first { (&s1, &s2) } else { (&s2, &s1) };
        let mut temp = Vector3::default();
        temp.set_linear_form(v2d.x(), on_curve.d1u, v2d.y(), on_curve.d1v);
        mset(d, 1, 1, guide.nplan.dot(temp) / 2.)?;
        let middle = Vector3::from(0.5 * (s1.point.xyz() + s2.point.xyz()) - guide.ptcur.xyz());
        mset(d, 1, 2, guide.dnplan.dot(middle) - guide.normtgcur)?;
        mset(d, 1, 3, guide.nplan.dot(other_surface.d1u) / 2.)?;
        mset(d, 1, 4, guide.nplan.dot(other_surface.d1v) / 2.)?;
        let ns1 = repaired(&self.surf1, uv1, s1.d1u.crossed(s1.d1v))?;
        let ns2 = repaired(&self.surf2, uv2, s2.d1u.crossed(s2.d1v))?;
        Ok(Jet {
            guide,
            v2d,
            s1,
            s2,
            ns1,
            ns2,
        })
    }

    /// `Derivatives(X, D)` without the residual.
    pub(crate) fn derivatives(
        &self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
        radius: InverseRadius,
        plane: PlaneDerivative,
    ) -> Result<bool, Failure> {
        let jet = self.jet(x, d, plane)?;
        jacobian(self.first, &jet, d, radius)?;
        Ok(true)
    }

    /// `Values(X, F, D)`.
    pub(crate) fn values(
        &self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
        radius: InverseRadius,
    ) -> Result<bool, Failure> {
        let jet = self.jet(x, d, PlaneDerivative::Exact)?;
        let (pts1, pts2, g) = (jet.s1.point, jet.s2.point, &jet.guide);
        vset(f, 1, plane_residual(g.nplan, g.the_d, pts1, pts2))?;
        let resul = section_values(radius.signed(), &jet);
        vset(f, 2, resul.x())?;
        vset(f, 3, resul.y())?;
        vset(f, 4, resul.z())?;
        jacobian(self.first, &jet, d, radius)?;
        Ok(true)
    }
}

/// `F(2..4)` of `Values`: `resul1.SetLinearForm(ray1, temp, gp_Vec(pts2, pts1))` minus
/// `ray2 * temp`, an operation order distinct from `Value`.
fn section_values((ray1, ray2): (f64, f64), jet: &Jet) -> Vector3 {
    let t = Terms::new(jet.guide.nplan, jet.ns1, jet.ns2);
    let mut temp = Vector3::default();
    temp.set_linear_form(t.ndotns1 / t.norm1, jet.guide.nplan, -1. / t.norm1, jet.ns1);
    let mut resul1 = Vector3::default();
    resul1.set_scaled_sum(ray1, temp, Vector3::between(jet.s2.point, jet.s1.point));
    temp.set_linear_form(t.ndotns2 / t.norm2, jet.guide.nplan, -1. / t.norm2, jet.ns2);
    resul1 -= ray2 * temp;
    resul1
}

/// The rows 2..4 of the Jacobian.
fn jacobian(
    first: bool,
    jet: &Jet,
    d: &mut Matrix<'_>,
    radius: InverseRadius,
) -> Result<(), Failure> {
    let nplan = jet.guide.nplan;
    let t = Terms::new(nplan, jet.ns1, jet.ns2);
    let (ray1, ray2) = radius.signed();
    let (s1, s2) = (&jet.s1, &jet.s2);
    // Derived compared to u1 and v1.
    let side1 = (t.ncrossns1, t.norm1, t.ndotns1);
    let du1 = surface_column(
        nplan,
        jet.ns1,
        side1,
        ray1,
        s1.d2u.crossed(s1.d1v) + s1.d1u.crossed(s1.d2uv),
        s1.d1u,
    );
    let dv1 = surface_column(
        nplan,
        jet.ns1,
        side1,
        ray1,
        s1.d2uv.crossed(s1.d1v) + s1.d1u.crossed(s1.d2v),
        s1.d1v,
    );
    if first {
        curve_column(d, jet.v2d, du1, dv1)?;
    } else {
        surface_columns(d, du1, dv1)?;
    }
    // Derived compared to w (parameter on guideline).
    let dw = guide_column(jet, &t, radius);
    mset(d, 2, 2, dw[0])?;
    mset(d, 3, 2, dw[1])?;
    mset(d, 4, 2, dw[2])?;
    // Derived compared to u2 and v2.
    let side2 = (t.ncrossns2, t.norm2, t.ndotns2);
    let du2 = second_surface_column(
        nplan,
        jet.ns2,
        side2,
        ray2,
        s2.d2u.crossed(s2.d1v) + s2.d1u.crossed(s2.d2uv),
        s2.d1u,
    );
    let dv2 = second_surface_column(
        nplan,
        jet.ns2,
        side2,
        ray2,
        s2.d2uv.crossed(s2.d1v) + s2.d1u.crossed(s2.d2v),
        s2.d1v,
    );
    if first {
        surface_columns(d, du2, dv2)
    } else {
        curve_column(d, jet.v2d, du2, dv2)
    }
}

/// `D(2..4, 2)`. With a constant radius, the radii enter the coefficients ("it is assumed
/// that the radius is constant"); with a law, the unit-radius terms are scaled by `ray`
/// and the law derivative term `dray * temp` is added.
fn guide_column(jet: &Jet, t: &Terms, radius: InverseRadius) -> [f64; 3] {
    let (k1, k2) = match radius {
        InverseRadius::Constant { ray1, ray2 } => (ray1, ray2),
        InverseRadius::Evolutive { sg1, sg2, .. } => (sg1, sg2),
    };
    let (resul1, resul2) = guide_terms(jet, t, k1, k2);
    let InverseRadius::Evolutive {
        sg1,
        sg2,
        ray,
        dray,
    } = radius
    else {
        return [
            resul1.x() + resul2.x(),
            resul1.y() + resul2.y(),
            resul1.z() + resul2.z(),
        ];
    };
    let mut temp = Vector3::default();
    temp.set_linear_form_three(
        [
            sg1 * t.ndotns1 / t.norm1 - sg2 * t.ndotns2 / t.norm2,
            -sg1 / t.norm1,
            sg2 / t.norm2,
        ],
        [jet.guide.nplan, jet.ns1, jet.ns2],
    );
    [
        ray * (resul1.x() + resul2.x()) + dray * temp.x(),
        ray * (resul1.y() + resul2.y()) + dray * temp.y(),
        ray * (resul1.z() + resul2.z()) + dray * temp.z(),
    ]
}

/// The derivatives along the guide of both projected normals, scaled by `k1` and `k2`.
fn guide_terms(jet: &Jet, t: &Terms, k1: f64, k2: f64) -> (Vector3, Vector3) {
    let (nplan, dnplan) = (jet.guide.nplan, jet.guide.dnplan);
    let grosterme = t.ncrossns1.dot(dnplan.crossed(jet.ns1)) / t.norm1 / t.norm1;
    let mut resul1 = Vector3::default();
    resul1.set_linear_form_three(
        [
            -k1 / t.norm1 * (grosterme * t.ndotns1 - dnplan.dot(jet.ns1)),
            k1 * t.ndotns1 / t.norm1,
            k1 * grosterme / t.norm1,
        ],
        [nplan, dnplan, jet.ns1],
    );
    let grosterme = t.ncrossns2.dot(dnplan.crossed(jet.ns2)) / t.norm2 / t.norm2;
    let mut resul2 = Vector3::default();
    resul2.set_linear_form_three(
        [
            k2 / t.norm2 * (grosterme * t.ndotns2 - dnplan.dot(jet.ns2)),
            -k2 * t.ndotns2 / t.norm2,
            -k2 * grosterme / t.norm2,
        ],
        [nplan, dnplan, jet.ns2],
    );
    (resul1, resul2)
}

/// The projection terms of both normals: `np ^ ns`, `|np ^ ns|` (1 below `Eps`), `np.ns`.
struct Terms {
    ncrossns1: Vector3,
    ncrossns2: Vector3,
    norm1: f64,
    norm2: f64,
    ndotns1: f64,
    ndotns2: f64,
}

impl Terms {
    fn new(nplan: Vector3, ns1: Vector3, ns2: Vector3) -> Self {
        let (ncrossns1, ncrossns2) = (nplan.crossed(ns1), nplan.crossed(ns2));
        let norm = |v: Vector3| {
            if v.magnitude() < EPS {
                1.
            } else {
                v.magnitude()
            }
        };
        Self {
            ncrossns1,
            ncrossns2,
            norm1: norm(ncrossns1),
            norm2: norm(ncrossns2),
            ndotns1: nplan.dot(ns1),
            ndotns2: nplan.dot(ns2),
        }
    }
}

/// The derivative of the first-surface residual along a parameter of normal derivative
/// `temp` and point derivative `d1`.
fn surface_column(
    nplan: Vector3,
    ns: Vector3,
    (ncrossns, norm, ndotns): (Vector3, f64, f64),
    ray: f64,
    temp: Vector3,
    d1: Vector3,
) -> Vector3 {
    let grosterme = ncrossns.dot(nplan.crossed(temp)) / norm / norm;
    let mut resul = Vector3::default();
    resul.set_linear_form_three_offset(
        [
            -ray / norm * (grosterme * ndotns - nplan.dot(temp)),
            ray * grosterme / norm,
            -ray / norm,
        ],
        [nplan, ns, temp],
        d1,
    );
    resul
}

/// The derivative of the second-surface residual (negated radius terms, minus `d1`).
fn second_surface_column(
    nplan: Vector3,
    ns: Vector3,
    (ncrossns, norm, ndotns): (Vector3, f64, f64),
    ray: f64,
    temp: Vector3,
    d1: Vector3,
) -> Vector3 {
    let grosterme = ncrossns.dot(nplan.crossed(temp)) / norm / norm;
    let mut resul = Vector3::default();
    resul.set_linear_form_three(
        [
            ray / norm * (grosterme * ndotns - nplan.dot(temp)),
            -ray * grosterme / norm,
            ray / norm,
        ],
        [nplan, ns, temp],
    );
    resul -= d1;
    resul
}

/// `D(2..4, 1)`: the derivative along the curve on surface.
fn curve_column(d: &mut Matrix<'_>, v2d: Vector2, du: Vector3, dv: Vector3) -> Result<(), Failure> {
    mset(d, 2, 1, v2d.x() * du.x() + v2d.y() * dv.x())?;
    mset(d, 3, 1, v2d.x() * du.y() + v2d.y() * dv.y())?;
    mset(d, 4, 1, v2d.x() * du.z() + v2d.y() * dv.z())
}

/// `D(2..4, 3)` and `D(2..4, 4)`: the derivatives along the other surface.
fn surface_columns(d: &mut Matrix<'_>, du: Vector3, dv: Vector3) -> Result<(), Failure> {
    for (col, value) in [(3, du), (4, dv)] {
        mset(d, 2, col, value.x())?;
        mset(d, 3, col, value.y())?;
        mset(d, 4, col, value.z())?;
    }
    Ok(())
}
