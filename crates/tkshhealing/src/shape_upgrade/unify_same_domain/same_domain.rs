//! `SameSurf` and `IsSameDomain` of ShapeUpgrade_UnifySameDomain.cxx: whether two faces
//! lie on the same surface.
use super::DataMapOfFacePlane;
use super::geometry::{clear_rts, get_cylinder, null_surface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomElementarySurface, GeomPlane, GeomSurface, GeomSweptSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::IntPatchImpImpIntersection;
use rustycad_tkgeombase::geom_lib::IsPlanarSurface;
use rustycad_tkmath::gp::{Cylinder3, Vector3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;

/// One bound of `SameSurf`: both infinite (the first replaced by `finite`) or both
/// finite within Precision::PConfusion.
fn same_bound(first: &mut f64, second: f64, infinite: fn(f64) -> bool, finite: f64) -> bool {
    if infinite(*first) {
        if !infinite(second) {
            return false;
        }
        *first = finite;
        return true;
    }
    !infinite(second) && (*first - second).abs() <= precision::p_confusion()
}

/// `SameSurf`: equal bounds (the infinite ones replaced by a unit range) and equal points
/// at the 4 Gauss-like parameters 0.3399811 and 0.7745966 of each range.
pub(super) fn same_surf(s1: &dyn GeomSurface, s2: &dyn GeomSurface) -> Result<bool, Failure> {
    let (mut uf1, mut ul1, mut vf1, mut vl1) = s1.bounds();
    let (uf2, ul2, vf2, vl2) = s2.bounds();
    let negative = precision::is_negative_infinite;
    let positive = precision::is_positive_infinite;
    let u_first = (-1.0_f64).min(ul1 - 1.);
    if !same_bound(&mut uf1, uf2, negative, u_first) {
        return Ok(false);
    }
    let v_first = (-1.0_f64).min(vl1 - 1.);
    if !same_bound(&mut vf1, vf2, negative, v_first) {
        return Ok(false);
    }
    if !same_bound(&mut ul1, ul2, positive, 1.0_f64.max(uf1 + 1.)) {
        return Ok(false);
    }
    if !same_bound(&mut vl1, vl2, positive, 1.0_f64.max(vf1 + 1.)) {
        return Ok(false);
    }
    same_sample_points(s1, s2, (uf1, ul1 - uf1), (vf1, vl1 - vf1))
}

fn same_sample_points(
    s1: &dyn GeomSurface,
    s2: &dyn GeomSurface,
    (uf, du): (f64, f64),
    (vf, dv): (f64, f64),
) -> Result<bool, Failure> {
    const COEFS: [f64; 2] = [0.3399811, 0.7745966];
    for cu in COEFS {
        let u = uf + cu * du;
        for cv in COEFS {
            let v = vf + cv * dv;
            if !s1
                .value(u, v)?
                .is_equal(&s2.value(u, v)?, precision::p_confusion())
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// `IsSameDomain(aFace, aCheckedFace, theLinTol, theAngTol, theFacePlaneMap)`: the same
/// located surface; or two planar surfaces (GeomLib_IsPlanarSurface) on parallel planes
/// closer than `lin_tol`, both bound to one plane in the map; or two elementary surfaces
/// whose IntPatch_ImpImpIntersection finds tangent faces; or two cylinders (cylindrical
/// or swept surfaces) with the same radius and axis.
pub(super) fn is_same_domain(
    face: &Face,
    checked: &Face,
    (lin_tol, ang_tol): (f64, f64),
    planes: &mut DataMapOfFacePlane,
) -> Result<bool, Failure> {
    let (s1, l1) = BRepTool::surface_with_location(face)?;
    let (s2, l2) = BRepTool::surface_with_location(checked)?;
    if s1 == s2 && l1 == l2 {
        return Ok(true);
    }
    let s1 = clear_rts(&BRepTool::surface(face)?);
    let s2 = clear_rts(&BRepTool::surface(checked)?);
    if same_plane(face, checked, (&s1, &s2), (lin_tol, ang_tol), planes)? {
        return Ok(true);
    }
    let elementary1 = !s1.cast::<dyn GeomElementarySurface>().is_null();
    if elementary1 && !s2.cast::<dyn GeomElementarySurface>().is_null() {
        return Ok(tangent_elementary(&s1, &s2, lin_tol));
    }
    same_cylinder(&s1, &s2, lin_tol)
}

/// The planar case of `IsSameDomain`.
fn same_plane(
    face: &Face,
    checked: &Face,
    (s1, s2): (&Handle<dyn GeomSurface>, &Handle<dyn GeomSurface>),
    (lin_tol, ang_tol): (f64, f64),
    planes: &mut DataMapOfFacePlane,
) -> Result<bool, Failure> {
    let checker1 = IsPlanarSurface::new(s1, lin_tol)?;
    if !checker1.is_planar() {
        return Ok(false);
    }
    let checker2 = IsPlanarSurface::new(s2, lin_tol)?;
    if !checker2.is_planar() {
        return Ok(false);
    }
    let (pln1, pln2) = (checker1.plan()?, checker2.plan()?);
    let direction = pln1.position().direction();
    if !(direction.is_parallel(pln2.position().direction(), ang_tol)
        && pln1.distance_plane(pln2) < lin_tol)
    {
        return Ok(false);
    }
    let plane = match (planes.seek(face), planes.seek(checked)) {
        (Some(plane), _) | (None, Some(plane)) => plane.clone(),
        (None, None) => Handle::new(GeomPlane::from_pln(pln1)),
    };
    planes.bind(face.clone().into_shape(), plane.clone());
    planes.bind(checked.clone().into_shape(), plane);
    Ok(true)
}

/// The elementary case of `IsSameDomain`: any failure of the intersection is false.
fn tangent_elementary(
    s1: &Handle<dyn GeomSurface>,
    s2: &Handle<dyn GeomSurface>,
    tol: f64,
) -> bool {
    let attempt = || -> Result<bool, Failure> {
        let a1: Adaptor3dSurfaceHandle = Handle::new(GeomAdaptorSurface::with_surface(s1)?).cast();
        let a2: Adaptor3dSurfaceHandle = Handle::new(GeomAdaptorSurface::with_surface(s2)?).cast();
        let (t1, t2) = (
            BRepTopAdaptorTopolTool::new(),
            BRepTopAdaptorTopolTool::new(),
        );
        let inter = IntPatchImpImpIntersection::with_surfaces(&a1, &t1, &a2, &t2, tol, tol, false)?;
        if !inter.is_done() || inter.is_empty()? {
            return Ok(false);
        }
        inter.tangent_faces()
    };
    attempt().unwrap_or(false)
}

fn is_cylinder_like(surface: &Handle<dyn GeomSurface>) -> bool {
    !surface.cast::<GeomCylindricalSurface>().is_null()
        || !surface.cast::<dyn GeomSweptSurface>().is_null()
}

/// The cylinder case of `IsSameDomain`.
fn same_cylinder(
    s1: &Handle<dyn GeomSurface>,
    s2: &Handle<dyn GeomSurface>,
    lin_tol: f64,
) -> Result<bool, Failure> {
    if s1.is_null() || s2.is_null() {
        return Err(null_surface());
    }
    if !(is_cylinder_like(s1) && is_cylinder_like(s2)) {
        return Ok(false);
    }
    let (Some(cyl1), Some(cyl2)) = (get_cylinder(s1)?, get_cylinder(s2)?) else {
        return Ok(false);
    };
    same_axis(&cyl1, &cyl2, lin_tol)
}

fn same_axis(cyl1: &Cylinder3, cyl2: &Cylinder3, lin_tol: f64) -> Result<bool, Failure> {
    if (cyl1.radius() - cyl2.radius()).abs() >= lin_tol {
        return Ok(false);
    }
    let direction = cyl1.position().direction();
    if !direction.is_parallel(cyl2.position().direction(), precision::angular()) {
        return Ok(false);
    }
    let offset = Vector3::between(cyl1.location(), cyl2.location());
    Ok(offset.square_magnitude() < lin_tol * lin_tol
        || offset.is_parallel(Vector3::from(direction), precision::angular())?)
}
