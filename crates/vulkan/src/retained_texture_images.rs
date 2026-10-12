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
enum SurfaceImageLoad {
    Ready(SurfaceTextureImage),
    Unavailable,
    Empty,
}
type SurfaceSnapshotMap = BTreeMap<Vec<u8>, Arc<SurfaceTextureImage>>;
type ReleasedSurfaceOwners =
    Mutex<BTreeMap<Vec<u8>, Arc<crate::surface_texture_lifetime::SurfaceTextureLifetime>>>;
#[derive(Default)]
pub(crate) struct RetainedTextureImages {
    pub coordinate_pipeline: crate::RenderTextureCoordinatePipeline,
    images: Mutex<SurfaceSnapshotMap>,
    released: ReleasedSurfaceOwners,
    pub cap_textures: crate::retained_cap_textures::RetainedCapTextures,
    pub samplers: crate::retained_texture_samplers::RetainedTextureSamplers,
}
impl RetainedTextureImages {
    /// Inject coordinate semantics without coupling image revisions to UVs; `Self::with_coordinate_pipeline(policy)`.
    pub fn with_coordinate_pipeline(
        coordinate_pipeline: crate::RenderTextureCoordinatePipeline,
    ) -> Self {
        Self {
            coordinate_pipeline,
            ..Default::default()
        }
    }
    /// Retire dead image handles without invalidating submitted snapshots; `cache.prune()?`.
    pub fn prune(&self) -> Result<(), RenderError> {
        self.images
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned texture snapshot cache"))?
            .retain(|_, image| image.lifetime.is_alive());
        self.released
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned released image cache"))?
            .retain(|_, lifetime| lifetime.is_alive());
        self.cap_textures.prune()?;
        self.samplers.prune()
    }
    pub fn was_released(&self, id: &[u8]) -> Result<bool, RenderError> {
        let released = self
            .released
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned released image cache"))?;
        Ok(released.contains_key(id))
    }
    pub fn bound_snapshot(
        &self,
        id: &[u8],
    ) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
        let images = self
            .images
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned texture snapshot cache"))?;
        Ok(images.get(id).cloned())
    }
    /// Freeze content at its native revision; `cache.snapshot(&texture)?`.
    pub fn snapshot(
        &self,
        handle: &Handle<dyn Graphic3dTextureMap>,
    ) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
        let id = handle.get_id().as_bytes().to_vec();
        let revision = handle.revision();
        let mut images = self
            .images
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned texture snapshot cache"))?;
        images.retain(|_, image| image.lifetime.is_alive());
        if let Some(image) = images
            .get(&id)
            .filter(|image| !id.is_empty() && image.revision == revision)
        {
            image.lifetime.register(handle);
            return Ok(Some(image.clone()));
        }
        cache_surface_snapshot(&mut images, handle, id, revision, &self.released)
    }
}
fn cache_surface_snapshot(
    images: &mut SurfaceSnapshotMap,
    handle: &Handle<dyn Graphic3dTextureMap>,
    id: Vec<u8>,
    revision: usize,
    released: &ReleasedSurfaceOwners,
) -> Result<Option<Arc<SurfaceTextureImage>>, RenderError> {
    let image = match load_surface_image(&**handle, id.clone(), revision)? {
        SurfaceImageLoad::Ready(image) => image,
        SurfaceImageLoad::Unavailable => return Ok(images.get(&id).cloned()),
        SurfaceImageLoad::Empty => {
            mark_released_image(released, handle, &id)?;
            images.remove(&id);
            return Ok(None);
        }
    };
    cache_ready_image(images, handle, id, image, released).map(Some)
}
fn cache_ready_image(
    images: &mut SurfaceSnapshotMap,
    handle: &Handle<dyn Graphic3dTextureMap>,
    id: Vec<u8>,
    image: SurfaceTextureImage,
    released: &ReleasedSurfaceOwners,
) -> Result<Arc<SurfaceTextureImage>, RenderError> {
    released
        .lock()
        .map_err(|e| rejected(e.to_string(), "unpoisoned released image cache"))?
        .remove(&id);
    image.lifetime.register(handle);
    let image = Arc::new(image);
    if !id.is_empty() {
        images.insert(id, image.clone());
    }
    Ok(image)
}
fn mark_released_image(
    released: &ReleasedSurfaceOwners,
    handle: &Handle<dyn Graphic3dTextureMap>,
    id: &[u8],
) -> Result<(), RenderError> {
    if id.is_empty() {
        return Ok(());
    }
    let mut released = released
        .lock()
        .map_err(|e| rejected(e.to_string(), "unpoisoned released image cache"))?;
    let lifetime = released.entry(id.to_vec()).or_default();
    lifetime.register(handle);
    Ok(())
}

fn load_surface_image(
    texture: &dyn Graphic3dTextureMap,
    id: Vec<u8>,
    revision: usize,
) -> Result<SurfaceImageLoad, RenderError> {
    let supported = surface_supported_formats();
    if let Some(image) = texture.get_compressed_image(&supported).get() {
        return crate::surface_texture_compressed::snapshot_compressed_texture(
            image, texture, id, revision,
        )
        .map(SurfaceImageLoad::Ready);
    }
    let handle = texture.get_image(&supported);
    let Some(image) = handle.get() else {
        return Ok(SurfaceImageLoad::Unavailable);
    };
    if image.is_empty() {
        return Ok(SurfaceImageLoad::Empty);
    }
    snapshot_uncompressed_image(image, texture, id, revision).map(SurfaceImageLoad::Ready)
}
fn snapshot_uncompressed_image(
    image: &ImagePixMap,
    texture: &dyn Graphic3dTextureMap,
    id: Vec<u8>,
    revision: usize,
) -> Result<SurfaceTextureImage, RenderError> {
    let (encoding, bytes) = snapshot_surface_pixels(image, texture.is_color_map())?;
    let width = u32::try_from(image.width()).map_err(|e| rejected(e, "32bit texture width"))?;
    let height = u32::try_from(image.height()).map_err(|e| rejected(e, "32bit texture height"))?;
    Ok(SurfaceTextureImage {
        lifetime: Arc::default(),
        id,
        revision,
        extent: [width, height],
        encoding,
        bytes: Arc::new(bytes),
        top_down: image.is_top_down(),
        mipmaps: texture.has_mipmaps(),
        supplied_levels: vec![],
    })
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
/// Reclaim snapshots even when the next view is empty; `prune_view_texture_snapshots(view)?`.
pub(crate) fn prune_view_texture_snapshots(view: &dyn Graphic3dCView) -> Result<(), RenderError> {
    let driver = view
        .structure_manager()
        .graphic_driver()
        .cast::<crate::Graphic3dRenderDriver>();
    if let Some(driver) = driver.get() {
        driver.texture_images.prune()?;
    }
    Ok(())
}
#[cfg(test)]
mod release_tests {
    use super::*;
    #[test]
    fn released_shared_images_keep_all_weak_owners_and_never_cache_empty_identity() {
        let cache = RetainedTextureImages::default();
        let first = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default())).cast();
        let second = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default())).cast();
        mark_released_image(&cache.released, &first, b"shared").unwrap();
        mark_released_image(&cache.released, &second, b"shared").unwrap();
        mark_released_image(&cache.released, &second, b"").unwrap();
        drop(first);
        cache.prune().unwrap();
        assert!(cache.was_released(b"shared").unwrap());
        assert!(!cache.was_released(b"").unwrap());
        drop(second);
        cache.prune().unwrap();
        assert!(!cache.was_released(b"shared").unwrap());
    }
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
    fn null_replacement_retains_native_content_but_empty_replacement_releases_and_repairs() {
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(one_pixel(255)));
        let cache = RetainedTextureImages::default();
        let original = cache.snapshot(&texture.cast()).unwrap().unwrap();
        texture.set_image(Handle::default());
        texture.update_revision();
        assert!(Arc::ptr_eq(
            &original,
            &cache.snapshot(&texture.cast()).unwrap().unwrap()
        ));
        texture.set_image(Handle::new(ImagePixMap::default()));
        texture.update_revision();
        assert!(cache.snapshot(&texture.cast()).unwrap().is_none());
        assert!(cache.images.lock().unwrap().is_empty());
        texture.set_image(one_pixel(64));
        texture.update_revision();
        assert_eq!(
            *cache.snapshot(&texture.cast()).unwrap().unwrap().bytes,
            vec![64, 0, 0, 255]
        );
        assert_eq!(*original.bytes, vec![255, 0, 0, 255]);
    }
    #[test]
    fn dead_texture_owners_are_reclaimed_while_existing_snapshots_remain_valid() {
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(one_pixel(255)));
        let cache = RetainedTextureImages::default();
        let prior = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert_eq!(cache.images.lock().unwrap().len(), 1);
        drop(texture);
        cache.prune().unwrap();
        assert!(cache.images.lock().unwrap().is_empty());
        assert_eq!(*prior.bytes, vec![255, 0, 0, 255]);
    }
    #[test]
    fn set_image_requires_content_revision_and_sampler_updates_reuse_upload() {
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(one_pixel(255)));
        let cache = RetainedTextureImages::default();
        let first = cache.snapshot(&texture.cast()).unwrap().unwrap();
        texture.set_image(one_pixel(64));
        texture.get_params().set_repeat(false);
        texture
            .get_params()
            .set_filter(Graphic3dTypeOfTextureFilter::Nearest);
        let stale = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &stale));
        assert_eq!(*stale.bytes, vec![255, 0, 0, 255]);
        texture.update_revision();
        let updated = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(!Arc::ptr_eq(&first, &updated));
        assert_eq!(*updated.bytes, vec![64, 0, 0, 255]);
        assert_eq!(updated.revision, first.revision + 1);
    }
    #[test]
    fn failed_image_is_not_cached_and_metadata_changes_require_revision() {
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default()));
        let cache = RetainedTextureImages::default();
        assert!(cache.snapshot(&texture.cast()).unwrap().is_none());
        texture.set_image(one_pixel(127));
        let first = cache.snapshot(&texture.cast()).unwrap().unwrap();
        texture.set_color_map(false);
        texture
            .get_image(&surface_supported_formats())
            .set_top_down(false);
        let stale = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &stale));
        texture.update_revision();
        let updated = cache.snapshot(&texture.cast()).unwrap().unwrap();
        assert_eq!(updated.encoding, SurfaceImageEncoding::Linear8);
        assert!(!updated.top_down);
    }
}
