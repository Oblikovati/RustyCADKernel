//! Viewer lights retain native list nodes and borrow live arguments through virtual driver callbacks.
use super::V3dViewer;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference};
use rustycad_tkservice::graphic3d::Graphic3dCLight;
impl V3dViewer {
    /// Reset actual defined nodes, then define the native white defaults; `viewer.set_default_lights()?`.
    pub fn set_default_lights(&self) -> Result<(), Failure> {
        loop {
            // Native SetDefaultLights intentionally owns this one local handle across DelLight.
            let first = self.state.read().defined_lights.first().cloned();
            let Some(light) = first else {
                break;
            };
            self.del_light(&light)?;
        }
        super::viewer_light_defaults::define_default_viewer_lights(self)
    }
    /// Activate the live target in every live active view; `viewer.set_light_on_one(&light)?`.
    pub fn set_light_on_one(
        &self,
        light: &dyn HandleReference<Graphic3dCLight>,
    ) -> Result<(), Failure> {
        if !self.is_global_light(light) {
            let appended = light.clone_handle();
            self.state.write().active_lights.append(appended);
        }
        let mut cursor = self.active_view_iterator();
        while cursor.more() {
            cursor.value_ref().observe().set_light_on_one(light)?;
            cursor.next();
        }
        Ok(())
    }
    /// Activate currently defined nodes, preserving callback list edits; `viewer.set_light_on()?`.
    pub fn set_light_on(&self) -> Result<(), Failure> {
        let mut cursor = self.defined_light_iterator();
        while cursor.more() {
            let light = cursor.value_ref();
            if !self.is_global_light(&light) {
                self.set_light_on_one(&light)?;
            }
            cursor.next();
        }
        Ok(())
    }
    /// Remove global membership before each real view update; `viewer.set_light_off_one(&light)?`.
    pub fn set_light_off_one(
        &self,
        light: &dyn HandleReference<Graphic3dCLight>,
    ) -> Result<(), Failure> {
        let identity = light.object_address();
        super::viewer_light_mutation::remove_light_node(
            &mut self.state.write().active_lights,
            identity,
        );
        let mut cursor = self.active_view_iterator();
        while cursor.more() {
            cursor.value_ref().observe().set_light_off_one(light)?;
            cursor.next();
        }
        Ok(())
    }
    /// Preserve native global-light failure before final clear; `viewer.set_light_off()?`.
    pub fn set_light_off(&self) -> Result<(), Failure> {
        let mut lights = self.active_light_iterator();
        while lights.more() {
            let mut views = self.active_view_iterator();
            while views.more() {
                views
                    .value_ref()
                    .observe()
                    .set_light_off_one(&lights.value_ref())?;
                views.next();
            }
            lights.next();
        }
        self.state.write().active_lights.clear();
        Ok(())
    }
    /// Define the current target without activating it; `viewer.add_light(&light)`.
    pub fn add_light(&self, light: &dyn HandleReference<Graphic3dCLight>) {
        let identity = light.object_address();
        if self
            .state
            .read()
            .defined_lights
            .iter()
            .any(|stored| stored.object_address() == identity)
        {
            return;
        }
        let appended = light.clone_handle();
        self.state.write().defined_lights.append(appended);
    }
    /// Deactivate before removing defined membership; `viewer.del_light(&light)?`.
    pub fn del_light(&self, light: &dyn HandleReference<Graphic3dCLight>) -> Result<(), Failure> {
        self.set_light_off_one(light)?;
        let identity = light.object_address();
        super::viewer_light_mutation::remove_light_node(
            &mut self.state.write().defined_lights,
            identity,
        );
        Ok(())
    }
    /// Update actual active nodes with no extra view ownership; `viewer.update_lights()`.
    pub fn update_lights(&self) {
        let mut cursor = self.active_view_iterator();
        while cursor.more() {
            cursor.value_ref().observe().update_lights();
            cursor.next();
        }
    }
    /// Check actual global-list target identity; `viewer.is_global_light(&light)`.
    pub fn is_global_light(&self, light: &dyn HandleReference<Graphic3dCLight>) -> bool {
        let identity = light.object_address();
        self.state
            .read()
            .active_lights
            .iter()
            .any(|stored| stored.object_address() == identity)
    }
    /// Explicit owning snapshot convenience; native ActiveLights binds active_lights_ref; `viewer.active_lights()`.
    pub fn active_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().active_lights.iter().cloned().collect()
    }
    /// Explicit owning snapshot convenience; native DefinedLights binds defined_lights_ref; `viewer.defined_lights()`.
    pub fn defined_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().defined_lights.iter().cloned().collect()
    }
}
