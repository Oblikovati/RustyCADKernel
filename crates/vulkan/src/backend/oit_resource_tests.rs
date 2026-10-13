//! Real allocations catch attachment/descriptor/pipeline errors before draw replay is enabled.
use super::{
    oit_descriptor::{OitDescriptorLayout, OitPassDescriptors},
    oit_pipeline::{OitGeometryPhase, OitPipelineTarget},
    oit_targets::OitTargets,
    pipeline::DrawPipeline,
    pipeline_variants::PipelineVariant,
    runtime::{VulkanOptions, VulkanRuntime},
};
use crate::RenderTransparency;
use ash::vk;

use std::sync::Arc;
#[test]
fn real_oit_resources_and_each_geometry_attachment_layout_are_validation_clean() {
    let runtime = VulkanRuntime::create(VulkanOptions {
        validation: true,
        device_index: Some(0),
    })
    .unwrap();
    let layout = OitDescriptorLayout::new(&runtime, true).unwrap();
    check_weighted_resources(&runtime, &layout);
    check_geometry_pipelines(&runtime, &layout);
    check_peeling_resources(&runtime, &layout);
    assert!(allocate_targets(&runtime, RenderTransparency::Unordered).is_none());
    assert!(
        runtime.validation.errors().is_empty(),
        "{:?}",
        runtime.validation.errors()
    );
}
fn allocate_targets(runtime: &Arc<VulkanRuntime>, mode: RenderTransparency) -> Option<OitTargets> {
    let extent = vk::Extent2D {
        width: 16,
        height: 16,
    };
    OitTargets::create(runtime, extent, mode, vk::SampleCountFlags::TYPE_1).unwrap()
}
fn check_weighted_resources(runtime: &Arc<VulkanRuntime>, layout: &Arc<OitDescriptorLayout>) {
    let targets =
        allocate_targets(runtime, RenderTransparency::Weighted { depth_factor: 0.5 }).unwrap();
    let OitTargets::Weighted {
        accumulation,
        coverage,
    } = &targets
    else {
        panic!("expected allocated weighted targets");
    };
    let _descriptors =
        OitPassDescriptors::new(layout, [accumulation.view, coverage.view], 0.5).unwrap();
    assert_eq!(accumulation.xor_view, vk::ImageView::null());
    assert_eq!(targets.geometry_shape().0[0], accumulation.format);
}
fn check_geometry_pipelines(runtime: &Arc<VulkanRuntime>, layout: &Arc<OitDescriptorLayout>) {
    let kind = PipelineVariant::from_index(0, false);
    for phase in [
        OitGeometryPhase::Weighted,
        OitGeometryPhase::Initial,
        OitGeometryPhase::Peeling,
    ] {
        for fixed_output in [false, true] {
            let target = expected_pipeline_target(phase, fixed_output);
            let _pipeline = DrawPipeline::new_oit(runtime, kind, target, layout).unwrap();
        }
    }
}
fn expected_pipeline_target(phase: OitGeometryPhase, fixed_output: bool) -> OitPipelineTarget {
    let formats = expected_geometry_formats(phase);
    let count = match phase {
        OitGeometryPhase::Weighted => 2,
        OitGeometryPhase::Initial => 1,
        OitGeometryPhase::Peeling => 3,
    };
    OitPipelineTarget {
        phase,
        formats,
        count,
        samples: vk::SampleCountFlags::TYPE_1,
        fixed_output,
    }
}
fn check_peeling_resources(runtime: &Arc<VulkanRuntime>, layout: &Arc<OitDescriptorLayout>) {
    let targets =
        allocate_targets(runtime, RenderTransparency::DepthPeeling { layers: 4 }).unwrap();
    let OitTargets::Peeling { ping, pong, back } = &targets else {
        panic!("expected allocated peeling targets");
    };
    let _first = OitPassDescriptors::new(layout, [ping[0].view, ping[1].view], 0.).unwrap();
    let _second = OitPassDescriptors::new(layout, [pong[0].view, pong[1].view], 0.).unwrap();
    assert_ne!(ping[0].handle, pong[0].handle);
    assert_ne!(back.handle, ping[2].handle);
    assert_eq!(targets.geometry_shape().1, vk::SampleCountFlags::TYPE_1);
}

fn expected_geometry_formats(phase: OitGeometryPhase) -> [vk::Format; 3] {
    if phase == OitGeometryPhase::Weighted {
        [
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Format::R16_SFLOAT,
            vk::Format::UNDEFINED,
        ]
    } else {
        [
            vk::Format::R32G32_SFLOAT,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::Format::R16G16B16A16_SFLOAT,
        ]
    }
}
