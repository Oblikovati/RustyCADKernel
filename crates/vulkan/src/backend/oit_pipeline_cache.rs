//! Cached OIT pipelines retain their source pipeline identity and attachment formats.
use super::{
    oit_descriptor::OitDescriptorLayout, oit_pipeline::OitPipelineTarget, pipeline::DrawPipeline,
    runtime::VulkanRuntime,
};
use crate::{RenderError, error::rejected};
use ash::vk::Handle;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub(super) struct OitPipelineCache(Mutex<OitCachedVariants>);
#[derive(Default)]
struct OitCachedVariants {
    layout: Option<Arc<OitDescriptorLayout>>,
    pipelines: BTreeMap<OitVariantKey, Arc<DrawPipeline>>,
    multisample: BTreeMap<[u64; 3], Arc<DrawPipeline>>,
    native_alpha: BTreeMap<[u64; 3], Arc<DrawPipeline>>,
    composites: BTreeMap<[u32; 5], Arc<super::oit_composite::OitCompositePipeline>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OitVariantKey {
    source: u64,
    phase: u8,
    formats: [i32; 3],
    count: usize,
    samples: u32,
    fixed_output: bool,
}
impl OitPipelineCache {
    fn lock_variants(
        &self,
        offending: &str,
    ) -> Result<std::sync::MutexGuard<'_, OitCachedVariants>, RenderError> {
        self.0
            .lock()
            .map_err(|_| rejected(offending, "an available renderer cache"))
    }
    pub fn native_alpha(
        &self,
        runtime: &Arc<VulkanRuntime>,
        source: &DrawPipeline,
        samples: ash::vk::SampleCountFlags,
        coverage: bool,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        let key = raster_variant_key(source.handle.as_raw(), samples, coverage);
        let mut cached = self.lock_variants("poisoned alpha pipeline cache")?;
        if let Some(pipeline) = cached.native_alpha.get(&key) {
            return Ok(pipeline.clone());
        }
        let layout = cache_layout(&mut cached, runtime)?;
        let pipeline = native_alpha_pipeline(runtime, source, samples, coverage, &layout)?;
        cached.native_alpha.insert(key, pipeline.clone());
        Ok(pipeline)
    }
    pub fn multisample(
        &self,
        runtime: &Arc<VulkanRuntime>,
        source: &Arc<DrawPipeline>,
        samples: ash::vk::SampleCountFlags,
        coverage: bool,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        if samples == ash::vk::SampleCountFlags::TYPE_1 {
            return Ok(source.clone());
        }
        let key = raster_variant_key(source.handle.as_raw(), samples, coverage);
        let mut cached = self.lock_variants("poisoned multisample pipeline cache")?;
        if let Some(pipeline) = cached.multisample.get(&key) {
            return Ok(pipeline.clone());
        }
        let pipeline = multisample_pipeline(runtime, source, samples, coverage)?;
        cached.multisample.insert(key, pipeline.clone());
        Ok(pipeline)
    }
    pub fn composite(
        &self,
        runtime: &Arc<VulkanRuntime>,
        target: super::oit_composite::OitCompositeTarget,
    ) -> Result<Arc<super::oit_composite::OitCompositePipeline>, RenderError> {
        let key = composite_variant_key(target);
        let mut cached = self.lock_variants("poisoned OIT composite cache")?;
        if let Some(pipeline) = cached.composites.get(&key) {
            return Ok(pipeline.clone());
        }
        let pipeline = Arc::new(super::oit_composite::OitCompositePipeline::new(
            runtime, target,
        )?);
        cached.composites.insert(key, pipeline.clone());
        Ok(pipeline)
    }
    pub fn layout(
        &self,
        runtime: &Arc<VulkanRuntime>,
    ) -> Result<Arc<OitDescriptorLayout>, RenderError> {
        let mut cached = self.lock_variants("poisoned OIT pipeline cache")?;
        cache_layout(&mut cached, runtime)
    }
    pub fn select(
        &self,
        runtime: &Arc<VulkanRuntime>,
        source: &DrawPipeline,
        target: OitPipelineTarget,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        let key = variant_key(source.handle.as_raw(), target);
        let mut cached = self.lock_variants("poisoned OIT pipeline cache")?;
        if let Some(pipeline) = cached.pipelines.get(&key) {
            return Ok(pipeline.clone());
        }
        let layout = cache_layout(&mut cached, runtime)?;
        let pipeline = geometry_pipeline(runtime, source, target, &layout)?;
        cached.pipelines.insert(key, pipeline.clone());
        Ok(pipeline)
    }
}
fn raster_variant_key(source: u64, samples: ash::vk::SampleCountFlags, coverage: bool) -> [u64; 3] {
    [source, samples.as_raw() as u64, u64::from(coverage)]
}
fn cache_layout(
    cached: &mut OitCachedVariants,
    runtime: &Arc<VulkanRuntime>,
) -> Result<Arc<OitDescriptorLayout>, RenderError> {
    if let Some(layout) = &cached.layout {
        return Ok(layout.clone());
    }
    let layout = OitDescriptorLayout::new(runtime, true)?;
    cached.layout = Some(layout.clone());
    Ok(layout)
}
fn variant_key(source: u64, target: OitPipelineTarget) -> OitVariantKey {
    OitVariantKey {
        source,
        phase: target.phase as u8,
        formats: target.formats.map(|format| format.as_raw()),
        count: target.count,
        samples: target.samples.as_raw(),
        fixed_output: target.fixed_output,
    }
}
fn native_alpha_pipeline(
    runtime: &Arc<VulkanRuntime>,
    source: &DrawPipeline,
    samples: ash::vk::SampleCountFlags,
    coverage: bool,
    layout: &Arc<OitDescriptorLayout>,
) -> Result<Arc<DrawPipeline>, RenderError> {
    DrawPipeline::new_native_alpha(runtime, source.kind, samples, coverage, layout).map(Arc::new)
}
fn multisample_pipeline(
    runtime: &Arc<VulkanRuntime>,
    source: &DrawPipeline,
    samples: ash::vk::SampleCountFlags,
    coverage: bool,
) -> Result<Arc<DrawPipeline>, RenderError> {
    DrawPipeline::new_multisample(runtime, source.kind, samples, coverage).map(Arc::new)
}
fn geometry_pipeline(
    runtime: &Arc<VulkanRuntime>,
    source: &DrawPipeline,
    target: OitPipelineTarget,
    layout: &Arc<OitDescriptorLayout>,
) -> Result<Arc<DrawPipeline>, RenderError> {
    DrawPipeline::new_oit(runtime, source.kind, target, layout).map(Arc::new)
}
fn composite_variant_key(target: super::oit_composite::OitCompositeTarget) -> [u32; 5] {
    [
        target.phase as u32,
        target.format.as_raw() as u32,
        target.depth.as_raw() as u32,
        u32::from(target.stencil),
        target.samples.as_raw(),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn raster_cache_identity_keeps_source_samples_and_coverage_distinct() {
        let one = ash::vk::SampleCountFlags::TYPE_1;
        let four = ash::vk::SampleCountFlags::TYPE_4;
        assert_eq!(super::raster_variant_key(7, one, false), [7, 1, 0]);
        assert_ne!(
            super::raster_variant_key(7, one, false),
            super::raster_variant_key(8, one, false)
        );
        assert_ne!(
            super::raster_variant_key(7, one, false),
            super::raster_variant_key(7, four, false)
        );
        assert_ne!(
            super::raster_variant_key(7, four, false),
            super::raster_variant_key(7, four, true)
        );
    }

    use super::super::oit_pipeline::OitGeometryPhase;
    use super::*;
    use ash::vk;
    #[test]
    fn cache_identity_includes_geometry_output_and_actual_attachment_properties() {
        let mut target = OitPipelineTarget {
            phase: OitGeometryPhase::Weighted,
            formats: [vk::Format::R16G16B16A16_SFLOAT; 3],
            count: 2,
            samples: vk::SampleCountFlags::TYPE_1,
            fixed_output: false,
        };
        let first = variant_key(1, target);
        assert_ne!(first, variant_key(2, target));
        target.fixed_output = true;
        assert_ne!(first, variant_key(1, target));
        target.fixed_output = false;
        target.samples = vk::SampleCountFlags::TYPE_4;
        assert_ne!(first, variant_key(1, target));
        target.samples = vk::SampleCountFlags::TYPE_1;
        target.formats[1] = vk::Format::R16_SFLOAT;
        assert_ne!(first, variant_key(1, target));
    }
}
