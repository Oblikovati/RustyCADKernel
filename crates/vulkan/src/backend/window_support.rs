//! Window extensions remain optional for offscreen-only devices.
use super::runtime::vk_error;
use crate::{RenderError, error::rejected};
use ash::{Entry, Instance, vk};
use std::ffi::CStr;
fn has_extension(available: &[vk::ExtensionProperties], name: &CStr) -> bool {
    available
        .iter()
        .any(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) } == name)
}
pub(super) fn instance_extensions(entry: &Entry) -> Result<Vec<*const i8>, RenderError> {
    let available = unsafe { entry.enumerate_instance_extension_properties(None) }
        .map_err(|e| vk_error("enumerate window extensions", e))?;
    let required = [
        ash::khr::surface::NAME,
        ash::khr::xlib_surface::NAME,
        ash::khr::get_surface_capabilities2::NAME,
        ash::ext::surface_maintenance1::NAME,
    ];
    Ok(if required.iter().all(|n| has_extension(&available, n)) {
        required.iter().map(|n| n.as_ptr()).collect()
    } else {
        vec![]
    })
}
pub(super) fn device_supported(
    instance: &Instance,
    physical: vk::PhysicalDevice,
) -> Result<bool, RenderError> {
    let available = unsafe { instance.enumerate_device_extension_properties(physical) }
        .map_err(|e| vk_error("enumerate swapchain extensions", e))?;
    if !has_extension(&available, ash::khr::swapchain::NAME)
        || !has_extension(&available, ash::ext::swapchain_maintenance1::NAME)
    {
        return Ok(false);
    }
    let mut maintenance = vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default();
    let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut maintenance);
    unsafe { instance.get_physical_device_features2(physical, &mut features) };
    Ok(maintenance.swapchain_maintenance1 == vk::TRUE)
}
pub(super) fn present_mode(
    modes: &[vk::PresentModeKHR],
    vsync: bool,
) -> Result<vk::PresentModeKHR, RenderError> {
    let preferred = if vsync {
        &[vk::PresentModeKHR::FIFO][..]
    } else {
        &[
            vk::PresentModeKHR::IMMEDIATE,
            vk::PresentModeKHR::MAILBOX,
            vk::PresentModeKHR::FIFO,
        ][..]
    };
    preferred
        .iter()
        .copied()
        .find(|m| modes.contains(m))
        .ok_or_else(|| {
            rejected(
                modes,
                "FIFO or a supported unsynchronized presentation mode",
            )
        })
}
pub(super) fn surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> Result<vk::SurfaceFormatKHR, RenderError> {
    let srgb = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
    for format in srgb {
        if formats.iter().any(|f| {
            f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && (f.format == format || f.format == vk::Format::UNDEFINED)
        }) {
            return Ok(vk::SurfaceFormatKHR {
                format,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            });
        }
    }
    Err(rejected(
        formats,
        "an RGBA8/BGRA8 sRGB nonlinear surface format",
    ))
}
pub(super) fn image_count(cap: &vk::SurfaceCapabilitiesKHR) -> u32 {
    let desired = cap.min_image_count.saturating_add(1);
    if cap.max_image_count == 0 {
        desired
    } else {
        desired.min(cap.max_image_count)
    }
}
pub(super) fn extent(
    cap: &vk::SurfaceCapabilitiesKHR,
    requested: [u32; 2],
) -> Result<vk::Extent2D, RenderError> {
    let chosen = if cap.current_extent.width != u32::MAX {
        cap.current_extent
    } else {
        vk::Extent2D {
            width: requested[0].clamp(cap.min_image_extent.width, cap.max_image_extent.width),
            height: requested[1].clamp(cap.min_image_extent.height, cap.max_image_extent.height),
        }
    };
    if chosen.width == 0 || chosen.height == 0 {
        return Err(rejected(chosen, "a nonzero drawable window extent"));
    }
    Ok(chosen)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presentation_modes_and_formats_are_explicit() {
        assert_eq!(
            present_mode(
                &[vk::PresentModeKHR::FIFO, vk::PresentModeKHR::IMMEDIATE],
                true
            )
            .unwrap(),
            vk::PresentModeKHR::FIFO
        );
        assert_eq!(
            present_mode(
                &[vk::PresentModeKHR::FIFO, vk::PresentModeKHR::IMMEDIATE],
                false
            )
            .unwrap(),
            vk::PresentModeKHR::IMMEDIATE
        );
        assert!(present_mode(&[], false).is_err());
        assert!(surface_format(&[]).is_err());
        let f = vk::SurfaceFormatKHR {
            format: vk::Format::UNDEFINED,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        };
        assert_eq!(
            surface_format(&[f]).unwrap().format,
            vk::Format::B8G8R8A8_SRGB
        );
    }
    #[test]
    fn surface_limits_bound_counts_and_dimensions() {
        let mut c = vk::SurfaceCapabilitiesKHR {
            min_image_count: 2,
            max_image_count: 2,
            current_extent: vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            },
            min_image_extent: vk::Extent2D {
                width: 1,
                height: 2,
            },
            max_image_extent: vk::Extent2D {
                width: 100,
                height: 80,
            },
            ..Default::default()
        };
        assert_eq!(image_count(&c), 2);
        c.max_image_count = 0;
        assert_eq!(image_count(&c), 3);
        assert_eq!(
            extent(&c, [200, 1]).unwrap(),
            vk::Extent2D {
                width: 100,
                height: 2
            }
        );
        c.current_extent = vk::Extent2D::default();
        assert!(extent(&c, [10, 10]).is_err());
        let names = [vk::ExtensionProperties::default()];
        assert!(!has_extension(&names, ash::khr::surface::NAME));
    }
}
