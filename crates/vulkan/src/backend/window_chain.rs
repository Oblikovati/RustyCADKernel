//! Swapchain images use concurrent sharing only when presentation requires another queue family.
use super::{
    runtime::{VulkanRuntime, vk_error},
    window_support,
    window_sync::WindowSync,
};
use crate::{RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowConfig {
    pub extent: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub mode: vk::PresentModeKHR,
    pub count: u32,
    pub transform: vk::SurfaceTransformFlagsKHR,
    pub alpha: vk::CompositeAlphaFlagsKHR,
}
impl WindowConfig {
    pub fn query(
        runtime: &VulkanRuntime,
        surface: &ash::khr::surface::Instance,
        handle: vk::SurfaceKHR,
        requested: [u32; 2],
        vsync: bool,
    ) -> Result<Self, RenderError> {
        let cap =
            unsafe { surface.get_physical_device_surface_capabilities(runtime.physical, handle) }
                .map_err(|e| vk_error("query surface capabilities", e))?;
        if !cap
            .supported_usage_flags
            .contains(vk::ImageUsageFlags::TRANSFER_DST)
        {
            return Err(rejected(
                cap.supported_usage_flags,
                "swapchain transfer-destination support",
            ));
        }
        let formats =
            unsafe { surface.get_physical_device_surface_formats(runtime.physical, handle) }
                .map_err(|e| vk_error("query surface formats", e))?;
        let modes =
            unsafe { surface.get_physical_device_surface_present_modes(runtime.physical, handle) }
                .map_err(|e| vk_error("query present modes", e))?;
        let alpha = [
            vk::CompositeAlphaFlagsKHR::OPAQUE,
            vk::CompositeAlphaFlagsKHR::INHERIT,
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
        ]
        .into_iter()
        .find(|a| cap.supported_composite_alpha.contains(*a))
        .ok_or_else(|| {
            rejected(
                cap.supported_composite_alpha,
                "a supported composite alpha mode",
            )
        })?;
        let format = window_support::surface_format(&formats)?;
        validate_blit(runtime, format.format)?;
        Ok(Self {
            extent: window_support::extent(&cap, requested)?,
            format,
            mode: window_support::present_mode(&modes, vsync)?,
            count: window_support::image_count(&cap),
            transform: cap.current_transform,
            alpha,
        })
    }
}
pub(super) struct WindowChain {
    runtime: Arc<VulkanRuntime>,
    pub loader: ash::khr::swapchain::Device,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub config: WindowConfig,
    pub sync: WindowSync,
    pub acquired_index: Option<u32>,
    pub stale: bool,
}
impl WindowChain {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        surface: vk::SurfaceKHR,
        present_family: u32,
        config: WindowConfig,
        old: vk::SwapchainKHR,
    ) -> Result<Self, RenderError> {
        let families = [runtime.queue_family, present_family];
        let mut info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(config.count)
            .image_format(config.format.format)
            .image_color_space(config.format.color_space)
            .image_extent(config.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(config.transform)
            .composite_alpha(config.alpha)
            .present_mode(config.mode)
            .clipped(true)
            .old_swapchain(old);
        if families[0] != families[1] {
            info = info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&families);
        }
        let loader = ash::khr::swapchain::Device::new(&runtime.instance, runtime.device());
        let sync = WindowSync::new(runtime)?;
        let handle = unsafe { loader.create_swapchain(&info, None) }
            .map_err(|e| vk_error("create window swapchain", e))?;
        let mut chain = Self {
            runtime: runtime.clone(),
            loader,
            handle,
            images: vec![],
            config,
            sync,
            acquired_index: None,
            stale: false,
        };
        chain.images = unsafe { chain.loader.get_swapchain_images(handle) }
            .map_err(|e| vk_error("read swapchain images", e))?;
        Ok(chain)
    }
    pub fn acquire(&mut self) -> Result<u32, vk::Result> {
        let (index, suboptimal) = unsafe {
            self.loader.acquire_next_image(
                self.handle,
                1_000_000_000,
                self.sync.acquired,
                self.sync.acquire_fence,
            )
        }?;
        self.acquired_index = Some(index);
        self.stale = suboptimal;
        Ok(index)
    }
    pub fn present(&mut self, queue: vk::Queue, index: u32) -> Result<(), RenderError> {
        let waits = [self.sync.rendered];
        let swaps = [self.handle];
        let indices = [index];
        let fences = [self.sync.present_fence];
        let mut fence_info = vk::SwapchainPresentFenceInfoEXT::default().fences(&fences);
        let info = vk::PresentInfoKHR::default()
            .wait_semaphores(&waits)
            .swapchains(&swaps)
            .image_indices(&indices)
            .push_next(&mut fence_info);
        let result = unsafe { self.loader.queue_present(queue, &info) };
        self.sync.pending = present_was_enqueued(result);
        if self.sync.pending {
            self.acquired_index = None;
        }
        match result {
            Ok(suboptimal) => {
                self.stale |= suboptimal;
                Ok(())
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.stale = true;
                Ok(())
            }
            Err(e) => Err(vk_error("present window frame", e)),
        }
    }
}
impl Drop for WindowChain {
    fn drop(&mut self) {
        let _ = self.sync.wait_present();
        unsafe {
            let _ = self.runtime.device().device_wait_idle();
            if let Some(index) = self.acquired_index.take() {
                let loader = ash::ext::swapchain_maintenance1::Device::new(
                    &self.runtime.instance,
                    self.runtime.device(),
                );
                let indices = [index];
                let info = vk::ReleaseSwapchainImagesInfoEXT::default()
                    .swapchain(self.handle)
                    .image_indices(&indices);
                let _ = loader.release_swapchain_images(&info);
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
fn validate_blit(runtime: &VulkanRuntime, destination: vk::Format) -> Result<(), RenderError> {
    for (format, required) in [
        (vk::Format::R8G8B8A8_SRGB, vk::FormatFeatureFlags::BLIT_SRC),
        (destination, vk::FormatFeatureFlags::BLIT_DST),
    ] {
        let actual = unsafe {
            runtime
                .instance
                .get_physical_device_format_properties(runtime.physical, format)
        }
        .optimal_tiling_features;
        if !actual.contains(required) {
            return Err(rejected(
                (format, actual),
                &format!("{required:?} format support"),
            ));
        }
    }
    Ok(())
}
fn present_was_enqueued(result: Result<bool, vk::Result>) -> bool {
    matches!(
        result,
        Ok(_)
            | Err(vk::Result::ERROR_OUT_OF_DATE_KHR
                | vk::Result::ERROR_SURFACE_LOST_KHR
                | vk::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT)
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_surface_presentations_still_own_the_fence() {
        for result in [
            Ok(false),
            Ok(true),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR),
            Err(vk::Result::ERROR_SURFACE_LOST_KHR),
        ] {
            assert!(present_was_enqueued(result));
        }
        for result in [
            vk::Result::ERROR_OUT_OF_HOST_MEMORY,
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY,
            vk::Result::ERROR_DEVICE_LOST,
        ] {
            assert!(!present_was_enqueued(Err(result)));
        }
    }
}
