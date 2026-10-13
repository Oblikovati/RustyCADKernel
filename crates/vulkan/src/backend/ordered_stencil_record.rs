//! Vulkan1.3 dynamic stencil state follows executed native commands, including inherited operations.
use super::runtime::VulkanRuntime;
use crate::{
    RenderStencilState,
    stencil_context::{StencilCompare, StencilOperation},
};
use ash::vk;
const STENCIL_FACES: vk::StencilFaceFlags = vk::StencilFaceFlags::FRONT_AND_BACK;
pub(super) const COMPOSITE_DYNAMIC: &[vk::DynamicState] = &[
    vk::DynamicState::VIEWPORT,
    vk::DynamicState::SCISSOR,
    vk::DynamicState::STENCIL_TEST_ENABLE,
    vk::DynamicState::STENCIL_OP,
    vk::DynamicState::STENCIL_COMPARE_MASK,
    vk::DynamicState::STENCIL_WRITE_MASK,
    vk::DynamicState::STENCIL_REFERENCE,
];
pub(super) fn apply(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    state: RenderStencilState,
) {
    let device = runtime.device();
    let [fail, depth_fail, pass] = state.operations.map(operation);
    unsafe {
        device.cmd_set_stencil_test_enable(command, state.enabled);
        device.cmd_set_stencil_op(
            command,
            STENCIL_FACES,
            fail,
            pass,
            depth_fail,
            comparison(state.compare),
        );
    }
    apply_stencil_masks(runtime, command, state);
}
fn apply_stencil_masks(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    state: RenderStencilState,
) {
    let device = runtime.device();
    unsafe {
        device.cmd_set_stencil_compare_mask(command, STENCIL_FACES, state.compare_mask);
        device.cmd_set_stencil_write_mask(command, STENCIL_FACES, state.write_mask);
        device.cmd_set_stencil_reference(command, STENCIL_FACES, state.reference);
    }
}
pub(super) fn finish_layer(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &super::frame::FrameResources,
    index: usize,
) {
    if resources.stencil.clear_after_layer[index] {
        super::capping_record::clear_stencil(runtime, command, resources.extent);
    }
    apply(runtime, command, resources.stencil.after_layer[index]);
}
fn comparison(compare: StencilCompare) -> vk::CompareOp {
    match compare {
        StencilCompare::Always => vk::CompareOp::ALWAYS,
        StencilCompare::NotEqual => vk::CompareOp::NOT_EQUAL,
        StencilCompare::Equal => vk::CompareOp::EQUAL,
    }
}
fn operation(op: StencilOperation) -> vk::StencilOp {
    match op {
        StencilOperation::Keep => vk::StencilOp::KEEP,
        StencilOperation::Invert => vk::StencilOp::INVERT,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_comparisons_operations_and_composite_dynamic_state_are_exact() {
        assert_eq!(comparison(StencilCompare::Always), vk::CompareOp::ALWAYS);
        assert_eq!(comparison(StencilCompare::Equal), vk::CompareOp::EQUAL);
        assert_eq!(
            comparison(StencilCompare::NotEqual),
            vk::CompareOp::NOT_EQUAL
        );
        assert_eq!(operation(StencilOperation::Keep), vk::StencilOp::KEEP);
        assert_eq!(operation(StencilOperation::Invert), vk::StencilOp::INVERT);
        assert!(COMPOSITE_DYNAMIC.contains(&vk::DynamicState::STENCIL_TEST_ENABLE));
        assert!(COMPOSITE_DYNAMIC.contains(&vk::DynamicState::STENCIL_WRITE_MASK));
    }
}
