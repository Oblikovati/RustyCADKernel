//! Surface-normal evaluation against Contap_SurfProps.
use super::common::{Reader, attempt, end, pnt, xyz};
use super::contana::contour_frame;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkhlr::contap::ContapSurfProps;
use rustycad_tkmath::gp::{Axis3, Point3};

fn warped_surface(height: f64) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut poles = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            poles[(i, j)] = Point3::new(
                f64::from(i - 1),
                f64::from(j - 1),
                f64::from((i - 1) * (j - 1)) * height,
            );
        }
    }
    Ok(Handle::new(GeomBezierSurface::new(&poles)?).cast())
}
fn make_surface(
    kind: i32,
    frame: Axis3,
    radius: f64,
    semi: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomSphericalSurface::new(frame, radius)?).cast(),
        2 => Handle::new(GeomCylindricalSurface::new(frame, radius)?).cast(),
        3 => Handle::new(GeomConicalSurface::new(frame, semi, radius)?).cast(),
        4 => Handle::new(GeomToroidalSurface::new(frame, radius, 0.5)?).cast(),
        _ => return warped_surface(radius),
    })
}
fn surface_adaptor(r: &mut Reader) -> Result<Adaptor3dSurfaceHandle, Failure> {
    let kind = r.integer();
    let frame = contour_frame(r)?;
    let (radius, semi) = (r.real(), r.real());
    let surface = make_surface(kind, frame, radius, semi)?;
    let bounds = (r.real(), r.real(), r.real(), r.real());
    Ok(Handle::new(GeomAdaptorSurface::with_bounds(&surface, bounds)?).cast())
}
/// Compare each native normal overload, including singular cones and generic patches.
pub fn properties_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let surface = surface_adaptor(&mut r)?;
        let (u, v) = (r.real(), r.real());
        print_surface_properties(&surface, u, v);
        Ok(())
    });
    end();
}
fn print_surface_properties(surface: &Adaptor3dSurfaceHandle, u: f64, v: f64) {
    attempt(|| {
        let (p, n) = ContapSurfProps::normale(surface, u, v)?;
        pnt(p);
        xyz(*n.xyz());
        Ok(())
    });
    attempt(|| {
        let (p, du, dv, n) = ContapSurfProps::deriv_and_norm(surface, u, v)?;
        pnt(p);
        xyz(*du.xyz());
        xyz(*dv.xyz());
        xyz(*n.xyz());
        Ok(())
    });
    attempt(|| {
        let (p, n, du, dv) = ContapSurfProps::norm_and_dn(surface, u, v)?;
        pnt(p);
        xyz(*n.xyz());
        xyz(*du.xyz());
        xyz(*dv.xyz());
        Ok(())
    });
}
