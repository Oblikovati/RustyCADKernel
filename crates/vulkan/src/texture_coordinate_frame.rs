//! Resolve native texture binding state after transparent/capping submission ordering.
use crate::{
    PrimitiveDraw, RenderError, RenderVertex,
    surface_texgen::{SurfaceTexgen, TextureGenerationBinding},
    surface_texture::{SurfaceTextureImage, SurfaceTextureStyle},
};
use std::sync::Arc;
#[derive(Default)]
struct OrderedTexgenBinding {
    active: Option<(Arc<SurfaceTextureImage>, TextureGenerationBinding)>,
}
/// Resolve immutable UVs and retire the temporary inputs; `resolve_ordered_texture_coordinates(draws)?`.
pub(crate) fn resolve_ordered_texture_coordinates(
    draws: &mut [PrimitiveDraw],
) -> Result<(), RenderError> {
    let mut binding = OrderedTexgenBinding::default();
    for draw in draws {
        binding.resolve(draw.surface_texture.as_mut(), &draw.vertices)?;
    }
    Ok(())
}
impl OrderedTexgenBinding {
    fn resolve(
        &mut self,
        style: Option<&mut SurfaceTextureStyle>,
        vertices: &[RenderVertex],
    ) -> Result<(), RenderError> {
        let Some(style) = style else {
            self.active = None;
            return Ok(());
        };
        let Some(snapshot) = style.pending_texgen.take() else {
            self.active = None;
            return Ok(());
        };
        let generation = self.bind(&style.image, &snapshot)?;
        style.uv = generation.coordinates(&snapshot, vertices)?;
        style.validate(vertices.len())
    }
    fn bind(
        &mut self,
        image: &Arc<SurfaceTextureImage>,
        snapshot: &SurfaceTexgen,
    ) -> Result<&TextureGenerationBinding, RenderError> {
        if self
            .active
            .as_ref()
            .is_none_or(|(bound, _)| !same_texgen_resource(bound, image))
        {
            self.active = Some((image.clone(), TextureGenerationBinding::new(snapshot)?));
        }
        Ok(&self.active.as_ref().expect("initialized texgen binding").1)
    }
}
fn same_texgen_resource(left: &Arc<SurfaceTextureImage>, right: &Arc<SurfaceTextureImage>) -> bool {
    // OCCT3d097a0 BindTextures compares shared resource pointers, not immutable upload revisions.
    if !left.id.is_empty() {
        return left.id == right.id;
    }
    Arc::ptr_eq(left, right)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::{graphic3d::*, image::*};
    #[test]
    fn shared_upload_revisions_keep_binding_identity_but_empty_ids_are_independent() {
        let cache = crate::retained_texture_images::RetainedTextureImages::default();
        let texture = binding_identity_texture();
        let first = cache.snapshot(&texture.cast()).unwrap().unwrap();
        texture.update_revision();
        let second = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(same_texgen_resource(&first, &second));
        texture.texture_root_base().fields.write().tex_id.clear();
        let unshared = cache.snapshot(&texture.cast()).unwrap().unwrap();
        let another = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(!same_texgen_resource(&unshared, &another));
        assert!(same_texgen_resource(&unshared, &unshared));
    }
    fn binding_identity_texture() -> Handle<Graphic3dTexture2D> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![255; 4]), 1, 1, 4));
        Handle::new(Graphic3dTexture2D::from_pix_map(image))
    }
}
