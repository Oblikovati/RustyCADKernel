//! Cache image revisions and sampler policies; replacement retires old resources after frame references.
use super::{
    runtime::VulkanRuntime,
    sampled_surface_image::SampledSurfaceImage,
    surface_image_descriptor::{SurfaceImageBinding, SurfaceImageLayout},
};
use crate::{RenderError, error::rejected, surface_texture::*};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
type SurfaceSamplerKey = (bool, i32, i32, i32, i32);
struct SurfaceImageEntry {
    source: Arc<SurfaceTextureImage>,
    image: Arc<SampledSurfaceImage>,
    bindings: BTreeMap<SurfaceSamplerKey, Arc<SurfaceImageBinding>>,
}
#[derive(Default)]
pub(super) struct SurfaceImageCache {
    images: Mutex<BTreeMap<Vec<u8>, SurfaceImageEntry>>,
}
impl SurfaceImageCache {
    pub fn binding(
        &self,
        runtime: &Arc<VulkanRuntime>,
        layout: &SurfaceImageLayout,
        style: &SurfaceTextureStyle,
    ) -> Result<Arc<SurfaceImageBinding>, RenderError> {
        let mut images = self
            .images
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned sampled image cache"))?;
        let mut uncached;
        let entry = if style.image.id.is_empty() {
            uncached = SurfaceImageEntry::new(runtime, &style.image)?;
            &mut uncached
        } else {
            cache_entry(&mut images, runtime, &style.image)?
        };
        let key = sampler_key(&style.sampler);
        if let Some(binding) = entry.bindings.get(&key) {
            return Ok(binding.clone());
        }
        let binding = SurfaceImageBinding::new(&entry.image, layout, &style.sampler)?;
        entry.bindings.insert(key, binding.clone());
        Ok(binding)
    }
}
impl SurfaceImageEntry {
    fn new(
        runtime: &Arc<VulkanRuntime>,
        source: &Arc<SurfaceTextureImage>,
    ) -> Result<Self, RenderError> {
        Ok(Self {
            source: source.clone(),
            image: SampledSurfaceImage::new(runtime, source)?,
            bindings: BTreeMap::new(),
        })
    }
}
fn cache_entry<'a>(
    entries: &'a mut BTreeMap<Vec<u8>, SurfaceImageEntry>,
    runtime: &Arc<VulkanRuntime>,
    source: &Arc<SurfaceTextureImage>,
) -> Result<&'a mut SurfaceImageEntry, RenderError> {
    let stale = entries
        .get(&source.id)
        .is_none_or(|entry| !Arc::ptr_eq(&entry.source, source));
    if stale {
        entries.insert(source.id.clone(), SurfaceImageEntry::new(runtime, source)?);
    }
    Ok(entries.get_mut(&source.id).expect("inserted sampled image"))
}
fn sampler_key(policy: &SurfaceSamplerPolicy) -> SurfaceSamplerKey {
    (
        policy.repeat,
        policy.filter as i32,
        policy.anisotropy as i32,
        policy.first_level,
        policy.last_level,
    )
}
