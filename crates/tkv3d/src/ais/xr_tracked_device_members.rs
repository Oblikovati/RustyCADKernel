//! Nonvirtual native properties preserve alias identity and update the ray synchronously.
use super::{AisXrLaserColorRef, AisXrTrackedDeviceSource};
use rustycad_tkernel::{quantity::Color, standard::Failure};
use rustycad_tkservice::aspect::AspectXRTrackedDeviceRole;
/// Native nonvirtual device members; `device.set_laser_length(2.0)?`.
pub trait AisXrTrackedDeviceMembers: AisXrTrackedDeviceSource {
    /// Reads device role; `device.role()`.
    fn role(&self) -> AspectXRTrackedDeviceRole {
        // Role never derives from the current model or presentation.
        self.xr_tracked_device_base().parameters.read().role
    }
    /// Sets role without redisplay invalidation; `device.set_role(AspectXRTrackedDeviceRole::Other)`.
    fn set_role(&self, role: AspectXRTrackedDeviceRole) {
        // OCCT 3d097a0 stores this property without touching presentations.
        self.xr_tracked_device_base().parameters.write().role = role;
    }
    /// Reads the current native unit scale; `device.unit_factor()`.
    fn unit_factor(&self) -> f32 {
        // Existing geometry remains frozen until the next Compute call.
        self.xr_tracked_device_base().parameters.read().unit_factor
    }
    /// Sets scale without validation or redisplay; `device.set_unit_factor(1000.)`.
    fn set_unit_factor(&self, factor: f32) {
        // Native setters allow negative, infinite and NaN scale values.
        self.xr_tracked_device_base().parameters.write().unit_factor = factor;
    }
    /// Reads stored ray length; `device.laser_length()`.
    fn laser_length(&self) -> f32 {
        // Length is stored even before a presentation creates the ray group.
        self.xr_tracked_device_base().parameters.read().laser_length
    }
    /// Borrows the live native color slot; `let alias=device.laser_color_ref();`.
    fn laser_color_ref(&self) -> AisXrLaserColorRef<'_> {
        // Returning a copied Color would lose updates observed through retained const references.
        AisXrLaserColorRef::new(&self.xr_tracked_device_base().parameters)
    }
    /// Calls the native nonvirtual protected refresh; `device.compute_laser_ray()?`.
    fn compute_laser_ray(&self) -> Result<(), Failure> {
        // Blanket membership prevents application overrides from intercepting native setter refreshes.
        super::ais_xr_tracked_device_compute_laser_ray(self)
    }
    /// Copies color only when native tolerance considers it unequal; `device.set_laser_color(color)?`.
    fn set_laser_color(&self, color: Color) -> Result<(), Failure> {
        let mut state = self.xr_tracked_device_base().parameters.write();
        if state.laser_color.is_equal(color) {
            return Ok(());
        }
        state.laser_color = color;
        drop(state);
        super::ais_xr_tracked_device_compute_laser_ray(self)
    }
    /// Stores changed length before refreshing the ray; `device.set_laser_length(f32::NAN)?`.
    fn set_laser_length(&self, length: f32) -> Result<(), Failure> {
        let mut state = self.xr_tracked_device_base().parameters.write();
        if state.laser_length == length {
            return Ok(());
        }
        state.laser_length = length;
        drop(state);
        super::ais_xr_tracked_device_compute_laser_ray(self)
    }
}
impl<T: AisXrTrackedDeviceSource + ?Sized> AisXrTrackedDeviceMembers for T {}
