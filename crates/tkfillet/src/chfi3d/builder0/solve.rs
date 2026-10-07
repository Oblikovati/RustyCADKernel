// Rust translation of ChFi3d_IntCS, ChFi3d_ComputesIntPC, ChFi3d_SearchPivot,
// ChFi3d_Parameters and ChFi3d_TrimCurve (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points the corner builders solve for: a curve through a surface, two fillet
//! pcurves meeting, the pivot of three fillets, and the parameters of a 3d point.
use super::bounds::in_period;
use crate::chfi_ds::FaceInterference;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_api::{ProjectPointOnCurve, ProjectPointOnSurf};
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkgeombase::extrema::{ExtAlgo, LocateExtCC};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::{elclib, elslib};

/// The parameters of a point of the surface brought into its periods from its first
/// parameters.
fn in_periods(
    s: &Handle<dyn Adaptor3dSurface>,
    (u1, v1): (f64, f64),
    (up, vp): (f64, f64),
) -> Result<(f64, f64), Failure> {
    let up = if s.is_u_periodic()? {
        in_period(up, u1, u1 + s.u_period()?, 1.0e-8)
    } else {
        up
    };
    let vp = if s.is_v_periodic()? {
        in_period(vp, v1, v1 + s.v_period()?, 1.0e-8)
    } else {
        vp
    };
    Ok((up, vp))
}

/// `ChFi3d_IntCS(S, C, p2dS, wc)`: the cut of the curve by the surface within both
/// domains: the one with the lowest parameter on the curve when `wc < -1e100`, the
/// highest when `wc > 1e100`, else the nearest to `wc`; `(p2dS, wc)` or `None`;
/// `int_cs(&s, &c, 0.5)?`.
pub fn int_cs(
    s: &Handle<dyn Adaptor3dSurface>,
    c: &Handle<dyn Adaptor3dCurve>,
    wc: f64,
) -> Result<Option<(Point2, f64)>, Failure> {
    let (uf, ul) = (c.first_parameter()?, c.last_parameter()?);
    let (u1, u2) = (s.first_u_parameter()?, s.last_u_parameter()?);
    let (v1, v2) = (s.first_v_parameter()?, s.last_v_parameter()?);
    let mut intersection = HInter::new();
    intersection.perform(c, s)?;
    let (keepfirst, keeplast) = (wc < -1.0e100, wc > 1.0e100);
    let mut temp = if keeplast {
        -1.0e100
    } else if keepfirst {
        1.0e100
    } else {
        0.0
    };
    let mut dist = 2.0e100;
    if !intersection.is_done() {
        return Ok(None);
    }
    let mut isol = 0;
    for i in 1..=intersection.nb_points()? {
        let pint = intersection.point(i)?;
        let (up, vp) = in_periods(s, (u1, v1), (pint.u(), pint.v()))?;
        let w = pint.w();
        if !(uf <= w && ul >= w && u1 <= up && u2 >= up && v1 <= vp && v2 >= vp) {
            continue;
        }
        if (keepfirst && w < temp) || (keeplast && w > temp) {
            (temp, isol) = (w, i);
        } else if (w - wc).abs() < dist {
            (dist, isol) = ((w - wc).abs(), i);
        }
    }
    if isol == 0 {
        return Ok(None);
    }
    let pint = intersection.point(isol)?;
    let (up, vp) = in_periods(s, (u1, v1), (pint.u(), pint.v()))?;
    Ok(Some((Point2::new(up, vp), pint.w())))
}

/// The pcurve on the fillet surface of the interference restricted around `u` (by
/// `min(0.1, 5%)` of its range) as a curve on the surface.
fn local_curve(
    fi: &FaceInterference,
    hs: &Handle<GeomAdaptorSurface>,
    u: f64,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    // std::min(0.1, 5%): 0.1 when the share is NaN.
    let share = 0.05 * (fi.last_parameter() - fi.first_parameter());
    let delt = if share < 0.1 { share } else { 0.1 };
    let hc2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
        fi.pcurve_on_surf(),
        u - delt,
        u + delt,
    )?)
    .cast();
    let surface: Handle<dyn Adaptor3dSurface> = hs.clone().cast();
    Adaptor3dCurveOnSurface::with_curve(&hc2d, &surface)
}

/// `ChFi3d_ComputesIntPC(Fi1, Fi2, HS1, HS2, UInt1, UInt2, P)`: the parameters of the
/// meeting of the pcurves of two interferences on their fillet surfaces, refined from
/// `(UInt1, UInt2)` by a local extremum when it is nearer, and the middle point;
/// `computes_int_pc((&fi1, &hs1), (&fi2, &hs2), (0.5, 0.5))?`.
pub fn computes_int_pc(
    (fi1, hs1): (&FaceInterference, &Handle<GeomAdaptorSurface>),
    (fi2, hs2): (&FaceInterference, &Handle<GeomAdaptorSurface>),
    (uint1, uint2): (f64, f64),
) -> Result<(f64, f64, Point3), Failure> {
    let uv = fi1.pcurve_on_surf().value(uint1)?;
    let p3d1 = hs1.value(uv.x(), uv.y())?;
    let uv = fi2.pcurve_on_surf().value(uint2)?;
    let p3d2 = hs2.value(uv.x(), uv.y())?;
    let distref2 = p3d1.square_distance(&p3d2);
    let mut result = (uint1, uint2, Point3::from((p3d1.xyz() + p3d2.xyz()) * 0.5));
    let (cons1, cons2) = (local_curve(fi1, hs1, uint1)?, local_curve(fi2, hs2, uint2)?);
    let ext = LocateExtCC::new(&cons1, &cons2, uint1, uint2)?;
    if ext.is_done() && ext.square_distance()? < distref2 {
        let (ponc1, ponc2) = ext.point()?;
        result = (
            ponc1.parameter(),
            ponc2.parameter(),
            Point3::from((ponc1.value().xyz() + ponc2.value().xyz()) * 0.5),
        );
    }
    Ok(result)
}

/// `ChFi3d_SearchPivot(s, u, t)`: the index (0 to 2) of the fillet whose two neighbours
/// do not cross on the opposite face (by their senses `s` and the parameters `u` of
/// their sections, within `t`), -1 when none;
/// `search_pivot(&[1, 1, 1], &[[0.0; 3]; 3], 1e-9)`.
///
/// ```
/// use rustycad_tkfillet::chfi3d::search_pivot;
/// assert_eq!(search_pivot(&[1, 1, 1], &[[0.0; 3]; 3], 1e-9), 0);
/// ```
pub fn search_pivot(s: &[i32; 3], u: &[[f64; 3]; 3], t: f64) -> i32 {
    let good = |sense: i32, delta: f64| if sense == 1 { delta >= -t } else { delta <= t };
    for i in 0..3 {
        let (n1, n2) = ((i + 1) % 3, (i + 2) % 3);
        let bondeb = good(s[n1], u[n1][i] - u[n1][n2]);
        let bonfin = good(s[n2], u[n2][i] - u[n2][n1]);
        if bondeb && bonfin {
            return i as i32;
        }
    }
    -1
}

/// `ChFi3d_Parameters(S, p3d, u, v)`: the parameters of the point on an elementary
/// surface, else of its single projection (StdFail_NotDone otherwise);
/// `parameters(&s, p)?`.
pub fn parameters(s: &Handle<dyn GeomSurface>, p3d: Point3) -> Result<(f64, f64), Failure> {
    let gas = GeomAdaptorSurface::with_surface(s)?;
    Ok(match gas.get_type()? {
        SurfaceType::Plane => elslib::plane_parameters(gas.plane()?.position(), p3d),
        SurfaceType::Cylinder => {
            let cyl = gas.cylinder()?;
            elslib::cylinder_parameters(cyl.position(), cyl.radius(), p3d)
        }
        SurfaceType::Cone => {
            let cone = gas.cone()?;
            elslib::cone_parameters(cone.position(), cone.ref_radius(), cone.semi_angle(), p3d)
        }
        SurfaceType::Sphere => {
            let sphere = gas.sphere()?;
            elslib::sphere_parameters(sphere.position(), sphere.radius(), p3d)
        }
        SurfaceType::Torus => {
            let torus = gas.torus()?;
            elslib::torus_parameters(
                torus.position(),
                torus.major_radius(),
                torus.minor_radius(),
                p3d,
            )
        }
        _ => {
            let tool = ProjectPointOnSurf::with_surface(p3d, s, ExtAlgo::Grad)?;
            if tool.nb_points()? != 1 {
                return Err(Failure::with_message(
                    "StdFail_NotDone",
                    "ChFi3d_Parameters() - no projection results",
                ));
            }
            tool.parameters(1)?
        }
    })
}

/// The single projection of a point on the curve (StdFail_NotDone otherwise).
fn projected_parameter(p: Point3, gc: &Handle<dyn GeomCurve>, which: &str) -> Result<f64, Failure> {
    let tool = ProjectPointOnCurve::with_curve(p, gc)?;
    if tool.nb_points() != 1 {
        return Err(Failure::with_message(
            "StdFail_NotDone",
            &format!("ChFi3d_TrimCurve() - no projection results for the {which} point"),
        ));
    }
    tool.parameter(1)
}

/// The parameter of a point on a conic.
fn conic_parameter(gac: &GeomAdaptorCurve, p: Point3) -> Result<Option<f64>, Failure> {
    Ok(Some(match gac.get_type()? {
        CurveType::Line => elclib::line_parameter(gac.line()?.position(), p),
        CurveType::Circle => elclib::circle_parameter(gac.circle()?.position(), p),
        CurveType::Ellipse => {
            let e = gac.ellipse()?;
            elclib::ellipse_parameter(e.position(), e.major_radius(), e.minor_radius(), p)?
        }
        CurveType::Hyperbola => {
            let h = gac.hyperbola()?;
            elclib::hyperbola_parameter(h.position(), h.major_radius(), h.minor_radius(), p)
        }
        CurveType::Parabola => elclib::parabola_parameter(gac.parabola()?.position(), p),
        _ => return Ok(None),
    }))
}

/// `ChFi3d_TrimCurve(gc, FirstP, LastP, gtc)`: the curve trimmed between the parameters
/// of the two points (on a conic, else by their single projections);
/// `trim_curve(&c, p1, p2)?`.
pub fn trim_curve(
    gc: &Handle<dyn GeomCurve>,
    first_p: Point3,
    last_p: Point3,
) -> Result<Handle<GeomTrimmedCurve>, Failure> {
    let gac = GeomAdaptorCurve::with_curve(gc)?;
    let (uf, ul) = match (
        conic_parameter(&gac, first_p)?,
        conic_parameter(&gac, last_p)?,
    ) {
        (Some(uf), Some(ul)) => (uf, ul),
        _ => (
            projected_parameter(first_p, gc, "first")?,
            projected_parameter(last_p, gc, "second")?,
        ),
    };
    Ok(Handle::new(GeomTrimmedCurve::new(gc, uf, ul, true, true)?))
}
