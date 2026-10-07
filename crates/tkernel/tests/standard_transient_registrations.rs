//! Resource_Manager, OSD_ThreadPool and Message_ProgressIndicator are Standard_Transient
//! classes (DEFINE_STANDARD_RTTIEXT(..., Standard_Transient)); the Rust types register the
//! same descriptor name and parent, which `get_type_name`, `get_type_descriptor` and
//! `DynamicType` answer.
use rustycad_tkernel::message::{ProgressIndicator, ProgressIndicatorHooks, ProgressScope};
use rustycad_tkernel::osd::ThreadPool;
use rustycad_tkernel::resource::{ResourceHost, ResourceManager};
use rustycad_tkernel::standard::{RuntimeTyped, Transient};

/// Indicator hooks that ignore every notification.
struct SilentHooks;
impl ProgressIndicatorHooks for SilentHooks {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
}

fn assert_descriptor<T: RuntimeTyped + Transient>(object: &T, name: &str) {
    assert_eq!(T::TYPE_NAME, name);
    assert_eq!(T::static_type().name(), name);
    assert_eq!(object.dynamic_type().name(), name);
    let parent = T::static_type().parent().map(|parent| parent.name().to_owned());
    assert_eq!(parent.as_deref(), Some("Standard_Transient"));
}

#[test]
fn resource_manager_registers_its_native_descriptor() {
    assert_descriptor(&ResourceManager::empty(ResourceHost::native()), "Resource_Manager");
}

#[test]
fn thread_pool_registers_its_native_descriptor() {
    assert_descriptor(&ThreadPool::new(1), "OSD_ThreadPool");
}

#[test]
fn progress_indicator_registers_its_native_descriptor() {
    let indicator = ProgressIndicator::new(Box::new(SilentHooks));
    assert_descriptor(&indicator, "Message_ProgressIndicator");
}
