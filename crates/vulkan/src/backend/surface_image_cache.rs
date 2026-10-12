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
    pub fn forget(&self, resource_id: &[u8]) -> Result<(), RenderError> {
        self.images
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned sampled image cache"))?
            .remove(resource_id);
        Ok(())
    }

    pub fn prune(&self) -> Result<(), RenderError> {
        self.images
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned sampled image cache"))?
            .retain(|_, entry| entry.source.lifetime.is_alive());
        Ok(())
    }

    pub fn binding(
        &self,
        runtime: &Arc<VulkanRuntime>,
        layout: &SurfaceImageLayout,
        style: &SurfaceTextureStyle,
    ) -> Result<Arc<SurfaceImageBinding>, RenderError> {
        if style.image.id.is_empty() {
            let mut entry = SurfaceImageEntry::new(runtime, &style.image)?;
            return cached_surface_binding(&mut entry, layout, &style.sampler);
        }
        let mut images = self
            .images
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned sampled image cache"))?;
        images.retain(|_, entry| entry.source.lifetime.is_alive());
        let entry = cache_entry(&mut images, runtime, &style.image)?;
        cached_surface_binding(entry, layout, &style.sampler)
    }
}
fn cached_surface_binding(
    entry: &mut SurfaceImageEntry,
    layout: &SurfaceImageLayout,
    policy: &SurfaceSamplerPolicy,
) -> Result<Arc<SurfaceImageBinding>, RenderError> {
    let key = sampler_key(policy);
    if let Some(binding) = entry.bindings.get(&key) {
        return Ok(binding.clone());
    }
    let binding = SurfaceImageBinding::new(&entry.image, layout, policy)?;
    // Submitted draws retain their Arc; cached historical level ranges must not grow without bound.
    entry.bindings.clear();
    entry.bindings.insert(key, binding.clone());
    Ok(binding)
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PrimitiveDraw, RenderBackend, RenderFrame, RenderVertex, identity_clip};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::{
        graphic3d::*,
        image::{ImageFormat, ImagePixMap},
    };
    fn retained_red_texture() -> Handle<Graphic3dTexture2D> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![255, 0, 0, 255]), 1, 1, 4));
        image.set_top_down(true);
        Handle::new(Graphic3dTexture2D::from_pix_map(image))
    }
    fn sampled_texture_frame(texture: &Handle<Graphic3dTexture2D>) -> RenderFrame {
        let images = crate::retained_texture_images::RetainedTextureImages::default();
        let image = images.snapshot(&texture.cast()).unwrap().unwrap();
        let mut draw = PrimitiveDraw::new(
            vec![
                RenderVertex::new([-1., -1., 0.], [1.; 4]),
                RenderVertex::new([1., -1., 0.], [1.; 4]),
                RenderVertex::new([0., 1., 0.], [1.; 4]),
            ],
            vec![0, 1, 2],
            identity_clip(),
        )
        .unwrap();
        draw.surface_texture = Some(SurfaceTextureStyle {
            image,
            pending_texgen: None,
            sampler: SurfaceSamplerPolicy::read(&texture.get_params()),
            uv: vec![[0., 0., 0., 1.]; 3],
            back_vertex: true,
            vertex_override: false,
            modulate: true,
        });
        RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap()
    }
    #[test]
    fn dead_surface_owners_release_cached_gpu_images_on_an_empty_redraw() {
        let texture = retained_red_texture();
        let frame = sampled_texture_frame(&texture);
        let mut renderer =
            super::super::VulkanRenderer::new(super::super::VulkanOptions::default()).unwrap();
        renderer.render(&frame).unwrap();
        assert_eq!(cached_surface_count(&renderer), 1);
        drop(texture);
        let empty = RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![]).unwrap();
        renderer.render(&empty).unwrap();
        assert_eq!(cached_surface_count(&renderer), 0);
        assert!(
            renderer.validation_errors().is_empty(),
            "{:?}",
            renderer.validation_errors()
        );
    }
    fn cached_surface_count(renderer: &super::super::VulkanRenderer) -> usize {
        renderer
            .pipeline
            .surface_images
            .images
            .lock()
            .unwrap()
            .len()
    }
    #[test]
    fn sampler_policy_replacement_retires_old_bindings_without_accumulating_descriptors() {
        let texture = retained_red_texture();
        let frame = sampled_texture_frame(&texture);
        let mut renderer =
            super::super::VulkanRenderer::new(super::super::VulkanOptions::default()).unwrap();
        renderer.render(&frame).unwrap();
        let prior = retained_cached_binding(&renderer);
        for repeat in [false, true, false] {
            render_sampler_replacement(&mut renderer, &frame, repeat);
            assert_eq!(cached_sampler_count(&renderer), 1);
        }
        assert_eq!(Arc::strong_count(&prior), 1);
        renderer.render(&frame).unwrap();
        assert_eq!(cached_sampler_count(&renderer), 1);
        assert!(renderer.validation_errors().is_empty());
    }
    fn render_sampler_replacement(
        renderer: &mut super::super::VulkanRenderer,
        frame: &RenderFrame,
        repeat: bool,
    ) {
        let mut draw = frame.draws()[0].clone();
        draw.surface_texture.as_mut().unwrap().sampler.repeat = repeat;
        let updated = RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap();
        renderer.render(&updated).unwrap();
    }
    fn retained_cached_binding(
        renderer: &super::super::VulkanRenderer,
    ) -> Arc<SurfaceImageBinding> {
        renderer
            .pipeline
            .surface_images
            .images
            .lock()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .bindings
            .values()
            .next()
            .unwrap()
            .clone()
    }
    fn cached_sampler_count(renderer: &super::super::VulkanRenderer) -> usize {
        renderer
            .pipeline
            .surface_images
            .images
            .lock()
            .unwrap()
            .values()
            .map(|entry| entry.bindings.len())
            .sum()
    }
}
