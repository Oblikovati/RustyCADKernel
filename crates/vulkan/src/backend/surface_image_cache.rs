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
type SurfaceImageKey = (Vec<u8>, i32);
struct SurfaceImageEntry {
    source: Arc<SurfaceTextureImage>,
    image: Arc<SampledSurfaceImage>,
    bindings: BTreeMap<SurfaceSamplerKey, Arc<SurfaceImageBinding>>,
}
#[derive(Default)]
pub(super) struct SurfaceImageCache {
    images: Mutex<BTreeMap<SurfaceImageKey, SurfaceImageEntry>>,
}
impl SurfaceImageCache {
    pub fn touch(
        &self,
        runtime: &Arc<VulkanRuntime>,
        source: &Arc<SurfaceTextureImage>,
        encoded: bool,
    ) -> Result<(), RenderError> {
        if source.id.is_empty() {
            return Ok(());
        }
        let mut images = self.images.lock().map_err(|error| {
            rejected(
                error.to_string(),
                "unpoisoned unsampled texture upload cache",
            )
        })?;
        let format = super::surface_image_format::sampled_binding_format(source.encoding, encoded);
        cache_entry(&mut images, runtime, source, format)?;
        Ok(())
    }
    pub fn forget(&self, resource_id: &[u8]) -> Result<(), RenderError> {
        self.images
            .lock()
            .map_err(|error| rejected(error.to_string(), "unpoisoned sampled image cache"))?
            .retain(|(id, _), _| id != resource_id);
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
        encoded: bool,
    ) -> Result<Arc<SurfaceImageBinding>, RenderError> {
        let format =
            super::surface_image_format::sampled_binding_format(style.image.encoding, encoded);
        if style.image.id.is_empty() {
            let mut entry = SurfaceImageEntry::new(runtime, &style.image, format)?;
            return cached_surface_binding(&mut entry, layout, &style.sampler);
        }
        let mut images = self
            .images
            .lock()
            .map_err(|e| rejected(e.to_string(), "unpoisoned sampled image cache"))?;
        images.retain(|_, entry| entry.source.lifetime.is_alive());
        let entry = cache_entry(&mut images, runtime, &style.image, format)?;
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
        format: ash::vk::Format,
    ) -> Result<Self, RenderError> {
        Ok(Self {
            source: source.clone(),
            image: SampledSurfaceImage::new(runtime, source, format)?,
            bindings: BTreeMap::new(),
        })
    }
}
fn cache_entry<'a>(
    entries: &'a mut BTreeMap<SurfaceImageKey, SurfaceImageEntry>,
    runtime: &Arc<VulkanRuntime>,
    source: &Arc<SurfaceTextureImage>,
    format: ash::vk::Format,
) -> Result<&'a mut SurfaceImageEntry, RenderError> {
    let key = (source.id.clone(), format.as_raw());
    let stale = entries
        .get(&key)
        .is_none_or(|entry| !Arc::ptr_eq(&entry.source, source));
    if stale {
        entries.insert(
            key.clone(),
            SurfaceImageEntry::new(runtime, source, format)?,
        );
    }
    Ok(entries.get_mut(&key).expect("inserted sampled image"))
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
    #[test]
    fn shared_source_keeps_encoded_and_programmable_filter_images_distinct() {
        let texture = retained_checker_texture();
        let frame = bilinear_checker_frame(&texture);
        let mut renderer = texture_cache_renderer();
        let shader = renderer.render(&frame).unwrap();
        let fixed = renderer.render(&fixed_checker_frame(&frame)).unwrap();
        assert!(
            fixed.rgba[144..147]
                .iter()
                .all(|channel| (120..=135).contains(channel))
        );
        // Hardware sRGB conversion has existing one-byte profile differences; these spaces differ by ~60.
        assert!(shader.rgba[144..147].iter().all(|channel| *channel >= 180));
        assert_eq!(cached_surface_count(&renderer), 2);
        assert_eq!(cached_surface_formats(&renderer), [37, 43]);
        assert_eq!(renderer.render(&frame).unwrap().rgba, shader.rgba);
        assert_eq!(cached_surface_count(&renderer), 2);
        assert!(renderer.validation_errors().is_empty());
    }
    fn texture_cache_renderer() -> super::super::VulkanRenderer {
        let options = super::super::VulkanOptions::default();
        super::super::VulkanRenderer::new(options).unwrap()
    }
    fn cached_surface_formats(renderer: &super::super::VulkanRenderer) -> Vec<i32> {
        renderer
            .pipeline
            .surface_images
            .images
            .lock()
            .unwrap()
            .values()
            .map(|entry| entry.image.format.as_raw())
            .collect()
    }
    fn retained_checker_texture() -> Handle<Graphic3dTexture2D> {
        let texture = retained_red_texture();
        let image = Handle::new(ImagePixMap::default());
        let pixels = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(pixels), 2, 2, 8));
        image.set_top_down(true);
        texture.set_image(image);
        texture
    }
    fn bilinear_checker_frame(texture: &Handle<Graphic3dTexture2D>) -> RenderFrame {
        let frame = sampled_texture_frame(texture);
        let mut draw = frame.draws()[0].clone();
        let style = draw.surface_texture.as_mut().unwrap();
        style.uv = vec![[0.5, 0.5, 0., 1.]; 3];
        style.sampler.filter = Graphic3dTypeOfTextureFilter::Bilinear;
        RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap()
    }
    fn fixed_checker_frame(frame: &RenderFrame) -> RenderFrame {
        let mut draw = frame.draws()[0].clone();
        draw.surface_texture.as_mut().unwrap().fixed_environment = true;
        RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap()
    }
    #[test]
    fn encoded_variants_share_revision_replacement_forgetting_and_retirement() {
        let texture = retained_checker_texture();
        let mut renderer = texture_cache_renderer();
        render_both_texture_formats(&mut renderer, &bilinear_checker_frame(&texture));
        let prior = retained_cached_image(&renderer);
        texture.set_image(replacement_green_image());
        texture.update_revision();
        let updated = bilinear_checker_frame(&texture);
        render_both_texture_formats(&mut renderer, &updated);
        assert!(!Arc::ptr_eq(&prior, &retained_cached_image(&renderer)));
        forget_frame_texture_formats(&renderer, &updated);
        render_both_texture_formats(&mut renderer, &updated);
        drop(texture);
        renderer
            .render(&RenderFrame::new(8, 8, [0.; 4], vec![]).unwrap())
            .unwrap();
        assert_eq!(cached_surface_count(&renderer), 0);
        assert!(renderer.validation_errors().is_empty());
    }
    fn forget_frame_texture_formats(renderer: &super::super::VulkanRenderer, frame: &RenderFrame) {
        let resource_id = &frame.draws()[0].surface_texture.as_ref().unwrap().image.id;
        renderer
            .pipeline
            .surface_images
            .forget(resource_id)
            .unwrap();
        assert_eq!(cached_surface_count(renderer), 0);
    }
    fn render_both_texture_formats(
        renderer: &mut super::super::VulkanRenderer,
        frame: &RenderFrame,
    ) {
        renderer.render(frame).unwrap();
        renderer.render(&fixed_checker_frame(frame)).unwrap();
        assert_eq!(cached_surface_count(renderer), 2);
    }
    #[test]
    fn mixed_encoded_and_programmable_draws_resume_existing_color_and_depth() {
        let texture = retained_checker_texture();
        let fixed = fixed_checker_frame(&bilinear_checker_frame(&texture));
        let overlay = ordinary_left_overlay();
        let mixed = RenderFrame::new(
            8,
            8,
            [0., 0., 0., 1.],
            vec![fixed.draws()[0].clone(), overlay.draws()[0].clone()],
        )
        .unwrap();
        let mut renderer = texture_cache_renderer();
        let previous = renderer.render(&fixed).unwrap();
        let control = renderer.render(&overlay).unwrap();
        let result = renderer.render(&mixed).unwrap();
        assert_eq!(&result.rgba[132..136], &control.rgba[132..136]);
        assert_eq!(&result.rgba[144..148], &previous.rgba[144..148]);
        assert_eq!(result.depth[36], previous.depth[36]);
        assert!(renderer.validation_errors().is_empty());
    }
    fn ordinary_left_overlay() -> RenderFrame {
        let vertices = vec![
            RenderVertex::new([-1., -1., 0.], [0.25, 0.25, 0.25, 1.]),
            RenderVertex::new([0., -1., 0.], [0.25, 0.25, 0.25, 1.]),
            RenderVertex::new([-0.5, 1., 0.], [0.25, 0.25, 0.25, 1.]),
        ];
        let draw = PrimitiveDraw::new(vertices, vec![0, 1, 2], identity_clip()).unwrap();
        RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap()
    }
    #[test]
    fn mapped_line_uploads_image_revisions_without_allocating_sampler_bindings() {
        let texture = retained_red_texture();
        let frame = sampled_texture_frame(&texture);
        let mut renderer =
            super::super::VulkanRenderer::new(super::super::VulkanOptions::default()).unwrap();
        let line = mapped_line_frame(&frame);
        renderer.render(&line).unwrap();
        assert_eq!(cached_surface_count(&renderer), 1);
        assert_eq!(cached_sampler_count(&renderer), 0);
        let before = retained_cached_image(&renderer);
        texture.set_image(replacement_green_image());
        texture.update_revision();
        renderer
            .render(&mapped_line_frame(&sampled_texture_frame(&texture)))
            .unwrap();
        assert!(!Arc::ptr_eq(&before, &retained_cached_image(&renderer)));
        assert_eq!(cached_sampler_count(&renderer), 0);
        renderer.render(&sampled_texture_frame(&texture)).unwrap();
        assert_eq!(cached_sampler_count(&renderer), 1);
        assert!(renderer.validation_errors().is_empty());
    }
    fn replacement_green_image() -> Handle<ImagePixMap> {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(ImageFormat::Rgba, Some(vec![0, 255, 0, 255]), 1, 1, 4));
        image.set_top_down(true);
        image
    }
    fn mapped_line_frame(frame: &RenderFrame) -> RenderFrame {
        let mut draw = PrimitiveDraw::segments(
            frame.draws()[0].vertices[..2].to_vec(),
            vec![0, 1],
            identity_clip(),
            crate::LineStyle::new(1., 0xffff, 1).unwrap(),
        )
        .unwrap();
        draw.surface_texture = frame.draws()[0].surface_texture.clone();
        draw.surface_texture_binding_only = true;
        RenderFrame::new(8, 8, [0., 0., 0., 1.], vec![draw]).unwrap()
    }
    fn retained_cached_image(renderer: &super::super::VulkanRenderer) -> Arc<SampledSurfaceImage> {
        renderer
            .pipeline
            .surface_images
            .images
            .lock()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .image
            .clone()
    }
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
            fixed_environment: false,
            sampler: SurfaceSamplerPolicy::read(&texture.get_params()),
            uv: vec![[0., 0., 0., 1.]; 3],
            back_vertex: true,
            vertex_override: false,
            modulate: true,
            native_gen_mode: rustycad_tkservice::graphic3d::Graphic3dTypeOfTextureMode::Manual,
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
