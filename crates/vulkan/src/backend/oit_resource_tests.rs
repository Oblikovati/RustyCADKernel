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

#[test]
fn real_oit_resources_and_each_geometry_attachment_layout_are_validation_clean() {
    let runtime = VulkanRuntime::create(VulkanOptions {
        validation: true,
        device_index: Some(0),
    })
    .unwrap();
    let extent = vk::Extent2D {
        width: 16,
        height: 16,
    };
    let layout = OitDescriptorLayout::new(&runtime, true).unwrap();
    let targets = OitTargets::create(
        &runtime,
        extent,
        RenderTransparency::Weighted { depth_factor: 0.5 },
        vk::SampleCountFlags::TYPE_1,
    )
    .unwrap()
    .unwrap();
    if let OitTargets::Weighted {
        accumulation,
        coverage,
    } = &targets
    {
        let _descriptors =
            OitPassDescriptors::new(&layout, [accumulation.view, coverage.view], 0.5).unwrap();
        assert_eq!(accumulation.xor_view, vk::ImageView::null());
    }
    check_geometry_pipelines(&runtime, &layout);
    check_peeling_resources(&runtime, extent, &layout);
    assert!(
        runtime.validation.errors().is_empty(),
        "{:?}",
        runtime.validation.errors()
    );
}
fn check_geometry_pipelines(
    runtime: &std::sync::Arc<VulkanRuntime>,
    layout: &std::sync::Arc<OitDescriptorLayout>,
) {
    let kind = PipelineVariant::from_index(0, false);
    for phase in [
        OitGeometryPhase::Weighted,
        OitGeometryPhase::Initial,
        OitGeometryPhase::Peeling,
    ] {
        let formats = if phase == OitGeometryPhase::Weighted {
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
        };
        let count = match phase {
            OitGeometryPhase::Weighted => 2,
            OitGeometryPhase::Initial => 1,
            OitGeometryPhase::Peeling => 3,
        };
        for fixed_output in [false, true] {
            let target = OitPipelineTarget {
                phase,
                formats,
                count,
                samples: vk::SampleCountFlags::TYPE_1,
                fixed_output,
            };
            let _pipeline = DrawPipeline::new_oit(runtime, kind, target, layout).unwrap();
        }
    }
}
fn check_peeling_resources(
    runtime: &std::sync::Arc<VulkanRuntime>,
    extent: vk::Extent2D,
    layout: &std::sync::Arc<OitDescriptorLayout>,
) {
    let targets = OitTargets::create(
        runtime,
        extent,
        RenderTransparency::DepthPeeling { layers: 4 },
        vk::SampleCountFlags::TYPE_1,
    )
    .unwrap()
    .unwrap();
    if let OitTargets::Peeling { ping, pong, back } = &targets {
        let _first = OitPassDescriptors::new(layout, [ping[0].view, ping[1].view], 0.).unwrap();
        let _second = OitPassDescriptors::new(layout, [pong[0].view, pong[1].view], 0.).unwrap();
        assert_ne!(ping[0].handle, pong[0].handle);
        assert_ne!(back.handle, ping[2].handle);
    }
    assert!(
        OitTargets::create(
            runtime,
            extent,
            RenderTransparency::Unordered,
            vk::SampleCountFlags::TYPE_1
        )
        .unwrap()
        .is_none()
    );
}
