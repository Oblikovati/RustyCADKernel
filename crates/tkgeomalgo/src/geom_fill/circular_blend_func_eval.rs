// Rust translation of the Approx_SweepFunction members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CircularBlendFunc.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluators and the queries of `GeomFill_CircularBlendFunc`: the arc between
//! the rail points, centred where the two unit vectors from the rails to the path
//! point meet at the radius, in the plane of those vectors (or normal to the path
//! when they are parallel).
use super::arc_section::{ArcPolesD1, ArcPolesD2};
use super::circular_blend_func::{GeomFillCircularBlendFunc, TrimmedCurves, fus_int, next_shape};
use super::frame_math::std_min;
use super::geomfill::{get_minimal_weights, get_tolerance, knots, mults};
use super::geomfill_circle::{
    CircleD0, CircleD1, CircleD2, get_circle, get_circle_d1, get_circle_d2,
};
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2, SweepFunction};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector3, Xyz};

/// The angular tolerance of parallel rail vectors (`TolAng`).
const TOL_ANG: f64 = 1.0e-6;

/// A unit vector and its first derivative from a vector and its derivative.
fn unit_d1(v: Vector3, dv: Vector3) -> (Vector3, Vector3) {
    let inv = 1.0 / v.magnitude();
    let mut n = v;
    n *= inv;
    let mut dn = Vector3::default();
    dn.set_scaled_sum(-dv.dot(n), n, dv);
    dn *= inv;
    (n, dn)
}

/// A unit vector and its first two derivatives.
fn unit_d2(v: Vector3, dv: Vector3, d2v: Vector3) -> (Vector3, Vector3, Vector3) {
    let inv = 1.0 / v.magnitude();
    let mut n = v;
    n *= inv;
    let sc = dv.dot(n);
    let mut d2n = Vector3::default();
    let a = 3.0 * sc * sc * inv - d2v.dot(n) - inv * dv.square_magnitude();
    d2n.set_linear_form_offset(a, n, -2.0 * sc * inv, dv, d2v);
    let mut dn = Vector3::default();
    dn.set_scaled_sum(-sc, n, dv);
    d2n *= inv;
    dn *= inv;
    (n, dn, d2n)
}

/// `0.5 (r a + r b + c + d)`: the arc centre (or a derivative of it).
fn half_form(r: f64, a: Xyz, b: Xyz, c: Xyz, d: Xyz) -> Xyz {
    let mut temp = Xyz::default();
    temp.set_linear_form_three_offset([r, r, 1.0], [a, b, c], d);
    temp * 0.5
}

/// The section circle of the unit rail vectors, the plane normal and the rail points.
fn circle_d0(r: f64, ns: (Vector3, Vector3), nplan: Vector3, pts: (Point3, Point3)) -> CircleD0 {
    let center = half_form(r, *ns.0.xyz(), *ns.1.xyz(), pts.0.xyz(), pts.1.xyz());
    CircleD0 {
        ns1: ns.0,
        ns2: ns.1,
        nplan,
        pts1: pts.0,
        pts2: pts.1,
        rayon: r,
        center: Point3::from(center),
    }
}

/// `circle` with the first derivatives (the radius is constant).
fn circle_d1(
    circle: CircleD0,
    dns: (Vector3, Vector3),
    dnplan: Vector3,
    dp: (Vector3, Vector3),
) -> CircleD1 {
    let d_center = half_form(
        circle.rayon,
        *dns.0.xyz(),
        *dns.1.xyz(),
        *dp.0.xyz(),
        *dp.1.xyz(),
    );
    CircleD1 {
        circle,
        dn1w: dns.0,
        dn2w: dns.1,
        dnplan,
        tang1: dp.0,
        tang2: dp.1,
        d_rayon: 0.0,
        d_center: Vector3::from(d_center),
    }
}

/// `d1` with the second derivatives.
fn circle_d2(
    d1: CircleD1,
    d2ns: (Vector3, Vector3),
    d2nplan: Vector3,
    d2p: (Vector3, Vector3),
) -> CircleD2 {
    let r = d1.circle.rayon;
    let d2_center = half_form(r, *d2ns.0.xyz(), *d2ns.1.xyz(), *d2p.0.xyz(), *d2p.1.xyz());
    CircleD2 {
        d1,
        d2n1w: d2ns.0,
        d2n2w: d2ns.1,
        d2nplan,
        dtang1: d2p.0,
        dtang2: d2p.1,
        d2_rayon: 0.0,
        d2_center: Vector3::from(d2_center),
    }
}

fn rail<'a>(
    h: &'a Handle<dyn Adaptor3dCurve>,
    name: &str,
) -> Result<&'a dyn Adaptor3dCurve, Failure> {
    live(h, "GeomFill_CircularBlendFunc", name)
}

impl GeomFillCircularBlendFunc {
    fn trimmed_curves(&self) -> TrimmedCurves {
        self.trimmed.read().clone()
    }

    /// `nplan` from the path derivatives (reversed with `myreverse`).
    fn path_normal(&self, v: Vector3) -> Vector3 {
        if self.myreverse { -v } else { v }
    }

    fn d0_circle(&self, param: f64) -> Result<CircleD0, Failure> {
        let t = self.trimmed_curves();
        let center = rail(&t.path, "myTPath")?.d0(param)?;
        let p1 = rail(&t.curve1, "myTCurve1")?.d0(param)?;
        let p2 = rail(&t.curve2, "myTCurve2")?.d0(param)?;
        let mut ns1 = Vector3::from(center.xyz() - p1.xyz());
        let mut ns2 = Vector3::from(center.xyz() - p2.xyz());
        let mut nplan = if !ns1.is_parallel(ns2, TOL_ANG)? {
            ns1.crossed(ns2)
        } else {
            self.path_normal(rail(&t.path, "myTPath")?.d1(param)?.1)
        };
        ns1.normalize()?;
        ns2.normalize()?;
        nplan.normalize()?;
        Ok(circle_d0(self.my_radius, (ns1, ns2), nplan, (p1, p2)))
    }

    /// `nplan` and its derivative: the cross product of the rail vectors, or the path
    /// derivatives when they are parallel.
    fn d1_plane(
        &self,
        t: &TrimmedCurves,
        param: f64,
        ns: [(Vector3, Vector3); 2],
    ) -> Result<(Vector3, Vector3), Failure> {
        let [(ns1, dns1), (ns2, dns2)] = ns;
        if !ns1.is_parallel(ns2, TOL_ANG)? {
            return Ok((ns1.crossed(ns2), dns1.crossed(ns2) + ns1.crossed(dns2)));
        }
        let (_, v1, v2) = rail(&t.path, "myTPath")?.d2(param)?;
        Ok((self.path_normal(v1), self.path_normal(v2)))
    }

    fn d1_circle(&self, param: f64) -> Result<CircleD1, Failure> {
        let t = self.trimmed_curves();
        let (center, d_center) = rail(&t.path, "myTPath")?.d1(param)?;
        let (p1, dp1) = rail(&t.curve1, "myTCurve1")?.d1(param)?;
        let (p2, dp2) = rail(&t.curve2, "myTCurve2")?.d1(param)?;
        let ns1 = Vector3::from(center.xyz() - p1.xyz());
        let ns2 = Vector3::from(center.xyz() - p2.xyz());
        let (dns1, dns2) = (d_center - dp1, d_center - dp2);
        let (nplan, dnplan) = self.d1_plane(&t, param, [(ns1, dns1), (ns2, dns2)])?;
        let ((ns1, dns1), (ns2, dns2)) = (unit_d1(ns1, dns1), unit_d1(ns2, dns2));
        let (nplan, dnplan) = unit_d1(nplan, dnplan);
        let c = circle_d0(self.my_radius, (ns1, ns2), nplan, (p1, p2));
        Ok(circle_d1(c, (dns1, dns2), dnplan, (dp1, dp2)))
    }

    /// `nplan` and its two derivatives (see `d1_plane`).
    fn d2_plane(
        &self,
        t: &TrimmedCurves,
        param: f64,
        ns: [(Vector3, Vector3, Vector3); 2],
    ) -> Result<(Vector3, Vector3, Vector3), Failure> {
        let [(ns1, dns1, d2ns1), (ns2, dns2, d2ns2)] = ns;
        if !ns1.is_parallel(ns2, TOL_ANG)? {
            let mut d2nplan = Vector3::default();
            d2nplan.set_linear_form_offset(
                1.0,
                d2ns1.crossed(ns2),
                2.0,
                dns1.crossed(dns2),
                ns1.crossed(d2ns2),
            );
            return Ok((
                ns1.crossed(ns2),
                dns1.crossed(ns2) + ns1.crossed(dns2),
                d2nplan,
            ));
        }
        let (_, v1, v2, v3) = rail(&t.path, "myTPath")?.d3(param)?;
        Ok((
            self.path_normal(v1),
            self.path_normal(v2),
            self.path_normal(v3),
        ))
    }

    fn d2_circle(&self, param: f64) -> Result<CircleD2, Failure> {
        let t = self.trimmed_curves();
        let (center, d_center, d2_center) = rail(&t.path, "myTPath")?.d2(param)?;
        let (p1, dp1, d2p1) = rail(&t.curve1, "myTCurve1")?.d2(param)?;
        let (p2, dp2, d2p2) = rail(&t.curve2, "myTCurve2")?.d2(param)?;
        let ns1 = (
            Vector3::from(center.xyz() - p1.xyz()),
            d_center - dp1,
            d2_center - d2p1,
        );
        let ns2 = (
            Vector3::from(center.xyz() - p2.xyz()),
            d_center - dp2,
            d2_center - d2p2,
        );
        let (nplan, dnplan, d2nplan) = self.d2_plane(&t, param, [ns1, ns2])?;
        let (ns1, dns1, d2ns1) = unit_d2(ns1.0, ns1.1, ns1.2);
        let (ns2, dns2, d2ns2) = unit_d2(ns2.0, ns2.1, ns2.2);
        let (nplan, dnplan, d2nplan) = unit_d2(nplan, dnplan, d2nplan);
        let c = circle_d0(self.my_radius, (ns1, ns2), nplan, (p1, p2));
        let c = circle_d1(c, (dns1, dns2), dnplan, (dp1, dp2));
        Ok(circle_d2(c, (d2ns1, d2ns2), d2nplan, (d2p1, d2p2)))
    }

    /// The fused intervals of the rails (at `S`) and of the path (one order higher).
    fn fused_intervals(&self, s: Shape) -> Result<Vec<f64>, Failure> {
        let path = rail(&self.my_path, "myPath")?;
        let (c1, c2) = (
            rail(&self.my_curve1, "myCurve1")?,
            rail(&self.my_curve2, "myCurve2")?,
        );
        let bounds = |c: &dyn Adaptor3dCurve, shape: Shape| -> Result<Vec<f64>, Failure> {
            let mut t: Array1<'static, f64> = one_based(c.nb_intervals(shape)? + 1)?;
            c.intervals(&mut t, shape)?;
            Ok(t.as_slice().to_vec())
        };
        let i_center = bounds(path, next_shape(s))?;
        let i_cbs = fus_int(&bounds(c1, s)?, &bounds(c2, s)?);
        Ok(fus_int(&i_center, &i_cbs))
    }
}

impl SweepFunction for GeomFillCircularBlendFunc {
    /// `D0`: the arc at `param` (always true).
    fn d0(
        &self,
        param: f64,
        _first: f64,
        _last: f64,
        poles: &mut Array1<'_, Point3>,
        _poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let c = self.d0_circle(param)?;
        get_circle(self.my_t_conv, &c, poles, weights)?;
        Ok(true)
    }

    fn d1(&self, param: f64, _first: f64, _last: f64, v: SweepD1<'_, '_>) -> Result<bool, Failure> {
        let c = self.d1_circle(param)?;
        let out = ArcPolesD1 {
            poles: v.poles,
            d_poles: v.d_poles,
            weights: v.weights,
            d_weights: v.d_weights,
        };
        get_circle_d1(self.my_t_conv, &c, out)?;
        Ok(true)
    }

    fn d2(&self, param: f64, _first: f64, _last: f64, v: SweepD2<'_, '_>) -> Result<bool, Failure> {
        let c = self.d2_circle(param)?;
        let out = ArcPolesD2 {
            d1: ArcPolesD1 {
                poles: v.d1.poles,
                d_poles: v.d1.d_poles,
                weights: v.d1.weights,
                d_weights: v.d1.d_weights,
            },
            d2_poles: v.d2_poles,
            d2_weights: v.d2_weights,
        };
        get_circle_d2(self.my_t_conv, &c, out)?;
        Ok(true)
    }

    fn nb_2d_curves(&self) -> Result<i32, Failure> {
        Ok(0)
    }

    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok((self.my_nb_poles, self.my_nb_knots, self.my_degree))
    }

    fn knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        knots(self.my_t_conv, t_knots)
    }

    fn mults(&self, t_mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        mults(self.my_t_conv, t_mults)
    }

    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(self.my_t_conv != ParameterisationType::Polynomial)
    }

    fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
        Ok(self.fused_intervals(s)?.len() as i32 - 1)
    }

    /// `Intervals(T, S)`: the fused bounds from `T(1)` on.
    fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        for (ii, value) in self.fused_intervals(s)?.into_iter().enumerate() {
            put(t, ii as i32 + 1, value)?;
        }
        Ok(())
    }

    /// `SetInterval(First, Last)`: the evaluators read the curves trimmed to it.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        let eps = precision::p_confusion();
        let trimmed = TrimmedCurves {
            path: rail(&self.my_path, "myPath")?.trim(first, last, eps)?,
            curve1: rail(&self.my_curve1, "myCurve1")?.trim(first, last, eps)?,
            curve2: rail(&self.my_curve2, "myCurve2")?.trim(first, last, eps)?,
        };
        *self.trimmed.write() = trimmed;
        Ok(())
    }

    /// `GetTolerance`: `SurfTol` inside, the arc tolerance next to the ends and
    /// `BoundTol` at the ends when smaller.
    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (low, up) = (tol3d.lower(), tol3d.upper());
        let tol = get_tolerance(
            self.my_t_conv,
            self.minang,
            self.my_radius,
            angle_tol,
            surf_tol,
        )?;
        tol3d.init(surf_tol);
        let (inner, outer) = (std_min(tol, surf_tol), std_min(tol, bound_tol));
        for (index, value) in [(up - 1, inner), (low + 1, inner), (up, outer), (low, outer)] {
            put(tol3d, index, value).map_err(|e| {
                sweep_failure(
                    "Standard_OutOfRange",
                    &format!("GeomFill_CircularBlendFunc::GetTolerance: {e}"),
                )
            })?;
        }
        Ok(())
    }

    /// `SetTolerance`: nothing to do.
    fn set_tolerance(&self, _tol3d: f64, _tol2d: f64) -> Result<(), Failure> {
        Ok(())
    }

    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        Ok(self.my_bary)
    }

    fn maximal_section(&self) -> Result<f64, Failure> {
        Ok(self.maxang * self.my_radius)
    }

    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        get_minimal_weights(self.my_t_conv, self.minang, self.maxang, weights)
    }
}
