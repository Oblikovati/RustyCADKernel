//! Functional integer-XOR boundaries use strict unchanged color/depth gates.
#[path = "support/blend_boundary_runtime.rs"]
mod blend_boundary_runtime;
#[allow(dead_code)]
#[path = "support/capping_runtime.rs"]
mod capping_runtime;
#[path = "support/device_options.rs"]
mod device_options;
use martensite_vulkan::*;
fn main() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    blend_boundary_runtime::check_blend_boundaries(&mut renderer);
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    println!("PASS: XOR cancellation, ordinary/lit/atlas and layered depth LOAD boundaries");
}
