//! Each submission owns its command pool and fence and finishes before frame resources drop.
use super::runtime::{VulkanRuntime, vk_error};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;
pub(super) struct CommandSubmission {
    runtime: Arc<VulkanRuntime>,
    pool: vk::CommandPool,
    fence: vk::Fence,
    pub command: vk::CommandBuffer,
}
impl CommandSubmission {
    pub fn new(runtime: &Arc<VulkanRuntime>) -> Result<Self, RenderError> {
        let info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(runtime.queue_family)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let pool = unsafe { runtime.device().create_command_pool(&info, None) }
            .map_err(|e| vk_error("create command pool", e))?;
        let mut submission = Self {
            runtime: runtime.clone(),
            pool,
            fence: vk::Fence::null(),
            command: vk::CommandBuffer::null(),
        };
        submission.allocate()?;
        submission.begin()?;
        Ok(submission)
    }
    fn begin(&self) -> Result<(), RenderError> {
        let info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.runtime
                .device()
                .begin_command_buffer(self.command, &info)
        }
        .map_err(|e| vk_error("begin command buffer", e))
    }
    fn allocate(&mut self) -> Result<(), RenderError> {
        let info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        self.command = unsafe { self.runtime.device().allocate_command_buffers(&info) }
            .map_err(|e| vk_error("allocate command buffer", e))?[0];
        self.fence = unsafe {
            self.runtime
                .device()
                .create_fence(&vk::FenceCreateInfo::default(), None)
        }
        .map_err(|e| vk_error("create submit fence", e))?;
        Ok(())
    }
    pub fn finish(&self) -> Result<(), RenderError> {
        let device = self.runtime.device();
        unsafe { device.end_command_buffer(self.command) }
            .map_err(|e| vk_error("end command buffer", e))?;
        let commands = [vk::CommandBufferSubmitInfo::default().command_buffer(self.command)];
        let submits = [vk::SubmitInfo2::default().command_buffer_infos(&commands)];
        unsafe { device.queue_submit2(self.runtime.queue, &submits, self.fence) }
            .map_err(|e| vk_error("submit frame", e))?;
        unsafe { device.wait_for_fences(&[self.fence], true, u64::MAX) }
            .map_err(|e| vk_error("wait for readback", e))
    }
}
impl Drop for CommandSubmission {
    fn drop(&mut self) {
        unsafe {
            // Also covers a failed fence wait after a successful submission.
            let _ = self.runtime.device().device_wait_idle();
            if self.fence != vk::Fence::null() {
                self.runtime.device().destroy_fence(self.fence, None);
            }
            self.runtime.device().destroy_command_pool(self.pool, None);
        }
    }
}
