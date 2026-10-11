//! Image revisions freeze uploaded content; sampler and transform changes do not reload it.
use crate::{
    RenderError, error::rejected, surface_texture::*,
    surface_texture_pixels::snapshot_surface_pixels,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{graphic3d::*, image::*};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
#[derive(Default)]
pub(crate) struct RetainedTextureImages {
    images: Mutex<BTreeMap<Vec<u8>, Arc<SurfaceTextureImage>>>,
}
impl RetainedTextureImages {
    pub fn snapshot(
        &self,
        texture: &dyn Graphic3dTextureMap,
    ) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
        let id = texture.get_id().as_bytes().to_vec();
        let revision = texture.revision();
        let mut images = self
            .images
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned texture snapshot cache"))?;
        if let Some(image) = images
            .get(&id)
            .filter(|image| !id.is_empty() && image.revision == revision)
        {
            return Ok(Some(image.clone()));
        }
        let Some(image) = load_surface_image(texture, id.clone(), revision)? else {
            return Ok(None);
        };
        let image = Arc::new(image);
        if !id.is_empty() {
            images.insert(id, image.clone());
        }
        Ok(Some(image))
    }
}
fn load_surface_image(
    texture: &dyn Graphic3dTextureMap,
    id: Vec<u8>,
    revision: usize,
) -> Result<Option<SurfaceTextureImage>, RenderError> {
    let supported = surface_supported_formats();
    if let Some(image) = texture.get_compressed_image(&supported).get() {
        return crate::surface_texture_compressed::snapshot_compressed_texture(
            image, texture, id, revision,
        )
        .map(Some);
    }
    let handle = texture.get_image(&supported);
    let Some(image) = handle.get().filter(|image| !image.is_empty()) else {
        return Ok(None);
    };
    let (encoding, bytes) = snapshot_surface_pixels(image, texture.is_color_map())?;
    let width = u32::try_from(image.width()).map_err(|e| rejected(e, "32bit texture width"))?;
    let height = u32::try_from(image.height()).map_err(|e| rejected(e, "32bit texture height"))?;
    Ok(Some(SurfaceTextureImage {
        id,
        revision,
        extent: [width, height],
        encoding,
        bytes: Arc::new(bytes),
        top_down: image.is_top_down(),
        mipmaps: texture.has_mipmaps(),
        supplied_levels: vec![],
    }))
}
fn surface_supported_formats() -> Handle<ImageSupportedFormats> {
    let formats = Handle::new(ImageSupportedFormats::default());
    for kind in [
        ImageCompressedFormat::RgbS3tcDxt1,
        ImageCompressedFormat::RgbaS3tcDxt1,
        ImageCompressedFormat::RgbaS3tcDxt3,
        ImageCompressedFormat::RgbaS3tcDxt5,
    ] {
        formats.add_compressed(kind);
    }
    for kind in [ImageFormat::Rgb, ImageFormat::Rgba] {
        formats.add(kind);
    }
    formats
}

#[cfg(test)]
mod tests {
    use super::*;
    fn one_pixel(red: u8) -> Handle<ImagePixMap> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![red, 0, 0, 255]), 1, 1, 4));
        image.set_top_down(true);
        image
    }
    #[test]
    fn set_image_requires_content_revision_and_sampler_updates_reuse_upload() {
        let texture = Graphic3dTexture2D::from_pix_map(one_pixel(255));
        let cache = RetainedTextureImages::default();
        let first = cache.snapshot(&texture).unwrap().unwrap();
        texture.set_image(one_pixel(64));
        texture.get_params().set_repeat(false);
        texture
            .get_params()
            .set_filter(Graphic3dTypeOfTextureFilter::Nearest);
        let stale = cache.snapshot(&texture).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &stale));
        assert_eq!(*stale.bytes, vec![255, 0, 0, 255]);
        texture.update_revision();
        let updated = cache.snapshot(&texture).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &updated));
        assert_eq!(*updated.bytes, vec![64, 0, 0, 255]);
        assert_eq!(updated.revision, first.revision + 1);
    }
    #[test]
    fn failed_image_is_not_cached_and_metadata_changes_require_revision() {
        let texture = Graphic3dTexture2D::from_pix_map(Handle::default());
        let cache = RetainedTextureImages::default();
        assert!(cache.snapshot(&texture).unwrap().is_none());
        texture.set_image(one_pixel(127));
        let first = cache.snapshot(&texture).unwrap().unwrap();
        texture.set_color_map(false);
        texture
            .get_image(&surface_supported_formats())
            .set_top_down(false);
        let stale = cache.snapshot(&texture).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &stale));
        texture.update_revision();
        let updated = cache.snapshot(&texture).unwrap().unwrap();
        assert_eq!(updated.encoding, SurfaceImageEncoding::Linear8);
        assert!(!updated.top_down);
    }
}
