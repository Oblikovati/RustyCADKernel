// SPDX-License-Identifier: GPL-2.0-only
//! Body totals by the divergence theorem over the oriented faces:
//! `V = 1/3 sum s_f integral (P - Q).N`, `integral (P - Q) dV = 1/4 sum s_f integral ((P - Q).N)(P - Q)`.
use super::accumulate::{AREA, AREA_MOMENT, FLUX, FLUX_MOMENT, Moments};
use super::boundary::sub_shapes;
use super::face::{face_moments, vertex_mean};
use super::measured::BodyProperties;
use super::native::native_body;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// Area, volume, volume centroid and surface centroid of `shape` from its B-rep data.
///
/// Every face occurrence counts; reversed faces subtract their flux and internal or
/// external faces bound no volume. Free-form faces use nested adaptive quadrature. When a
/// face's trims do not close in its parameter plane the whole body uses the ported
/// `BRepGProp` fixed-order integration and reports `IntegrationMethod::NativeUnbounded`.
///
/// ```
/// use rustycad_kernel::properties::{body_properties, IntegrationMethod};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
///
/// let mut maker = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0).unwrap();
/// let properties = body_properties(maker.shape().unwrap()).unwrap();
/// assert_eq!(properties.method, IntegrationMethod::ClosedForm);
/// assert!((properties.volume.value - 24.0).abs() <= properties.volume.error_bound);
/// assert!((properties.area.value - 52.0).abs() <= properties.area.error_bound);
/// let [x, y, z] = properties.centroid.point;
/// assert!((x - 1.0).abs() + (y - 1.5).abs() + (z - 2.0).abs() <= 3.0 * properties.centroid.error_bound);
/// ```
pub fn body_properties(shape: &Shape) -> Result<BodyProperties, Failure> {
    let reference = vertex_mean(shape)?;
    let faces = sub_shapes(shape, ShapeEnum::Face)?;
    let mut total = Moments::default();
    for face in &faces {
        let Some(moments) = face_moments(&topods::face(face)?, reference)? else {
            return native_body(shape, faces.len());
        };
        total.add_face(&moments, orientation_sign(face.orientation()));
    }
    Ok(BodyProperties {
        area: total.measured(AREA, 1.0),
        volume: total.measured(FLUX, 1.0 / 3.0),
        centroid: total.centroid(reference, (FLUX, FLUX_MOMENT), 0.75),
        surface_centroid: total.centroid(reference, (AREA, AREA_MOMENT), 1.0),
        face_count: faces.len(),
        method: total.method,
    })
}

fn orientation_sign(orientation: Orientation) -> f64 {
    match orientation {
        Orientation::Forward => 1.0,
        Orientation::Reversed => -1.0,
        _ => 0.0,
    }
}
