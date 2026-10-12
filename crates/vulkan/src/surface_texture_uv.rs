//! Native shader texture transform preserves projective Q and packed live UV buffers.
use crate::{RenderError, error::rejected, primitive::PrimitiveSubmission};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn surface_texture_coordinates(
    primitive: &PrimitiveSubmission,
    parameters: &Graphic3dTextureParams,
    top_down: bool,
) -> Result<Vec<[f32; 4]>, RenderError> {
    surface_input_uv(primitive).map(|coordinates| coordinates.into_iter()
        .map(|uv| transform_surface_uv(uv,parameters,top_down)).collect())
}
/// Preserve the actual UV buffer before choosing a coordinate pipeline; `surface_input_uv(primitive)?`.
pub(crate) fn surface_input_uv(primitive: &PrimitiveSubmission) -> Result<Vec<[f32;4]>,RenderError> {
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
            Ok(uv)
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
    SurfaceUvTransform::read(parameters,top_down).programmable(uv)
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) struct SurfaceUvTransform {
    sine: f32,
    cosine: f32,
    scale: [f32;2],
    shift: [f32;2],
}
impl SurfaceUvTransform {
    pub fn read(parameters:&Graphic3dTextureParams,top_down:bool) -> Self {
        let (sine,cosine)=(-parameters.rotation()).to_radians().sin_cos();
        let scale=parameters.scale();let shift=parameters.translation();
        // The actual OCCT oracle uses bottom-up UV convention; Vulkan image rows retain source order.
        let ty=-shift.y()+if top_down {1./scale.y()}else{0.};
        Self {sine,cosine,scale:[scale.x(),if top_down {-scale.y()}else{scale.y()}],shift:[-shift.x(),ty]}
    }
    pub fn programmable(self,uv:[f32;4]) -> [f32;4] {
        self.apply(uv,1.)
    }
    pub fn generated(self,uv:[f32;4]) -> [f32;4] {
        // OCCT3d097a0 SetTextureMatrix multiplies Q in FFP; THE_VARY_TexCoord_Trsf does not.
        self.apply(uv,uv[3])
    }
    fn apply(self,uv:[f32;4],weight:f32) -> [f32;4] {
        let x=uv[0]*self.cosine-uv[1]*self.sine;
        let y=uv[0]*self.sine+uv[1]*self.cosine;
        [(x+self.shift[0]*weight)*self.scale[0],(y+self.shift[1]*weight)*self.scale[1],uv[2],uv[3]]
    }
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
