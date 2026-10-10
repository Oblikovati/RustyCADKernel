//! Validation callback storage outlives the messenger and never prints from driver threads.
use ash::{ext::debug_utils, vk};
use std::{
    ffi::{CStr, c_void},
    sync::Mutex,
};
#[derive(Default)]
pub(super) struct ValidationMessages {
    errors: Mutex<Vec<String>>,
}
impl ValidationMessages {
    pub fn errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|messages| messages.clone())
            .unwrap_or_default()
    }
    pub fn create_info(&self) -> vk::DebugUtilsMessengerCreateInfoEXT<'_> {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(validation_callback))
            .user_data(std::ptr::from_ref(self).cast_mut().cast())
    }
}
unsafe extern "system" fn validation_callback(
    _: vk::DebugUtilsMessageSeverityFlagsEXT,
    _: vk::DebugUtilsMessageTypeFlagsEXT,
    message: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    context: *mut c_void,
) -> vk::Bool32 {
    // Vulkan owns the callback duration; both pointers remain valid until the messenger is destroyed.
    let (Some(message), Some(storage)) = (unsafe { message.as_ref() }, unsafe {
        context.cast::<ValidationMessages>().as_ref()
    }) else {
        return vk::FALSE;
    };
    if !message.p_message.is_null()
        && let Ok(mut errors) = storage.errors.lock()
    {
        errors.push(
            unsafe { CStr::from_ptr(message.p_message) }
                .to_string_lossy()
                .into_owned(),
        );
    }
    vk::FALSE
}
pub(super) struct ValidationMessenger {
    loader: debug_utils::Instance,
    handle: vk::DebugUtilsMessengerEXT,
}
impl ValidationMessenger {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        messages: &ValidationMessages,
    ) -> Result<Self, vk::Result> {
        let loader = debug_utils::Instance::new(entry, instance);
        let handle = unsafe { loader.create_debug_utils_messenger(&messages.create_info(), None) }?;
        Ok(Self { loader, handle })
    }
}
impl Drop for ValidationMessenger {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_debug_utils_messenger(self.handle, None) };
    }
}
