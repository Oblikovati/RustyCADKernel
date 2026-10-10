//! Runtime ownership keeps the loader, instance and device alive until all resources drop.
use super::validation::{ValidationMessages, ValidationMessenger};
use crate::error::{RenderError, rejected};
use ash::{Device, Entry, Instance, vk};
use std::{ffi::CStr, sync::Arc};
/// Device creation policy; `VulkanOptions { validation: true, ..Default::default() }`.
#[derive(Clone, Copy, Debug, Default)]
pub struct VulkanOptions {
    /// Require the Khronos validation layer when enabled.
    pub validation: bool,
    /// Select a physical device by enumeration index, or the first capable device.
    pub device_index: Option<usize>,
}
pub(super) struct VulkanRuntime {
    entry: Entry,
    pub instance: Instance,
    messenger: Option<ValidationMessenger>,
    device: Option<Device>,
    pub validation: Box<ValidationMessages>,
    pub physical: vk::PhysicalDevice,
    pub queue: vk::Queue,
    pub queue_family: u32,
    pub memory: vk::PhysicalDeviceMemoryProperties,
    pub limits: vk::PhysicalDeviceLimits,
    pub name: String,
}
impl VulkanRuntime {
    pub fn create(options: VulkanOptions) -> Result<Arc<Self>, RenderError> {
        let mut runtime = Self::load(options.validation)?;
        if options.validation {
            runtime.messenger = Some(
                ValidationMessenger::new(&runtime.entry, &runtime.instance, &runtime.validation)
                    .map_err(|e| vk_error("create validation messenger", e))?,
            );
        }
        runtime.initialize_device(options.device_index)?;
        Ok(Arc::new(runtime))
    }
    fn load(enable_validation: bool) -> Result<Self, RenderError> {
        let entry =
            unsafe { Entry::load() }.map_err(|e| rejected(e, "a loadable Vulkan 1.3 runtime"))?;
        let instance = create_instance(&entry, enable_validation)?;
        Ok(Self {
            entry,
            instance,
            messenger: None,
            device: None,
            validation: Box::default(),
            physical: vk::PhysicalDevice::null(),
            queue: vk::Queue::null(),
            queue_family: 0,
            memory: Default::default(),
            limits: Default::default(),
            name: String::new(),
        })
    }
    pub fn device(&self) -> &Device {
        self.device.as_ref().expect("initialized Vulkan device")
    }
    fn initialize_device(&mut self, index: Option<usize>) -> Result<(), RenderError> {
        let (physical, family) = choose_device(&self.instance, index)?;
        let properties = unsafe { self.instance.get_physical_device_properties(physical) };
        self.name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        self.limits = properties.limits;
        self.memory = unsafe {
            self.instance
                .get_physical_device_memory_properties(physical)
        };
        self.device = Some(create_device(&self.instance, physical, family)?);
        self.queue = unsafe { self.device().get_device_queue(family, 0) };
        self.physical = physical;
        self.queue_family = family;
        Ok(())
    }
    pub fn memory_type(
        &self,
        bits: u32,
        flags: vk::MemoryPropertyFlags,
    ) -> Result<u32, RenderError> {
        super::memory_selection::compatible_memory_type(&self.memory, bits, flags)
            .ok_or_else(|| rejected((bits, flags), "a compatible Vulkan memory type"))
    }
}
impl Drop for VulkanRuntime {
    fn drop(&mut self) {
        if let Some(device) = self.device.take() {
            // Renderer submissions complete before returning; idle also guards failure cleanup.
            unsafe {
                let _ = device.device_wait_idle();
                device.destroy_device(None);
            }
        }
        self.messenger.take();
        unsafe { self.instance.destroy_instance(None) };
    }
}
fn create_instance(entry: &Entry, validation: bool) -> Result<Instance, RenderError> {
    let app = vk::ApplicationInfo::default()
        .application_name(c"Martensite")
        .api_version(vk::API_VERSION_1_3);
    let mut info = vk::InstanceCreateInfo::default().application_info(&app);
    let layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
    let extensions = [
        ash::ext::debug_utils::NAME.as_ptr(),
        ash::ext::validation_features::NAME.as_ptr(),
    ];
    let mut features = vk::ValidationFeaturesEXT::default()
        .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION]);
    if validation {
        info = info
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .push_next(&mut features);
    }
    unsafe { entry.create_instance(&info, None) }
        .map_err(|e| vk_error("create Vulkan 1.3 instance", e))
}
fn choose_device(
    instance: &Instance,
    selected: Option<usize>,
) -> Result<(vk::PhysicalDevice, u32), RenderError> {
    let devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|e| vk_error("enumerate devices", e))?;
    for (index, physical) in devices.iter().enumerate() {
        if selected.is_some_and(|wanted| wanted != index) {
            continue;
        }
        if let Some(family) = capable_queue(instance, *physical) {
            return Ok((*physical, family));
        }
    }
    Err(rejected(
        selected,
        "a Vulkan 1.3 graphics device with dynamic rendering and synchronization2",
    ))
}
fn capable_queue(instance: &Instance, physical: vk::PhysicalDevice) -> Option<u32> {
    let properties = unsafe { instance.get_physical_device_properties(physical) };
    if properties.api_version < vk::API_VERSION_1_3 {
        return None;
    }
    let mut features = vk::PhysicalDeviceVulkan13Features::default();
    let mut query = vk::PhysicalDeviceFeatures2::default().push_next(&mut features);
    unsafe { instance.get_physical_device_features2(physical, &mut query) };
    if features.dynamic_rendering == vk::FALSE || features.synchronization2 == vk::FALSE {
        return None;
    }
    unsafe { instance.get_physical_device_queue_family_properties(physical) }
        .iter()
        .position(|family| {
            family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .map(|index| index as u32)
}
pub(super) fn vk_error(operation: &str, error: vk::Result) -> RenderError {
    rejected(format!("{operation}: {error:?}"), "VK_SUCCESS")
}

fn create_device(
    instance: &Instance,
    physical: vk::PhysicalDevice,
    family: u32,
) -> Result<Device, RenderError> {
    let queues = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(family)
        .queue_priorities(&[1.0])];
    let mut features = vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .synchronization2(true);
    let info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queues)
        .push_next(&mut features);
    unsafe { instance.create_device(physical, &info, None) }
        .map_err(|e| vk_error("create device", e))
}
