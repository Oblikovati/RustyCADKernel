//! Query the selected retained instance/device; no loader subprocess or global policy.
use crate::RenderDriverIdentity;
use ash::{Instance, vk};
use std::ffi::CStr;

pub(super) fn query_driver_identity(
    instance: &Instance,
    physical: vk::PhysicalDevice,
) -> RenderDriverIdentity {
    let mut driver = vk::PhysicalDeviceDriverProperties::default();
    let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut driver);
    // Both output structures and their p_next link stay live throughout this query.
    unsafe { instance.get_physical_device_properties2(physical, &mut properties) };
    owned_driver_identity(&properties.properties, &driver)
}
fn owned_driver_identity(
    physical: &vk::PhysicalDeviceProperties,
    driver: &vk::PhysicalDeviceDriverProperties<'_>,
) -> RenderDriverIdentity {
    RenderDriverIdentity {
        vendor_id: physical.vendor_id,
        device_id: physical.device_id,
        driver_id: driver.driver_id.as_raw() as u32,
        driver_version: physical.driver_version,
        driver_name: property_string(&driver.driver_name),
        driver_info: property_string(&driver.driver_info),
        device_name: property_string(&physical.device_name),
    }
}
fn property_string(bytes: &[std::ffi::c_char]) -> String {
    // Vulkan guarantees these property arrays have a null terminator.
    unsafe { CStr::from_ptr(bytes.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queried_properties_keep_vendor_defined_raw_versions() {
        let physical = vk::PhysicalDeviceProperties {
            vendor_id: 7,
            device_id: 8,
            driver_version: u32::MAX,
            ..Default::default()
        };
        let driver = vk::PhysicalDeviceDriverProperties {
            driver_id: vk::DriverId::MESA_LLVMPIPE,
            ..Default::default()
        };
        let owned = owned_driver_identity(&physical, &driver);
        assert_eq!(owned.vendor_id, 7);
        assert_eq!(owned.device_id, 8);
        assert_eq!(owned.driver_version, u32::MAX);
        assert_eq!(owned.driver_id, 13);
        assert_eq!(owned.device_name, "");
    }
    #[test]
    fn property_strings_are_owned_and_stop_at_null() {
        let mut characters = [65, 0, 66];
        let owned = property_string(&characters);
        characters[0] = 67;
        assert_eq!(property_string(&characters), "C");
        assert_eq!(owned, "A");
    }
}
