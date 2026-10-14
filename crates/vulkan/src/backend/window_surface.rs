//! Own the native window and display until every swapchain and Vulkan surface is destroyed.
use super::{
    runtime::{VulkanRuntime, vk_error},
    window_chain::{WindowChain, WindowConfig},
};
use crate::{RenderError, error::rejected};
use ash::vk;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::{AspectWindow, AspectWindowMembers};
use std::sync::Arc;
pub(crate) struct WindowPresenter {
    runtime: Arc<VulkanRuntime>,
    surface: ash::khr::surface::Instance,
    handle: vk::SurfaceKHR,
    pub(super) chain: Option<WindowChain>,
    pub(super) present_family: u32,
    _window: Handle<dyn AspectWindow>,
}
impl WindowPresenter {
    pub(super) fn new(
        runtime: &Arc<VulkanRuntime>,
        window: &Handle<dyn AspectWindow>,
    ) -> Result<Self, RenderError> {
        let handle = create_window_surface(runtime, window)?;
        let mut presenter = Self::with_surface(runtime, window, handle);
        presenter.present_family = presenter.choose_present_family()?;
        Ok(presenter)
    }
    fn with_surface(
        runtime: &Arc<VulkanRuntime>,
        window: &Handle<dyn AspectWindow>,
        handle: vk::SurfaceKHR,
    ) -> Self {
        Self {
            runtime: runtime.clone(),
            surface: ash::khr::surface::Instance::new(&runtime.entry, &runtime.instance),
            handle,
            chain: None,
            present_family: 0,
            _window: window.clone(),
        }
    }
    fn choose_present_family(&self) -> Result<u32, RenderError> {
        let order =
            std::iter::once(self.runtime.queue_family).chain(0..self.runtime.queues.len() as u32);
        for family in order {
            if unsafe {
                self.surface.get_physical_device_surface_support(
                    self.runtime.physical,
                    family,
                    self.handle,
                )
            }
            .map_err(|e| vk_error("query presentation queue", e))?
            {
                return Ok(family);
            }
        }
        Err(rejected(
            self.runtime.name.as_str(),
            "a queue supporting the selected native window",
        ))
    }
    pub(super) fn acquire(&mut self, extent: [u32; 2], vsync: bool) -> Result<u32, RenderError> {
        let config = WindowConfig::query(&self.runtime, &self.surface, self.handle, extent, vsync)?;
        if let Some(chain) = self.chain.as_mut() {
            chain.sync.wait_present()?;
        }
        if self
            .chain
            .as_ref()
            .is_none_or(|c| c.stale || c.config != config)
        {
            self.recreate(config)?;
        }
        let result = self.chain.as_mut().unwrap().acquire();
        let index = match result {
            Ok(index) => index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate(WindowConfig::query(
                    &self.runtime,
                    &self.surface,
                    self.handle,
                    extent,
                    vsync,
                )?)?;
                self.chain
                    .as_mut()
                    .unwrap()
                    .acquire()
                    .map_err(|e| vk_error("reacquire resized window", e))?
            }
            Err(e) => return Err(vk_error("acquire window image", e)),
        };
        self.chain.as_ref().unwrap().sync.wait_acquire()?;
        Ok(index)
    }
    fn recreate(&mut self, config: WindowConfig) -> Result<(), RenderError> {
        let old = self.chain.take();
        let previous = old.as_ref().map_or(vk::SwapchainKHR::null(), |c| c.handle);
        self.chain = Some(WindowChain::new(
            &self.runtime,
            self.handle,
            self.present_family,
            config,
            previous,
        )?);
        drop(old);
        Ok(())
    }
}
impl Drop for WindowPresenter {
    fn drop(&mut self) {
        self.chain.take();
        unsafe { self.surface.destroy_surface(self.handle, None) };
    }
}

// Mesa WSI retention ends after instance teardown; a mutable Handle alone cannot pin it.
fn retain_runtime_display(
    runtime: &VulkanRuntime,
    display: &Handle<rustycad_tkservice::aspect::AspectDisplayConnection>,
) -> Result<usize, RenderError> {
    use rustycad_tkservice::aspect::{AspectDisplayConnection, AspectDisplayLeasePurpose};
    let lease =
        AspectDisplayConnection::acquire_lease(display, AspectDisplayLeasePurpose::VulkanInstance)?;
    let address = lease.display_address();
    let mut retained = runtime.displays.write();
    if !retained.iter().any(|saved| saved.same_connection(&lease)) {
        retained.push(lease);
    }
    Ok(address)
}

fn checked_native_display(
    runtime: &VulkanRuntime,
    window: &Handle<dyn AspectWindow>,
) -> Result<usize, RenderError> {
    let display = window.native_display_connection();
    if !runtime.window_support
        || display.is_null()
        || display.get_display_aspect() == 0
        || window.native_handle() == 0
    {
        return Err(rejected(
            (runtime.window_support, window.native_handle()),
            "a live Xlib window/display and Vulkan swapchain-maintenance support",
        ));
    }
    retain_runtime_display(runtime, &display)
}
fn create_window_surface(
    runtime: &VulkanRuntime,
    window: &Handle<dyn AspectWindow>,
) -> Result<vk::SurfaceKHR, RenderError> {
    let display_address = checked_native_display(runtime, window)?;
    let xlib = ash::khr::xlib_surface::Instance::new(&runtime.entry, &runtime.instance);
    // Vulkan XIDs use C unsigned long; reject narrowing on LLP64 hosts.
    #[allow(clippy::useless_conversion)]
    let native_window = vk::Window::try_from(window.native_handle())
        .map_err(|_| rejected(window.native_handle(), "an XID fitting C unsigned long"))?;
    let info = vk::XlibSurfaceCreateInfoKHR::default()
        .dpy(display_address as *mut vk::Display)
        .window(native_window);
    unsafe { xlib.create_xlib_surface(&info, None) }
        .map_err(|e| vk_error("create Xlib Vulkan surface", e))
}
