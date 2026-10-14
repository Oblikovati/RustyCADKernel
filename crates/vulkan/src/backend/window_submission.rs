//! Native presentation waits for acquisition before copying the completed model-plus-GUI frame.
use super::{submission::CommandSubmission, window_chain::WindowChain};
use crate::RenderError;
pub(super) fn finish(commands: &CommandSubmission, chain: &WindowChain) -> Result<(), RenderError> {
    let waits = [ash::vk::SemaphoreSubmitInfo::default()
        .semaphore(chain.sync.acquired)
        .stage_mask(ash::vk::PipelineStageFlags2::TRANSFER)];
    let signals = [ash::vk::SemaphoreSubmitInfo::default()
        .semaphore(chain.sync.rendered)
        .stage_mask(ash::vk::PipelineStageFlags2::ALL_COMMANDS)];
    commands.finish_synced(&waits, &signals)
}
