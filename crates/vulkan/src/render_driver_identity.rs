//! Owned Vulkan driver provenance; raw numeric versions remain vendor-defined.
/// Identify the selected implementation without guessing from a display name.
/// For example, `driver.driver_identity().unwrap().driver_version` is the exact Vulkan `u32`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderDriverIdentity {
    /// Native vendor identifier for comparing proven device cohorts.
    pub vendor_id: u32,
    /// Native device identifier; retain exact hardware identity.
    pub device_id: u32,
    /// Vulkan driver identifier, separate from its display name.
    pub driver_id: u32,
    /// Vendor-defined version bits; preserve rather than universally decoding them.
    pub driver_version: u32,
    /// Owned implementation name from the selected physical device.
    pub driver_name: String,
    /// Owned implementation build description for fixture provenance.
    pub driver_info: String,
    /// Owned device name for distinguishing hardware and software cohorts.
    pub device_name: String,
}
