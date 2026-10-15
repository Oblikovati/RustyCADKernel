// SPDX-License-Identifier: GPL-2.0-only
//! The integrand of each face surface, read from the located face geometry: closed-form
//! families for the five analytic kinds, nested quadrature for every other surface.
use super::general::GeneralSurface;
use super::planar::planar_field;
use super::poly::Poly;
use super::revolution::{Frame, Profile, revolution_fields};
use super::trig_poly::TrigPoly;
use super::trim_integral::FaceIntegrand;
use super::univariate::Univariate;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Direction3};

/// The integrand of `surface` about `reference`; free-form surfaces integrate their `u`
/// antiderivative from `u_start`, a parameter inside the face.
/// Example: `face_integrand(adaptor, [0.0; 3], 0.0)?` on a plane gives `FaceIntegrand::Planar`.
pub(crate) fn face_integrand(
    surface: BRepAdaptorSurface,
    reference: [f64; 3],
    u_start: f64,
) -> Result<FaceIntegrand, Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Plane => {
            let frame = frame(&surface.plane()?.position(), reference);
            FaceIntegrand::Planar(planar_field(&frame))
        }
        SurfaceType::Cylinder => cylinder(&surface, reference)?,
        SurfaceType::Cone => cone(&surface, reference)?,
        SurfaceType::Sphere => sphere(&surface, reference)?,
        SurfaceType::Torus => torus(&surface, reference)?,
        _ => FaceIntegrand::General(Box::new(GeneralSurface::new(surface, reference, u_start))),
    })
}

fn cylinder(surface: &BRepAdaptorSurface, reference: [f64; 3]) -> Result<FaceIntegrand, Failure> {
    let cylinder = surface.cylinder()?;
    let radius = Poly::constant(cylinder.radius());
    let profile = profile(radius, Poly::linear(0.0, 1.0), 1.0, &cylinder.position())?;
    let frame = frame(&cylinder.position(), reference);
    Ok(FaceIntegrand::PolynomialRevolution(Box::new(
        revolution_fields(&profile, &frame),
    )))
}

fn cone(surface: &BRepAdaptorSurface, reference: [f64; 3]) -> Result<FaceIntegrand, Failure> {
    let cone = surface.cone()?;
    let (sin, cos) = cone.semi_angle().sin_cos();
    let radius = Poly::linear(cone.ref_radius(), sin);
    let profile = profile(radius, Poly::linear(0.0, cos), 1.0, &cone.position())?;
    let frame = frame(&cone.position(), reference);
    Ok(FaceIntegrand::PolynomialRevolution(Box::new(
        revolution_fields(&profile, &frame),
    )))
}

fn sphere(surface: &BRepAdaptorSurface, reference: [f64; 3]) -> Result<FaceIntegrand, Failure> {
    let sphere = surface.sphere()?;
    let r = sphere.radius();
    let (radius, height) = (TrigPoly::harmonic(1, r, 0.0), TrigPoly::harmonic(1, 0.0, r));
    let profile = profile(radius, height, r, &sphere.position())?;
    let frame = frame(&sphere.position(), reference);
    Ok(FaceIntegrand::PeriodicRevolution(Box::new(
        revolution_fields(&profile, &frame),
    )))
}

fn torus(surface: &BRepAdaptorSurface, reference: [f64; 3]) -> Result<FaceIntegrand, Failure> {
    let torus = surface.torus()?;
    let (major, minor) = (torus.major_radius(), torus.minor_radius());
    let radius = TrigPoly::constant(major).add(&TrigPoly::harmonic(1, minor, 0.0));
    let height = TrigPoly::harmonic(1, 0.0, minor);
    let profile = profile(radius, height, minor, &torus.position())?;
    let frame = frame(&torus.position(), reference);
    Ok(FaceIntegrand::PeriodicRevolution(Box::new(
        revolution_fields(&profile, &frame),
    )))
}

fn profile<V>(radius: V, height: V, speed: f64, position: &Axis3) -> Result<Profile<V>, Failure> {
    let handedness = if position.direct()? { 1.0 } else { -1.0 };
    Ok(Profile {
        radius,
        height,
        speed,
        handedness,
    })
}

fn frame(position: &Axis3, reference: [f64; 3]) -> Frame {
    let origin = position.location();
    let direction = |d: Direction3| *d.coordinates();
    Frame {
        offset: [0, 1, 2].map(|k| origin.coordinates()[k] - reference[k]),
        x: direction(position.x_direction()),
        y: direction(position.y_direction()),
        z: direction(position.direction()),
    }
}
