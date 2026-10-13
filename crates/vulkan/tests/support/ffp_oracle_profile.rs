//! Select independently captured native fixtures for the complete affected FFP family.
use martensite_vulkan::{Graphic3dRenderDriver, RenderDriverIdentity};
use rustycad_tkernel::standard::Handle;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfpNativeOracleProfile {
    SoftwareMesa25,
    SoftwareMesa26,
    NativeAmdMesa26,
}
type FfpOracleBufferPair = (&'static [u8], &'static [u8]);
const MESA26_VERSION: u32 = (26 << 22) | (1 << 12) | 6;
const MESA25_VERSION: u32 = (25 << 22) | (2 << 12) | 8;
const MESA26_INFO: &str = "Mesa 26.1.6-1pop0~1787580452~24.04~a5619ea";

const PROVEN_COHORTS: &[(FfpNativeOracleProfile, u32, &str)] = &[
    (
        FfpNativeOracleProfile::SoftwareMesa25,
        MESA25_VERSION,
        "Mesa 25.2.8-0ubuntu0.24.04.4 (LLVM 20.1.2)",
    ),
    (
        FfpNativeOracleProfile::SoftwareMesa26,
        MESA26_VERSION,
        "Mesa 26.1.6-1pop0~1787580452~24.04~a5619ea (LLVM 20.1.2)",
    ),
    (
        FfpNativeOracleProfile::NativeAmdMesa26,
        MESA26_VERSION,
        MESA26_INFO,
    ),
];
/// Select only proved native capture cohorts; `ffp_native_oracle_profile(&identity)?`.
pub fn ffp_native_oracle_profile(
    identity: &RenderDriverIdentity,
) -> Result<FfpNativeOracleProfile, String> {
    for &(profile, version, info) in PROVEN_COHORTS {
        if identity.driver_version == version
            && identity.driver_info == info
            && verified_profile_device(identity, profile)
        {
            return Ok(profile);
        }
    }
    Err(format!(
        "received driver identity={identity:?}; expected proved llvmpipe Mesa25.2.8 Ubuntu4 LLVM20.1.2/128-bit or Mesa26.1.6 Pop LLVM20.1.2/128-or-256-bit, or native AMD8060S RADV Mesa26.1.6 Pop"
    ))
}
fn verified_profile_device(
    identity: &RenderDriverIdentity,
    profile: FfpNativeOracleProfile,
) -> bool {
    match profile {
        FfpNativeOracleProfile::NativeAmdMesa26 => verified_native_amd_profile(identity),
        _ => verified_software_profile(identity, profile),
    }
}
fn verified_software_profile(
    identity: &RenderDriverIdentity,
    profile: FfpNativeOracleProfile,
) -> bool {
    let known_width = identity.device_name == "llvmpipe (LLVM 20.1.2, 128 bits)"
        || (profile == FfpNativeOracleProfile::SoftwareMesa26
            && identity.device_name == "llvmpipe (LLVM 20.1.2, 256 bits)");
    identity.vendor_id == 0x10005
        && identity.device_id == 0
        && identity.driver_id == 13
        && identity.driver_name == "llvmpipe"
        && known_width
}
fn verified_native_amd_profile(identity: &RenderDriverIdentity) -> bool {
    identity.vendor_id == 0x1002
        && identity.device_id == 0x1586
        && identity.driver_id == 3
        && identity.driver_version == MESA26_VERSION
        && identity.driver_name == "radv"
        && identity.driver_info == MESA26_INFO
        && identity.device_name == "AMD Radeon 8060S Graphics (RADV STRIX_HALO)"
}

#[path = "ffp_mesa25_asset_catalog.rs"]
mod mesa25_catalog;
/// Return only independently proved native assets; `mesa25_ffp_asset("bilinear")`.
pub fn mesa25_ffp_asset(mode: &str) -> Option<FfpOracleBufferPair> {
    mesa25_catalog::find_mesa25_asset(mode)
}
/// Preserve existing native26/AMD assets, selecting Mesa25 only for independently captured cases; `profiled_ffp_asset(&identity, mode, host, software)?`.
pub fn profiled_ffp_asset(
    identity: &RenderDriverIdentity,
    mode: &str,
    host: FfpOracleBufferPair,
    software: FfpOracleBufferPair,
) -> Result<Option<FfpOracleBufferPair>, String> {
    let Some(mesa25) = mesa25_ffp_asset(mode) else {
        return Ok(None);
    };
    let asset = match ffp_native_oracle_profile(identity)? {
        FfpNativeOracleProfile::SoftwareMesa25 => mesa25,
        FfpNativeOracleProfile::SoftwareMesa26 => software,
        FfpNativeOracleProfile::NativeAmdMesa26 => host,
    };
    Ok(Some(asset))
}
/// Select from actual driver provenance; `profiled_driver_asset(driver, mode, host, software)`.
pub fn profiled_driver_asset(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
    host: FfpOracleBufferPair,
    software: FfpOracleBufferPair,
) -> Option<FfpOracleBufferPair> {
    mesa25_ffp_asset(mode)?;
    let identity = driver
        .driver_identity()
        .expect("expected actual Vulkan driver identity for FFP asset");
    profiled_ffp_asset(&identity, mode, host, software).unwrap()
}
#[cfg(test)]
#[path = "ffp_oracle_profile_tests.rs"]
mod tests;
