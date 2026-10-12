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
    pub fn native_alpha(
        &self, runtime: &Arc<VulkanRuntime>, source: &DrawPipeline,
        samples: ash::vk::SampleCountFlags, coverage: bool,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        let key = [source.handle.as_raw(), samples.as_raw() as u64, u64::from(coverage)];
        let mut cached = self.0.lock().map_err(|_| rejected("poisoned alpha pipeline cache", "an available renderer cache"))?;
        if let Some(pipeline) = cached.native_alpha.get(&key) { return Ok(pipeline.clone()); }
        let layout = cache_layout(&mut cached, runtime)?;
        let pipeline = Arc::new(DrawPipeline::new_native_alpha(runtime, source.kind, samples, coverage, &layout)?);
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
        let key = [
            source.handle.as_raw(),
            samples.as_raw() as u64,
            u64::from(coverage),
        ];
        let mut cached = self.0.lock().map_err(|_| {
            rejected(
                "poisoned multisample pipeline cache",
                "an available renderer cache",
            )
        })?;
        if let Some(pipeline) = cached.multisample.get(&key) {
            return Ok(pipeline.clone());
        }
        let pipeline = Arc::new(DrawPipeline::new_multisample(
            runtime,
            source.kind,
            samples,
            coverage,
        )?);
        cached.multisample.insert(key, pipeline.clone());
        Ok(pipeline)
    }
    pub fn composite(
        &self,
        runtime: &Arc<VulkanRuntime>,
        target: super::oit_composite::OitCompositeTarget,
    ) -> Result<Arc<super::oit_composite::OitCompositePipeline>, RenderError> {
        let key = [
            target.phase as u32,
            target.format.as_raw() as u32,
            target.depth.as_raw() as u32,
            u32::from(target.stencil),
            target.samples.as_raw(),
        ];
        let mut cached = self.0.lock().map_err(|_| {
            rejected(
                "poisoned OIT composite cache",
                "an available renderer cache",
            )
        })?;
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
        let mut cached = self
            .0
            .lock()
            .map_err(|_| rejected("poisoned OIT pipeline cache", "an available renderer cache"))?;
        cache_layout(&mut cached, runtime)
    }
    pub fn select(
        &self,
        runtime: &Arc<VulkanRuntime>,
        source: &DrawPipeline,
        target: OitPipelineTarget,
    ) -> Result<Arc<DrawPipeline>, RenderError> {
        let key = variant_key(source.handle.as_raw(), target);
        let mut cached = self
            .0
            .lock()
            .map_err(|_| rejected("poisoned OIT pipeline cache", "an available renderer cache"))?;
        if let Some(pipeline) = cached.pipelines.get(&key) {
            return Ok(pipeline.clone());
        }
        let layout = cache_layout(&mut cached, runtime)?;
        let pipeline = Arc::new(DrawPipeline::new_oit(
            runtime,
            source.kind,
            target,
            &layout,
        )?);
        cached.pipelines.insert(key, pipeline.clone());
        Ok(pipeline)
    }
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
#[cfg(test)]
mod tests {
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
