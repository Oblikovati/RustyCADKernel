//! Dedicated cap readiness follows MCountAspect; texture revisions alone do not refresh it.
use crate::{
    RenderError, retained_texture_images::RetainedTextureImages,
    surface_texture::SurfaceTextureImage,
};
use rustycad_tkernel::standard::{Handle, WeakHandle};
use rustycad_tkservice::graphic3d::*;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};
#[derive(Clone)]
pub(crate) struct CapTextureSnapshot {
    pub image: Option<Arc<SurfaceTextureImage>>,
    pub parameters: Handle<Graphic3dTextureParams>,
    pub resource_id: Vec<u8>,
}
struct CapTextureReadiness {
    owner: WeakHandle<Graphic3dClipPlane>,
    aspect_revision: u32,
    snapshot: CapTextureSnapshot,
}
type CapReadinessMap = BTreeMap<usize, CapTextureReadiness>;
#[derive(Default)]
pub(crate) struct RetainedCapTextures {
    bindings: Mutex<CapReadinessMap>,
}
impl RetainedCapTextures {
    pub fn prune(&self) -> Result<(), RenderError> {
        // Empty redraws also retire resources belonging to deleted planes.
        drop(self.live_bindings()?);
        Ok(())
    }
    fn live_bindings(&self) -> Result<MutexGuard<'_, CapReadinessMap>, RenderError> {
        let mut bindings = self
            .bindings
            .lock()
            .map_err(|e| crate::error::rejected(e.to_string(), "unpoisoned cap readiness cache"))?;
        bindings.retain(|_, binding| !binding.owner.upgrade().is_null());
        Ok(bindings)
    }
    pub fn snapshot(
        &self,
        plane: &Handle<Graphic3dClipPlane>,
        texture: &Handle<dyn Graphic3dTextureMap>,
        images: &RetainedTextureImages,
    ) -> Result<CapTextureSnapshot, RenderError> {
        let mut bindings = self.live_bindings()?;
        let key = plane.object_address() as usize;
        if let Some(binding) = bindings
            .get(&key)
            .filter(|binding| binding.cap_aspect_ready_for(plane))
        {
            return binding.bound_snapshot(images);
        }
        let binding = CapTextureReadiness::new(plane, texture, images)?;
        let snapshot = binding.snapshot.clone();
        bindings.insert(key, binding);
        Ok(snapshot)
    }
}
impl CapTextureReadiness {
    fn cap_aspect_ready_for(&self, plane: &Graphic3dClipPlane) -> bool {
        !plane.to_use_object_properties() && self.aspect_revision == plane.m_count_aspect()
    }

    fn new(
        plane: &Handle<Graphic3dClipPlane>,
        texture: &Handle<dyn Graphic3dTextureMap>,
        images: &RetainedTextureImages,
    ) -> Result<Self, RenderError> {
        Ok(Self {
            owner: plane.downgrade(),
            aspect_revision: plane.m_count_aspect(),
            snapshot: CapTextureSnapshot {
                image: images.snapshot(texture)?,
                parameters: texture.get_params(),
                resource_id: texture.get_id().as_bytes().to_vec(),
            },
        })
    }
    fn bound_snapshot(
        &self,
        images: &RetainedTextureImages,
    ) -> Result<CapTextureSnapshot, RenderError> {
        let mut snapshot = self.snapshot.clone();
        if let Some(image) = images.bound_snapshot(&self.snapshot.resource_id)? {
            snapshot.parameters = images
                .samplers
                .bound_parameters(&image, &snapshot.parameters)?;
            snapshot.image = Some(image);
        }
        if images.was_released(&self.snapshot.resource_id)? {
            snapshot.image = None;
        }
        Ok(snapshot)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
    fn cap_revision_image(red: u8) -> Handle<ImagePixMap> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![red, 0, 0, 255]), 1, 1, 4));
        image.set_top_down(true);
        image
    }
    fn snapshot_cap_fixture(
        cache: &RetainedTextureImages,
        plane: &Handle<Graphic3dClipPlane>,
        texture: &Handle<Graphic3dTexture2D>,
    ) -> CapTextureSnapshot {
        cache
            .cap_textures
            .snapshot(plane, &texture.cast(), cache)
            .unwrap()
    }
    #[test]
    fn dedicated_cap_content_stays_stale_until_its_aspect_counter_is_refreshed() {
        let cache = RetainedTextureImages::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(cap_revision_image(255)));
        let plane = Handle::new(Graphic3dClipPlane::default());
        let before = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        texture.set_image(cap_revision_image(0));
        texture.update_revision();
        let stale = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        assert!(Arc::ptr_eq(&before, &stale));
        plane.set_capping_texture(texture.cast()).unwrap();
        let refreshed = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        assert_eq!(before.bytes.as_slice(), [255, 0, 0, 255]);
        assert_eq!(refreshed.bytes.as_slice(), [0, 0, 0, 255]);
    }
    #[test]
    fn inherited_caps_revalidate_content_and_an_ordinary_upload_updates_a_shared_resource() {
        let cache = RetainedTextureImages::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(cap_revision_image(255)));
        let plane = Handle::new(Graphic3dClipPlane::default());
        snapshot_cap_fixture(&cache, &plane, &texture);
        texture.set_image(cap_revision_image(0));
        texture.update_revision();
        cache.snapshot(&texture.cast()).unwrap();
        let shared = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        assert_eq!(shared.bytes.as_slice(), [0, 0, 0, 255]);
        plane.set_use_object_texture(true);
        texture.set_image(cap_revision_image(255));
        texture.update_revision();
        let inherited = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        assert_eq!(inherited.bytes.as_slice(), [255, 0, 0, 255]);
    }
    #[test]
    fn cap_cache_does_not_retain_a_dead_plane_on_an_empty_redraw() {
        let cache = RetainedTextureImages::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(cap_revision_image(255)));
        let plane = Handle::new(Graphic3dClipPlane::default());
        snapshot_cap_fixture(&cache, &plane, &texture);
        assert_eq!(cache.cap_textures.bindings.lock().unwrap().len(), 1);
        drop(plane);
        cache.prune().unwrap();
        assert!(cache.cap_textures.bindings.lock().unwrap().is_empty());
    }
    #[test]
    fn shared_release_turns_stale_cap_black_and_repair_reuses_the_current_resource() {
        let cache = RetainedTextureImages::default();
        let texture = Handle::new(Graphic3dTexture2D::from_pix_map(cap_revision_image(255)));
        let plane = Handle::new(Graphic3dClipPlane::default());
        snapshot_cap_fixture(&cache, &plane, &texture);
        texture.set_image(Handle::new(ImagePixMap::default()));
        texture.update_revision();
        assert!(cache.snapshot(&texture.cast()).unwrap().is_none());
        assert!(
            snapshot_cap_fixture(&cache, &plane, &texture)
                .image
                .is_none()
        );
        texture.set_image(cap_revision_image(0));
        texture.update_revision();
        cache.snapshot(&texture.cast()).unwrap();
        let repaired = snapshot_cap_fixture(&cache, &plane, &texture)
            .image
            .unwrap();
        assert_eq!(repaired.bytes.as_slice(), [0, 0, 0, 255]);
    }
    #[test]
    fn unavailable_dedicated_cap_keeps_bound_identity_and_modulation_until_refresh() {
        let cache = RetainedTextureImages::default();
        let old = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default()));
        let replacement = Handle::new(Graphic3dTexture2D::from_pix_map(Handle::default()));
        replacement.get_params().set_modulate(false);
        let plane = Handle::new(Graphic3dClipPlane::default());
        let before = snapshot_cap_fixture(&cache, &plane, &old);
        let stale = snapshot_cap_fixture(&cache, &plane, &replacement);
        assert_eq!(before.resource_id, stale.resource_id);
        assert!(stale.parameters.is_modulate());
        plane.set_capping_texture(replacement.cast()).unwrap();
        let after = snapshot_cap_fixture(&cache, &plane, &replacement);
        assert_ne!(before.resource_id, after.resource_id);
        assert!(!after.parameters.is_modulate());
    }
}
