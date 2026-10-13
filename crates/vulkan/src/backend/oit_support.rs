//! Query real floating attachment support behind a Martensite-owned boundary.
use super::runtime::VulkanRuntime;
use ash::vk;
pub(super) trait OitFormatProbe {
    fn floating_attachment(&self, format: vk::Format) -> bool;
}
pub(super) struct DeviceOitFormats<'a>(pub &'a VulkanRuntime);
impl OitFormatProbe for DeviceOitFormats<'_> {
    fn floating_attachment(&self, format: vk::Format) -> bool {
        let properties = unsafe {
            self.0
                .instance
                .get_physical_device_format_properties(self.0.physical, format)
        };
        properties
            .optimal_tiling_features
            .contains(oit_required_features())
    }
}
fn oit_required_features() -> vk::FormatFeatureFlags {
    vk::FormatFeatureFlags::COLOR_ATTACHMENT
        | vk::FormatFeatureFlags::COLOR_ATTACHMENT_BLEND
        | vk::FormatFeatureFlags::SAMPLED_IMAGE
        | vk::FormatFeatureFlags::TRANSFER_SRC
}
pub(super) fn floating_sample_counts(
    runtime: &VulkanRuntime,
    format: vk::Format,
) -> vk::SampleCountFlags {
    let usage = floating_attachment_usage();
    unsafe {
        runtime
            .instance
            .get_physical_device_image_format_properties(
                runtime.physical,
                format,
                vk::ImageType::TYPE_2D,
                vk::ImageTiling::OPTIMAL,
                usage,
                vk::ImageCreateFlags::empty(),
            )
    }
    .map(|properties| properties.sample_counts)
    .unwrap_or(vk::SampleCountFlags::TYPE_1)
}
fn floating_attachment_usage() -> vk::ImageUsageFlags {
    vk::ImageUsageFlags::COLOR_ATTACHMENT
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::TRANSFER_SRC
}
pub(super) fn weighted_attachment_formats(probe: &dyn OitFormatProbe) -> Option<[vk::Format; 2]> {
    let color = preferred_weighted_color(probe)?;
    let single = weighted_scalar_format(color);
    Some([
        color,
        if probe.floating_attachment(single) {
            single
        } else {
            color
        },
    ])
}
fn preferred_weighted_color(probe: &dyn OitFormatProbe) -> Option<vk::Format> {
    [
        vk::Format::R16G16B16A16_SFLOAT,
        vk::Format::R32G32B32A32_SFLOAT,
    ]
    .into_iter()
    .find(|format| probe.floating_attachment(*format))
}
fn weighted_scalar_format(color: vk::Format) -> vk::Format {
    if color == vk::Format::R16G16B16A16_SFLOAT {
        return vk::Format::R16_SFLOAT;
    }
    vk::Format::R32_SFLOAT
}
pub(super) fn peeling_attachment_formats(probe: &dyn OitFormatProbe) -> Option<[vk::Format; 3]> {
    let formats = [
        vk::Format::R32G32_SFLOAT,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::Format::R16G16B16A16_SFLOAT,
    ];
    formats
        .iter()
        .all(|format| probe.floating_attachment(*format))
        .then_some(formats)
}
#[cfg(test)]
mod tests {
    use super::*;
    struct NamedOitFormatDevice {
        formats: Vec<vk::Format>,
    }
    impl OitFormatProbe for NamedOitFormatDevice {
        fn floating_attachment(&self, format: vk::Format) -> bool {
            self.formats.contains(&format)
        }
    }
    #[test]
    fn native_half_preference_falls_back_to_scalar_float_and_equal_components() {
        let mut device = NamedOitFormatDevice {
            formats: vec![
                vk::Format::R16G16B16A16_SFLOAT,
                vk::Format::R16_SFLOAT,
                vk::Format::R32G32B32A32_SFLOAT,
            ],
        };
        assert_eq!(
            weighted_attachment_formats(&device),
            Some([vk::Format::R16G16B16A16_SFLOAT, vk::Format::R16_SFLOAT])
        );
        device
            .formats
            .retain(|format| *format != vk::Format::R16_SFLOAT);
        assert_eq!(
            weighted_attachment_formats(&device),
            Some([vk::Format::R16G16B16A16_SFLOAT; 2])
        );
    }
    #[test]
    fn float_color_and_scalar_fallback_remains_usable_without_half_formats() {
        let device = NamedOitFormatDevice {
            formats: vec![vk::Format::R32G32B32A32_SFLOAT, vk::Format::R32_SFLOAT],
        };
        assert_eq!(
            weighted_attachment_formats(&device),
            Some([vk::Format::R32G32B32A32_SFLOAT, vk::Format::R32_SFLOAT])
        );
    }
    #[test]
    fn peeling_requires_its_actual_rg32_half_color_contract() {
        let mut device = NamedOitFormatDevice {
            formats: vec![vk::Format::R32G32_SFLOAT, vk::Format::R16G16B16A16_SFLOAT],
        };
        assert!(peeling_attachment_formats(&device).is_some());
        device.formats = vec![vk::Format::R32G32B32A32_SFLOAT];
        assert!(peeling_attachment_formats(&device).is_none());
        device.formats.clear();
        assert!(weighted_attachment_formats(&device).is_none());
    }
}
