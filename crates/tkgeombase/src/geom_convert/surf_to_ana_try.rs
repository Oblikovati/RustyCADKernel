// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_SurfToAnaSurf.cxx
// (TryCylinerCone, TryTorusSphere, ComputeGap) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder, cone, sphere and torus builders of `GeomConvert_SurfToAnaSurf`
//! from recognized isolines, and the gap of a candidate surface.
use super::{AnaCurveFit, ConvType, CurveToAnaCurve};
use crate::geom_lib::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface, SurfaceBounds,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::elslib::{self, ElementarySurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{
    Axis3, Circle3, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::f64::consts::PI;

// `ComputeCurve(curve, toler, p1, p2, ..., GeomConvert_Target, GeomAbs_Circle)`
// cast to a circle (null when not a circle).
fn target_circle(
    curve: &Handle<dyn GeomCurve>,
    toler: f64,
    range: (f64, f64),
) -> Result<Handle<GeomCircle>, Failure> {
    let mut fit = AnaCurveFit::new(0.0, 0.0);
    let found = CurveToAnaCurve::compute_curve(
        curve,
        toler,
        range,
        &mut fit,
        ConvType::Target,
        CurveType::Circle,
    )?;
    Ok(found.cast::<GeomCircle>())
}

// The gp circle of a live Geom_Circle handle.
fn circ_of(circle: &Handle<GeomCircle>) -> Result<Circle3, Failure> {
    live(circle, "GeomConvert_SurfToAnaSurf: Geom_Circle")?.circ()
}

// The radius and center of an iso circle, or 0 and the point at (Last - First) / 2
// of the isoline (not its middle, as natively) when it is not a circle.
fn radius_and_center(
    circle: &Handle<GeomCircle>,
    iso: &Handle<dyn GeomCurve>,
) -> Result<(f64, Point3), Failure> {
    if circle.is_some() {
        let circ: Circle3 = circ_of(circle)?;
        return Ok((circ.radius(), circ.location()));
    }
    let curve: &dyn GeomCurve = live(iso, "GeomConvert_SurfToAnaSurf::TryCylinerCone")?;
    Ok((
        0.0,
        curve.value((curve.last_parameter() - curve.first_parameter()) / 2.0)?,
    ))
}

/// `TryCylinerCone(theSurf, theVCase, theUmidiso, theVmidiso, theU1, theU2, theV1,
/// theV2, theToler)`: the cylinder (three equal radii) or the cone (monotonic radii)
/// through the circles of the first, middle and last isolines across the line
/// isoline; null otherwise.
pub(super) fn try_cyliner_cone(
    surf: &Handle<dyn GeomSurface>,
    v_case: bool,
    (umidiso, vmidiso): (&Handle<dyn GeomCurve>, &Handle<dyn GeomCurve>),
    (u1, u2, v1, v2): SurfaceBounds,
    toler: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface: &dyn GeomSurface = live(surf, "GeomConvert_SurfToAnaSurf::TryCylinerCone")?;
    let (params, first_iso, last_iso, mid_circ, line_iso) = if v_case {
        (
            (u1, u2),
            surface.v_iso(v1)?,
            surface.v_iso(v2)?,
            vmidiso,
            umidiso,
        )
    } else {
        (
            (v1, v2),
            surface.u_iso(u1)?,
            surface.u_iso(u2)?,
            umidiso,
            vmidiso,
        )
    };
    let isoline: Direction3 = live(
        &line_iso.cast::<GeomLine>(),
        "GeomConvert_SurfToAnaSurf::TryCylinerCone",
    )?
    .lin()
    .direction();
    let first_circ = target_circle(&first_iso, toler, params)?;
    let last_circ = target_circle(&last_iso, toler, params)?;
    if first_circ.is_null() && last_circ.is_null() {
        return Ok(Handle::default());
    }
    let (r1, p1) = radius_and_center(&first_circ, &first_iso)?;
    let mid: Circle3 = circ_of(&mid_circ.cast::<GeomCircle>())?;
    let r2: f64 = mid.radius();
    let (r3, p3) = radius_and_center(&last_circ, &last_iso)?;
    // cylinder
    if (r2 - r1).abs() < toler && (r3 - r1).abs() < toler && (r3 - r2).abs() < toler {
        let axes = Axis3::from_normal(p1, Direction3::from_vector(Vector3::between(p1, p3))?)?;
        return Ok(Handle::new(GeomCylindricalSurface::new(axes, r1)?).cast());
    }
    // cone
    if (r1.abs() > r2.abs() && r2.abs() > r3.abs()) || (r3.abs() > r2.abs() && r2.abs() > r1.abs())
    {
        return cone_through(isoline, (r1, p1), (r3, p3));
    }
    Ok(Handle::default())
}

// The cone from the smaller end circle towards the other, of semi-angle the
// (acute) angle of the line isoline with the axis.
fn cone_through(
    isoline: Direction3,
    (r1, p1): (f64, Point3),
    (r3, p3): (f64, Point3),
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut semiangle: f64 = Vector3::from(isoline).angle(Vector3::between(p3, p1))?;
    if semiangle > PI / 2.0 {
        semiangle = PI - semiangle;
    }
    let (radius, axes): (f64, Axis3) = if r1 > r3 {
        (
            r3,
            Axis3::from_normal(p3, Direction3::from_vector(Vector3::between(p3, p1))?)?,
        )
    } else {
        (
            r1,
            Axis3::from_normal(p1, Direction3::from_vector(Vector3::between(p1, p3))?)?,
        )
    };
    Ok(Handle::new(GeomConicalSurface::new(axes, semiangle, radius)?).cast())
}

/// `TryTorusSphere(theSurf, circle, otherCircle, Param1, Param2, aParam1ToCrv,
/// aParam2ToCrv, toler, isTryUMajor)`: the sphere or torus whose minor circles are
/// `circle` and the isolines at 1/3 and 2/3 of [Param1, Param2] (V isolines when
/// `is_try_u_major`) of the same radius; null otherwise.
pub(super) fn try_torus_sphere(
    surf: &Handle<dyn GeomSurface>,
    (circle, other_circle): (&Handle<GeomCircle>, &Handle<GeomCircle>),
    (param1, param2): (f64, f64),
    to_crv: (f64, f64),
    toler: f64,
    is_try_u_major: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface: &dyn GeomSurface = live(surf, "GeomConvert_SurfToAnaSurf::TryTorusSphere")?;
    let minor: Circle3 = circ_of(circle)?;
    let r: f64 = minor.radius();
    let (at1, at2): (f64, f64) = (
        param1 + ((param2 - param1) / 3.0),
        param1 + ((param2 - param1) * 2.0 / 3.0),
    );
    let (iso1, iso2) = if is_try_u_major {
        (surface.v_iso(at1)?, surface.v_iso(at2)?)
    } else {
        (surface.u_iso(at1)?, surface.u_iso(at2)?)
    };
    let circle1 = target_circle(&iso1, toler, to_crv)?;
    let circle2 = target_circle(&iso2, toler, to_crv)?;
    if circle1.is_null() || circle2.is_null() {
        return Ok(Handle::default());
    }
    let (c1, c2): (Circle3, Circle3) = (circ_of(&circle1)?, circ_of(&circle2)?);
    // Check the radii.
    if (r - c1.radius()).abs() > toler || (r - c2.radius()).abs() > toler {
        return Ok(Handle::default());
    }
    // The centers of the major radius.
    let (pnt1, pnt2, pnt3): (Point3, Point3, Point3) =
        (minor.location(), c1.location(), c2.location());
    if pnt1.distance(&pnt2) < toler || pnt1.distance(&pnt3) < toler {
        // compute sphere
        let main_dir: Direction3 = circ_of(other_circle)?.axis().direction();
        let axes = Axis3::from_normal(minor.location(), main_dir)?;
        return Ok(Handle::new(GeomSphericalSurface::new(axes, r)?).cast());
    }
    let mut circ = Circle3::default();
    if !CurveToAnaCurve::get_circle(&mut circ, pnt1, pnt2, pnt3)? {
        return Ok(Handle::default());
    }
    let center: Point3 = circ.location();
    let dir = Direction3::from_xyz((pnt1.xyz() - center.xyz()).crossed(pnt3.xyz() - center.xyz()))?;
    let ax3 = Axis3::from_normal(center, dir)?;
    Ok(Handle::new(GeomToroidalSurface::new(ax3, circ.radius(), r)?).cast())
}

/// The elementary surface of a candidate, for the projections of ComputeGap.
enum GapQuadric {
    Plane(Plane3),
    Cylinder(Cylinder3),
    Cone(Cone3),
    Sphere(Sphere3),
    Torus(Torus3),
}

impl GapQuadric {
    // The quadric of an elementary adaptor type, None for another type.
    fn of(adaptor: &GeomAdaptorSurface) -> Result<Option<Self>, Failure> {
        Ok(Some(match adaptor.get_type()? {
            SurfaceType::Plane => Self::Plane(adaptor.plane()?),
            SurfaceType::Cylinder => Self::Cylinder(adaptor.cylinder()?),
            SurfaceType::Cone => Self::Cone(adaptor.cone()?),
            SurfaceType::Sphere => Self::Sphere(adaptor.sphere()?),
            SurfaceType::Torus => Self::Torus(adaptor.torus()?),
            _ => return Ok(None),
        }))
    }

    // ElSLib::Value(ElSLib::Parameters(point)).
    fn projection(&self, point: Point3) -> Point3 {
        match self {
            Self::Plane(plane) => reprojected(plane, point),
            Self::Cylinder(cylinder) => reprojected(cylinder, point),
            Self::Cone(cone) => reprojected(cone, point),
            Self::Sphere(sphere) => reprojected(sphere, point),
            Self::Torus(torus) => reprojected(torus, point),
        }
    }
}

// The point of `surface` at the parameters of `point`.
fn reprojected<S: ElementarySurface>(surface: &S, point: Point3) -> Point3 {
    let (s, t): (f64, f64) = elslib::parameters(surface, point);
    elslib::value(s, t, surface)
}

/// `ComputeGap(theSurf, theU1, theU2, theV1, theV2, theNewSurf, theTol =
/// RealLast())`: the greatest distance of the 20 x 20 cell centers of `surf` to
/// their reprojection on the elementary `new_surf`, stopping at the first row
/// beyond `tol`; Precision::Infinite() for a non-elementary `new_surf`.
pub(super) fn compute_gap(
    surf: &Handle<dyn GeomSurface>,
    (u1, u2, v1, v2): SurfaceBounds,
    new_surf: &Handle<dyn GeomSurface>,
    tol: f64,
) -> Result<f64, Failure> {
    let adaptor = GeomAdaptorSurface::with_surface(new_surf)?;
    let Some(quadric) = GapQuadric::of(&adaptor)? else {
        return Ok(precision::infinite());
    };
    let surface: &dyn GeomSurface = live(surf, "GeomConvert_SurfToAnaSurf::ComputeGap")?;
    let np: i32 = 21;
    let du: f64 = (u2 - u1) / f64::from(np - 1);
    let dv: f64 = (v2 - v1) / f64::from(np - 1);
    let (du2, dv2): (f64, f64) = (du / 2.0, dv / 2.0);
    let mut gap: f64 = 0.0;
    for j in 1..np {
        let v: f64 = v1 + dv * f64::from(j - 1) + dv2;
        for i in 1..np {
            let u: f64 = u1 + du * f64::from(i - 1) + du2;
            let point: Point3 = surface.d0(u, v)?;
            let dis: f64 = point.distance(&quadric.projection(point));
            if dis > gap {
                gap = dis;
            }
            if gap > tol {
                return Ok(gap);
            }
        }
    }
    Ok(gap)
}
