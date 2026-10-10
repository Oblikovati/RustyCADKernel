//! Presentation fences protect semaphore reuse and swapchain destruction (VK_EXT_swapchain_maintenance1).
use super::runtime::{VulkanRuntime, vk_error};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;
pub(super) struct WindowSync {
    runtime: Arc<VulkanRuntime>,
    pub acquired: vk::Semaphore,
    pub rendered: vk::Semaphore,
    pub acquire_fence: vk::Fence,
    pub present_fence: vk::Fence,
    pub pending: bool,
}
impl WindowSync {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let mut sync = Self {
            runtime: runtime.clone(),
            acquired: vk::Semaphore::null(),
            rendered: vk::Semaphore::null(),
            acquire_fence: vk::Fence::null(),
            present_fence: vk::Fence::null(),
            pending: false,
        };
        let device = runtime.device();
        sync.acquired =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                .map_err(|e| vk_error("create acquire semaphore", e))?;
        sync.rendered =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                .map_err(|e| vk_error("create present semaphore", e))?;
        sync.acquire_fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }
            .map_err(|e| vk_error("create acquisition fence", e))?;
        sync.present_fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }
            .map_err(|e| vk_error("create presentation fence", e))?;
        Ok(sync)
    }
    pub fn wait_present(&mut self) -> Result<(), RenderError> {
        if !self.pending {
            return Ok(());
        }
        unsafe {
            self.runtime
                .device()
                .wait_for_fences(&[self.present_fence], true, u64::MAX)
        }
        .map_err(|e| vk_error("wait for presentation resources", e))?;
        self.pending = false;
        unsafe { self.runtime.device().reset_fences(&[self.present_fence]) }
            .map_err(|e| vk_error("reset presentation fence", e))
    }
    pub fn wait_acquire(&self) -> Result<(), RenderError> {
        unsafe {
            self.runtime
                .device()
                .wait_for_fences(&[self.acquire_fence], true, u64::MAX)
        }
        .map_err(|e| vk_error("wait for window acquisition", e))?;
        unsafe { self.runtime.device().reset_fences(&[self.acquire_fence]) }
            .map_err(|e| vk_error("reset acquisition fence", e))
    }
}
impl Drop for WindowSync {
    fn drop(&mut self) {
        let _ = self.wait_present();
        unsafe {
            let _ = self.runtime.device().device_wait_idle();
            for fence in [self.acquire_fence, self.present_fence] {
                if fence != vk::Fence::null() {
                    self.runtime.device().destroy_fence(fence, None);
                }
            }
            for semaphore in [self.acquired, self.rendered] {
                if semaphore != vk::Semaphore::null() {
                    self.runtime.device().destroy_semaphore(semaphore, None);
                }
            }
        }
    }
}
