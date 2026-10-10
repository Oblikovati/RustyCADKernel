//! Test processes receive an explicit enumeration index from the profile harness.
use martensite_vulkan::VulkanOptions;
pub fn validation_options() -> VulkanOptions {
    let device_index = std::env::var("MARTENSITE_VULKAN_DEVICE").ok().map(|value| {
        value
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("received {value:?}; expected a Vulkan device index"))
    });
    VulkanOptions {
        validation: true,
        device_index,
    }
}
