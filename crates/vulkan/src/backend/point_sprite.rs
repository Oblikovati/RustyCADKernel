//! Point sprites retain native size limits and immutable nearest-filtered texels.
use super::{
    runtime::VulkanRuntime,
    storage_descriptor::{StorageDescriptorLayout, UploadedStorage},
};
use crate::{RasterImage, RenderError, error::rejected};
use std::sync::Arc;
pub(super) fn upload_sprite(
    runtime: &Arc<VulkanRuntime>,
    layout: &StorageDescriptorLayout,
    image: &RasterImage,
) -> Result<UploadedStorage, RenderError> {
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flatten()
        .flat_map(|v| v.to_ne_bytes())
        .collect();
    UploadedStorage::new(runtime, layout, &bytes)
}
pub(super) fn validate_point_size(
    size: f32,
    large: bool,
    range: [f32; 2],
) -> Result<(), RenderError> {
    if (!large && size != 1.) || !(range[0]..=range[1]).contains(&size) {
        return Err(rejected(
            (size, large, range),
            "point size within enabled device capabilities",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn point_sizes_respect_feature_and_range() {
        assert!(validate_point_size(1., false, [1., 64.]).is_ok());
        assert!(validate_point_size(3., false, [1., 64.]).is_err());
        assert!(validate_point_size(3., true, [1., 64.]).is_ok());
        assert!(validate_point_size(65., true, [1., 64.]).is_err());
    }
}
