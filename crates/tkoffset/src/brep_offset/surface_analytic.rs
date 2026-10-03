// Analytic branches of BRepOffset::Surface; preserve native radius and frame conventions.
use super::surface::{BRepOffsetStatus, SurfaceOffset, trimmed_surface};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomGeometry, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis3, Vector3};
use std::f64::consts::PI;

pub(super) fn analytic_surface(
    input: &Handle<dyn GeomSurface>,
    offset: f64,
    allow_c0: bool,
) -> Result<SurfaceOffset, Failure> {
    let kind = input
        .dynamic_type()
        .expect("BRepOffset::Surface: null input; expected a non-null Geom_Surface");
    match kind.name() {
        "Geom_Plane" => plane_surface(&input.cast(), offset),
        "Geom_CylindricalSurface" => cylinder_surface(&input.cast(), offset),
        "Geom_ConicalSurface" => cone_surface(&input.cast(), offset),
        "Geom_SphericalSurface" => sphere_surface(&input.cast(), offset),
        "Geom_ToroidalSurface" => torus_surface(&input.cast(), offset),
        "Geom_RectangularTrimmedSurface" => trimmed_surface(
            &input.cast::<GeomRectangularTrimmedSurface>(),
            offset,
            allow_c0,
        ),
        _ => Ok((Handle::default(), BRepOffsetStatus::Good)),
    }
}

fn plane_surface(input: &GeomPlane, offset: f64) -> Result<SurfaceOffset, Failure> {
    let frame = input.position();
    let normal = Vector3::from(frame.x_direction()).crossed(Vector3::from(frame.y_direction()));
    let moved = input.translated(normal.scaled(offset))?;
    Ok((moved.cast(), BRepOffsetStatus::Good))
}

fn signed_radius(frame: Axis3, radius: f64, offset: f64) -> Result<f64, Failure> {
    if frame.direct()? {
        return Ok(radius + offset);
    }
    Ok(radius - offset)
}

fn radial_status(radius: f64) -> BRepOffsetStatus {
    if radius >= precision::confusion() {
        return BRepOffsetStatus::Good;
    }
    if radius <= -precision::confusion() {
        return BRepOffsetStatus::Reversed;
    }
    BRepOffsetStatus::Degenerated
}

fn cylinder_surface(input: &GeomCylindricalSurface, offset: f64) -> Result<SurfaceOffset, Failure> {
    let mut frame = input.position();
    let radius = signed_radius(frame, input.radius(), offset)?;
    let status = radial_status(radius);
    if status == BRepOffsetStatus::Degenerated {
        return Ok((Handle::default(), status));
    }
    if status == BRepOffsetStatus::Reversed {
        frame.rotate(frame.axis(), PI)?;
    }
    let result = GeomCylindricalSurface::new(frame, radius.abs())?;
    Ok((Handle::new(result).cast(), status))
}

// Native tests Radius >= 0, so NaN follows the reversal branch too.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn cone_surface(input: &GeomConicalSurface, offset: f64) -> Result<SurfaceOffset, Failure> {
    let mut frame = input.position();
    let mut angle = input.semi_angle();
    let mut radius = input.ref_radius() + offset * angle.cos();
    frame.translate(Vector3::from(frame.direction()).scaled(-offset * angle.sin()));
    if !(radius >= 0.0) {
        radius = -radius;
        frame.rotate(frame.axis(), PI)?;
        angle = -angle;
    }
    let result = GeomConicalSurface::new(frame, angle, radius)?;
    Ok((Handle::new(result).cast(), BRepOffsetStatus::Good))
}

fn sphere_surface(input: &GeomSphericalSurface, offset: f64) -> Result<SurfaceOffset, Failure> {
    let mut frame = input.position();
    let radius = signed_radius(frame, input.radius(), offset)?;
    let status = radial_status(radius);
    if status == BRepOffsetStatus::Degenerated {
        return Ok((Handle::default(), status));
    }
    if status == BRepOffsetStatus::Reversed {
        frame.rotate(frame.axis(), PI)?;
        frame.z_reverse();
    }
    let result = GeomSphericalSurface::new(frame, radius.abs())?;
    Ok((Handle::new(result).cast(), status))
}

// Preserve the native strict comparison, including unordered radii.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn torus_surface(input: &GeomToroidalSurface, offset: f64) -> Result<SurfaceOffset, Failure> {
    let frame = input.position();
    let major = input.major_radius();
    if !(input.minor_radius() < major) {
        return Ok((Handle::default(), BRepOffsetStatus::Good));
    }
    let minor = signed_radius(frame, input.minor_radius(), offset)?;
    let status = radial_status(minor);
    if status != BRepOffsetStatus::Good {
        return Ok((Handle::default(), status));
    }
    let result = GeomToroidalSurface::new(frame, major, minor)?;
    Ok((Handle::new(result).cast(), status))
}
