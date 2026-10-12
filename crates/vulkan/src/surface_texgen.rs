//! Snapshot only coordinate inputs; binding-time Eye planes follow the final draw order.
use crate::{RenderError, RenderVertex, error::rejected, surface_texture_uv::SurfaceUvTransform};
use rustycad_tkernel::ncollection::{Mat4, Vec4};
use rustycad_tkservice::graphic3d::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SurfaceTexgen {
    pub mode: Graphic3dTypeOfTextureMode,
    pub planes: [[f32; 4]; 2],
    pub model_view: Mat4<f32>,
    pub input: Vec<[f32; 4]>,
    pub transform: SurfaceUvTransform,
}

impl SurfaceTexgen {
    /// Freeze live parameters before frame submission; `SurfaceTexgen::snapshot(uv, params, top_down, world, view)`.
    pub fn snapshot(
        input: Vec<[f32; 4]>,
        parameters: &Graphic3dTextureParams,
        top_down: bool,
        world: [f32; 16],
        view: [f32; 16],
    ) -> Box<Self> {
        Box::new(Self {
            mode: parameters.gen_mode(),
            planes: [
                parameters.gen_plane_s().data(),
                parameters.gen_plane_t().data(),
            ],
            model_view: Mat4::from_data(view) * Mat4::from_data(world),
            input,
            transform: SurfaceUvTransform::read(parameters, top_down),
        })
    }
}
pub(crate) struct TextureGenerationBinding {
    mode: Graphic3dTypeOfTextureMode,
    planes: [[f32; 4]; 2],
    inverse_bound: Option<Mat4<f32>>,
}
impl TextureGenerationBinding {
    /// GL_EYE_PLANE is transformed at binding under the loaded matrix; `Self::new(snapshot)?`.
    pub fn new(snapshot: &SurfaceTexgen) -> Result<Self, RenderError> {
        let inverse_bound = if snapshot.mode == Graphic3dTypeOfTextureMode::Eye {
            Some(snapshot.model_view.inverted().map_err(|_| {
                rejected(
                    snapshot.model_view.data(),
                    "invertible binding-time Eye model-view",
                )
            })?)
        } else {
            None
        };
        Ok(Self {
            mode: snapshot.mode,
            planes: snapshot.planes,
            inverse_bound,
        })
    }
    pub fn coordinates(
        &self,
        snapshot: &SurfaceTexgen,
        vertices: &[RenderVertex],
    ) -> Result<Vec<[f32; 4]>, RenderError> {
        if snapshot.input.len() != vertices.len() {
            return Err(rejected(
                (snapshot.input.len(), vertices.len()),
                "one raw homogeneous UV per texgen vertex",
            ));
        }
        Ok(vertices
            .iter()
            .zip(&snapshot.input)
            .map(|(vertex, input)| self.vertex_coordinates(snapshot, vertex, input))
            .collect())
    }
    fn vertex_coordinates(
        &self,
        snapshot: &SurfaceTexgen,
        vertex: &RenderVertex,
        input: &[f32; 4],
    ) -> [f32; 4] {
        let position = Vec4::new(
            vertex.position[0],
            vertex.position[1],
            vertex.position[2],
            vertex.weight,
        );
        let position = self.inverse_bound.map_or(position, |inverse| {
            inverse * (snapshot.model_view * position)
        });
        snapshot
            .transform
            .generated(self.generated_input(input, position))
    }
    fn generated_input(&self, input: &[f32; 4], position: Vec4<f32>) -> [f32; 4] {
        if self.mode == Graphic3dTypeOfTextureMode::Manual {
            return *input;
        }
        let s = Vec4::from_array(self.planes[0]).dot(&position);
        let t = Vec4::from_array(self.planes[1]).dot(&position);
        [s, t, input[2], input[3]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn texgen_fixture(mode: Graphic3dTypeOfTextureMode, translation: f32) -> Box<SurfaceTexgen> {
        let parameters = Graphic3dTextureParams::default();
        parameters.set_gen_mode(
            mode,
            Vec4::new(0.0625, 0., 0., 0.5),
            Vec4::new(0., 0.0625, 0., 0.5),
        );
        let mut matrix = Mat4::<f32>::identity();
        matrix.set_value(0, 3, translation);
        SurfaceTexgen::snapshot(
            vec![[0., 0., 7., 2.]],
            &parameters,
            false,
            matrix.data(),
            Mat4::<f32>::identity().data(),
        )
    }
    #[test]
    fn object_generation_preserves_supplied_r_q_and_ignores_model_translation() {
        let snapshot = texgen_fixture(Graphic3dTypeOfTextureMode::Object, 4.);
        let binding = TextureGenerationBinding::new(&snapshot).unwrap();
        let vertices = [RenderVertex::new([-8., -8., 0.], [1.; 4])];
        assert_eq!(
            binding.coordinates(&snapshot, &vertices).unwrap(),
            vec![[0., 0., 7., 2.]]
        );
    }
    #[test]
    fn eye_generation_retains_the_binding_matrix_until_the_image_is_rebound() {
        let first = texgen_fixture(Graphic3dTypeOfTextureMode::Eye, -4.);
        let second = texgen_fixture(Graphic3dTypeOfTextureMode::Eye, 4.);
        let vertices = [RenderVertex::new([-8., -8., 0.], [1.; 4])];
        let binding = TextureGenerationBinding::new(&first).unwrap();
        assert_eq!(
            binding.coordinates(&second, &vertices).unwrap(),
            vec![[0.5, 0., 7., 2.]]
        );
        let rebound = TextureGenerationBinding::new(&second).unwrap();
        assert_eq!(
            rebound.coordinates(&second, &vertices).unwrap(),
            vec![[0., 0., 7., 2.]]
        );
    }
    #[test]
    fn manual_compatibility_keeps_r_q_and_rejects_wrong_shape_or_singular_eye_binding() {
        let mut snapshot = texgen_fixture(Graphic3dTypeOfTextureMode::Manual, 0.);
        let binding = TextureGenerationBinding::new(&snapshot).unwrap();
        let vertices = [RenderVertex::new([9.; 3], [1.; 4])];
        assert_eq!(
            binding.coordinates(&snapshot, &vertices).unwrap(),
            snapshot.input
        );
        assert!(binding.coordinates(&snapshot, &[]).is_err());
        snapshot.mode = Graphic3dTypeOfTextureMode::Eye;
        snapshot.model_view = Mat4::from_data([0.; 16]);
        assert!(TextureGenerationBinding::new(&snapshot).is_err());
    }
}
