//! Native shader texture transform preserves projective Q and packed live UV buffers.
use crate::{RenderError, error::rejected, primitive::PrimitiveSubmission};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn surface_texture_coordinates(
    primitive: &PrimitiveSubmission,
    parameters: &Graphic3dTextureParams,
    top_down: bool,
) -> Result<Vec<[f32; 4]>, RenderError> {
    let buffer = &*primitive.attributes;
    let attribute = buffer.attribute_data(Graphic3dTypeOfAttribute::Uv);
    (0..buffer.nb_elements())
        .map(|index| {
            let uv = match attribute {
                Some((slot, offset, stride)) => {
                    read_uv(buffer, slot, offset + stride * index as usize)?
                }
                None => [0., 0., 0., 1.],
            };
            Ok(transform_surface_uv(uv, parameters, top_down))
        })
        .collect()
}
fn read_uv(
    buffer: &dyn Graphic3dBufferSource,
    slot: i32,
    offset: usize,
) -> Result<[f32; 4], RenderError> {
    let kind = buffer
        .attribute(slot)
        .ok_or_else(|| rejected(slot, "allocated UV attribute"))?
        .data_type;
    let uv = match kind {
        Graphic3dTypeOfData::Vec2 => buffer
            .read_at::<Vec2<f32>>(offset)
            .map(|v| [v.x(), v.y(), 0., 1.]),
        Graphic3dTypeOfData::Vec3 => buffer
            .read_at::<Vec3<f32>>(offset)
            .map(|v| [v.x(), v.y(), v.z(), 1.]),
        Graphic3dTypeOfData::Vec4 => buffer.read_at::<Vec4<f32>>(offset).map(|v| v.data()),
        _ => return Err(rejected(kind, "Vec2, Vec3 or Vec4 texture coordinates")),
    };
    uv.ok_or_else(|| rejected(offset, "allocated UV bytes"))
}
pub(crate) fn transform_surface_uv(
    uv: [f32; 4],
    parameters: &Graphic3dTextureParams,
    top_down: bool,
) -> [f32; 4] {
    let angle = (-parameters.rotation()).to_radians();
    let (sin, cos) = angle.sin_cos();
    let scale = parameters.scale();
    let shift = parameters.translation();
    let x = uv[0] * cos - uv[1] * sin;
    let y = uv[0] * sin + uv[1] * cos;
    // The actual OCCT oracle uses bottom-up UV convention; Vulkan image rows retain source order.
    let ty = if top_down {
        -shift.y() + 1. / scale.y()
    } else {
        -shift.y()
    };
    let v = (y + ty) * if top_down { -scale.y() } else { scale.y() };
    [(x - shift.x()) * scale.x(), v, uv[2], uv[3]]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_packed_transform_rotates_before_translating_and_scaling() {
        let parameters = Graphic3dTextureParams::default();
        parameters.set_rotation(90.);
        parameters.set_scale(Vec2::new(2., 3.));
        parameters.set_translation(Vec2::new(0.25, 0.5));
        let result = transform_surface_uv([1., 2., 4., 5.], &parameters, false);
        for (actual, expected) in result.into_iter().zip([3.5, -4.5, 4., 5.]) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
    #[test]
    fn native_top_down_pack_preserves_projective_q() {
        let parameters = Graphic3dTextureParams::default();
        parameters.set_scale(Vec2::new(2., 4.));
        parameters.set_translation(Vec2::new(0.25, 0.5));
        assert_eq!(
            transform_surface_uv([1., 2., 3., 7.], &parameters, true),
            [1.5, -7., 3., 7.]
        );
        assert_eq!(
            transform_surface_uv([1., 2., 3., 7.], &parameters, false),
            [1.5, 6., 3., 7.]
        );
    }
}
