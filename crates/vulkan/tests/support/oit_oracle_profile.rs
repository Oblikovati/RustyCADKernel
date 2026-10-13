//! Accepted OIT assets were independently reproduced on both proved Mesa software cohorts.
use super::ffp_oracle_profile::{FfpNativeOracleProfile, ffp_native_oracle_profile};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;

/// Select independently proved software buffers using the actual device; `software_oracle(&driver)`.
pub fn software_oracle(driver: &Handle<Graphic3dRenderDriver>) -> bool {
    let identity = driver
        .driver_identity()
        .expect("expected selected Vulkan physical-device identity");
    matches!(
        ffp_native_oracle_profile(&identity).unwrap(),
        FfpNativeOracleProfile::SoftwareMesa25 | FfpNativeOracleProfile::SoftwareMesa26
    )
}
