//! Named font fake provides deterministic geometry without filesystem or FreeType I/O.
use martensite_vulkan::*;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkservice::graphic3d::*;
#[derive(Default)]
pub struct FixedGlyphSource {
    pub requests: SharedState<Vec<(Vec<u8>, u32, f32)>>,
}
impl TextLayoutSource for FixedGlyphSource {
    fn layout(
        &self,
        text: &Graphic3dText,
        _: &dyn Graphic3dAspectsSource,
        params: &Graphic3dRenderingParams,
    ) -> Result<Option<GlyphLayout>, RenderError> {
        self.requests.write().push((
            text.text().to_c_string().to_vec(),
            params.resolution,
            text.height(),
        ));
        if text.text().is_empty() {
            return Ok(None);
        }
        Ok(Some(GlyphLayout {
            positions: vec![[0., 0.], [4., 0.], [4., 6.], [0., 6.]],
            uv: vec![[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
            indices: vec![0, 1, 2, 0, 2, 3],
            atlas: RasterImage::new([2, 2], vec![[1.; 4]; 4])?,
            formatter_bounds: rustycad_tkservice::font::FontRect {
                left: 0.,
                right: 4.,
                top: 6.,
                bottom: 0.,
            },
            point_size: text.height() as u32,
        }))
    }
}
