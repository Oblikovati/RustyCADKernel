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
    pub(super) entry: Entry,
    pub window_support: bool,
    pub displays: rustycad_tkernel::standard::SharedState<
        Vec<
            rustycad_tkernel::standard::Handle<rustycad_tkservice::aspect::AspectDisplayConnection>,
        >,
    >,
    pub queues: Vec<vk::Queue>,
    pub instance: Instance,
    messenger: Option<ValidationMessenger>,
    device: Option<Device>,
    pub validation: Box<ValidationMessages>,
    pub physical: vk::PhysicalDevice,
    pub queue: vk::Queue,
    pub queue_family: u32,
    pub memory: vk::PhysicalDeviceMemoryProperties,
    pub limits: vk::PhysicalDeviceLimits,
    pub depth_clamp: bool,
    pub logic_op: bool,
    pub geometry_shader: bool,
    pub polygon_lines: bool,
    pub large_points: bool,
    pub sampler_anisotropy: bool,
    pub sample_rate_shading: bool,
    pub depth_resolve_modes: vk::ResolveModeFlags,
    pub stencil_resolve_modes: vk::ResolveModeFlags,
    pub lines: super::line_device::LineCapabilities,
    pub line_stipple: Option<ash::ext::line_rasterization::Device>,
    pub name: String,
    pub quad_diagonal: crate::quad_assembly::QuadDiagonal,
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
        let window_support = !super::window_support::instance_extensions(&entry)?.is_empty();
        let instance = create_instance(&entry, enable_validation)?;
        Ok(Self {
            entry,
            window_support,
            queues: vec![],
            displays: Default::default(),
            instance,
            messenger: None,
            device: None,
            validation: Box::default(),
            physical: vk::PhysicalDevice::null(),
            queue: vk::Queue::null(),
            queue_family: 0,
            memory: Default::default(),
            limits: Default::default(),
            depth_clamp: false,
            logic_op: false,
            geometry_shader: false,
            polygon_lines: false,
            large_points: false,
            sampler_anisotropy: false,
            sample_rate_shading: false,
            depth_resolve_modes: vk::ResolveModeFlags::empty(),
            stencil_resolve_modes: vk::ResolveModeFlags::empty(),
            lines: Default::default(),
            line_stipple: None,
            name: String::new(),
            quad_diagonal: Default::default(),
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
        self.quad_diagonal = crate::quad_assembly::QuadDiagonal::for_vendor(properties.vendor_id);
        self.depth_clamp =
            unsafe { self.instance.get_physical_device_features(physical) }.depth_clamp == vk::TRUE;
        self.large_points = unsafe { self.instance.get_physical_device_features(physical) }
            .large_points
            == vk::TRUE;
        self.sampler_anisotropy = unsafe { self.instance.get_physical_device_features(physical) }
            .sampler_anisotropy
            == vk::TRUE;
        self.logic_op =
            unsafe { self.instance.get_physical_device_features(physical) }.logic_op == vk::TRUE;
        let native_features = unsafe { self.instance.get_physical_device_features(physical) };
        self.sample_rate_shading = native_features.sample_rate_shading == vk::TRUE;
        let mut resolve = vk::PhysicalDeviceDepthStencilResolveProperties::default();
        let mut query = vk::PhysicalDeviceProperties2::default().push_next(&mut resolve);
        unsafe {
            self.instance
                .get_physical_device_properties2(physical, &mut query);
        }
        self.depth_resolve_modes = resolve.supported_depth_resolve_modes;
        self.stencil_resolve_modes = resolve.supported_stencil_resolve_modes;
        self.geometry_shader = native_features.geometry_shader == vk::TRUE;
        self.polygon_lines = native_features.fill_mode_non_solid == vk::TRUE;
        self.lines = super::line_device::LineCapabilities::query(&self.instance, physical)?;
        self.memory = unsafe {
            self.instance
                .get_physical_device_memory_properties(physical)
        };
        self.window_support &= super::window_support::device_supported(&self.instance, physical)?;
        self.device = Some(create_device(
            &self.instance,
            physical,
            self.window_support,
            self.depth_clamp,
            self.large_points,
            self.lines,
            self.logic_op,
        )?);
        self.line_stipple = self
            .lines
            .stippled
            .then(|| ash::ext::line_rasterization::Device::new(&self.instance, self.device()));
        let families = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical)
        };
        self.queues = (0..families.len())
            .map(|i| unsafe { self.device().get_device_queue(i as u32, 0) })
            .collect();
        self.queue = self.queues[family as usize];
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
    let mut extensions = super::window_support::instance_extensions(entry)?;
    if validation {
        extensions.extend([
            ash::ext::debug_utils::NAME.as_ptr(),
            ash::ext::validation_features::NAME.as_ptr(),
        ]);
    }
    info = info.enabled_extension_names(&extensions);
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
    windows: bool,
    depth_clamp: bool,
    large_points: bool,
    lines: super::line_device::LineCapabilities,
    logic_op: bool,
) -> Result<Device, RenderError> {
    let families = unsafe { instance.get_physical_device_queue_family_properties(physical) };
    let queues: Vec<_> = (0..families.len())
        .map(|i| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(i as u32)
                .queue_priorities(&[1.0])
        })
        .collect();
    let mut features = vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .synchronization2(true);
    let supported = unsafe { instance.get_physical_device_features(physical) };
    let core_features = vk::PhysicalDeviceFeatures::default()
        .sample_rate_shading(supported.sample_rate_shading == vk::TRUE)
        .sampler_anisotropy(supported.sampler_anisotropy == vk::TRUE)
        .texture_compression_bc(supported.texture_compression_bc == vk::TRUE)
        .depth_clamp(depth_clamp)
        .large_points(large_points)
        .wide_lines(lines.wide)
        .logic_op(logic_op)
        .geometry_shader(supported.geometry_shader == vk::TRUE)
        .fill_mode_non_solid(supported.fill_mode_non_solid == vk::TRUE);
    let mut info = vk::DeviceCreateInfo::default()
        .enabled_features(&core_features)
        .queue_create_infos(&queues)
        .push_next(&mut features);
    let mut extensions = vec![];
    if lines.bresenham {
        extensions.push(ash::ext::line_rasterization::NAME.as_ptr());
    }
    let mut maintenance = vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default()
        .swapchain_maintenance1(windows);
    if windows {
        extensions.extend([
            ash::khr::swapchain::NAME.as_ptr(),
            ash::ext::swapchain_maintenance1::NAME.as_ptr(),
        ]);
        info = info.push_next(&mut maintenance);
    }
    info = info.enabled_extension_names(&extensions);
    let mut line_features = vk::PhysicalDeviceLineRasterizationFeaturesEXT::default()
        .bresenham_lines(lines.bresenham)
        .stippled_bresenham_lines(lines.stippled);
    if lines.bresenham {
        info = info
            .enabled_extension_names(&extensions)
            .push_next(&mut line_features);
    }
    unsafe { instance.create_device(physical, &info, None) }
        .map_err(|e| vk_error("create device", e))
}
