// SPDX-License-Identifier: GPL-2.0-only
//! Fallback through the ported `BRepGProp` fixed-order integration for faces whose trims do
//! not close in the parameter plane (Green's theorem needs a closed boundary). OCCT gives no
//! error for it, so the bound is infinite: `IntegrationMethod::NativeUnbounded`.
use super::measured::{BodyProperties, IntegrationMethod, Measured, MeasuredPoint};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// Area and area centroid of `face` by `BRepGProp::surface_properties`, unbounded.
/// Example: `let (area, centroid, method) = native_face_area(&face)?;`.
pub(crate) fn native_face_area(
    face: &Shape,
) -> Result<(Measured, MeasuredPoint, IntegrationMethod), Failure> {
    let (area, centroid) = surface(face)?;
    Ok((area, centroid, IntegrationMethod::NativeUnbounded))
}

/// Area, volume and both centroids of `shape` natively; `native_body(&solid, faces)?`.
pub(crate) fn native_body(shape: &Shape, face_count: usize) -> Result<BodyProperties, Failure> {
    let (area, surface_centroid) = surface(shape)?;
    let mut props = GProps::new();
    BRepGProp::volume_properties(shape, &mut props, false, false, false)?;
    Ok(BodyProperties {
        area,
        volume: unbounded(props.mass()),
        centroid: centre(&props),
        surface_centroid,
        face_count,
        method: IntegrationMethod::NativeUnbounded,
    })
}

fn surface(shape: &Shape) -> Result<(Measured, MeasuredPoint), Failure> {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false)?;
    Ok((unbounded(props.mass()), centre(&props)))
}

fn unbounded(value: f64) -> Measured {
    Measured {
        value,
        error_bound: f64::INFINITY,
    }
}

fn centre(props: &GProps) -> MeasuredPoint {
    MeasuredPoint {
        point: *props.centre_of_mass().coordinates(),
        error_bound: f64::INFINITY,
    }
}
