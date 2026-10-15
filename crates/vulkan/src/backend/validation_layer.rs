//! Loader-selected validation-layer provenance keeps renderer gates independent of host packages.
use crate::error::{RenderError, rejected};
use ash::vk;

/// Khronos validation layer version reported by the Vulkan loader.
///
/// Example: `ValidationLayerVersion::from_spec(vk::make_api_version(0, 1, 4, 313))`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidationLayerVersion {
    /// Major version.
    pub major: u32,
    /// Minor version.
    pub minor: u32,
    /// Patch (SDK header) version.
    pub patch: u32,
}

/// The checksum-pinned layer (`tools/vulkan_validation_cache.py`) that passed parallel replay.
///
/// Ubuntu 24.04's 1.3.275 validates dynamic-rendering depth resolves with color-only ordering
/// (a false SYNC-HAZARD-READ-AFTER-WRITE against the same instance's depth load) and crashes in
/// parallel multi-instance tests; see docs/vulkan-renderer.md#linux-validation-dependency.
pub const VERIFIED_VALIDATION_LAYER: ValidationLayerVersion = ValidationLayerVersion {
    major: 1,
    minor: 4,
    patch: 313,
};

impl ValidationLayerVersion {
    /// Decode a layer `specVersion`; `ValidationLayerVersion::from_spec(spec).patch`.
    pub fn from_spec(spec: u32) -> Self {
        Self {
            major: vk::api_version_major(spec),
            minor: vk::api_version_minor(spec),
            patch: vk::api_version_patch(spec),
        }
    }
}

/// Read the Khronos validation layer the loader would enable, if any.
///
/// Example: `assert!(active_validation_layer()? >= Some(VERIFIED_VALIDATION_LAYER))`.
pub fn active_validation_layer() -> Result<Option<ValidationLayerVersion>, RenderError> {
    let entry =
        unsafe { ash::Entry::load() }.map_err(|e| rejected(e, "a loadable Vulkan 1.3 runtime"))?;
    let layers = unsafe { entry.enumerate_instance_layer_properties() }
        .map_err(|e| rejected(e, "enumerable Vulkan instance layers"))?;
    Ok(layers
        .iter()
        .find(|layer| layer.layer_name_as_c_str() == Ok(c"VK_LAYER_KHRONOS_validation"))
        .map(|layer| ValidationLayerVersion::from_spec(layer.spec_version)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec_versions_order_the_legacy_layer_below_the_verified_dependency() {
        let legacy = ValidationLayerVersion::from_spec(vk::make_api_version(0, 1, 3, 275));
        let pinned = ValidationLayerVersion::from_spec(vk::make_api_version(0, 1, 4, 313));
        assert_eq!((legacy.major, legacy.minor, legacy.patch), (1, 3, 275));
        assert_eq!(pinned, VERIFIED_VALIDATION_LAYER);
        assert!(legacy < VERIFIED_VALIDATION_LAYER);
    }
}
