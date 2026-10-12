//! A borrowed color slot preserves native const-reference lifetime without holding a lock.
use super::xr_tracked_device::XrTrackedDeviceParameters;
use rustycad_tkernel::{quantity::Color, standard::SharedState};
/// A live color reference; `let alias=device.laser_color_ref(); device.set_laser_color(red)?;`.
#[derive(Clone, Copy)]
pub struct AisXrLaserColorRef<'a> {
    parameters: &'a SharedState<XrTrackedDeviceParameters>,
}
impl<'a> AisXrLaserColorRef<'a> {
    pub(super) fn new(parameters: &'a SharedState<XrTrackedDeviceParameters>) -> Self {
        // The borrowed device, not an owning color snapshot, controls the alias lifetime.
        Self { parameters }
    }
    /// Copies the color at observation time; `let color=alias.value();`.
    pub fn value(&self) -> Color {
        // A short read permits callbacks and setters while aliases remain alive.
        self.parameters.read().laser_color
    }
    /// Compares actual native color slots; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AisXrLaserColorRef<'_>) -> bool {
        // Equal colors from different devices are distinct const-reference objects.
        std::ptr::eq(self.parameters, other.parameters)
    }
}
