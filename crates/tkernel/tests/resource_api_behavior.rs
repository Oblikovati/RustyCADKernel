//! Behavior of every Resource member the upstream suites do not reach.
#[path = "resource_api_behavior/manager.rs"]
mod manager;
#[path = "resource_manager/support.rs"]
mod support;
#[path = "resource_api_behavior/unicode.rs"]
mod unicode;

/// The ResourceDebug flag and the Resource_Unicode format are process-wide, as natively;
/// tests that construct managers or convert through the current format run one at a time.
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SERIAL.lock().unwrap_or_else(|poison| poison.into_inner())
}
