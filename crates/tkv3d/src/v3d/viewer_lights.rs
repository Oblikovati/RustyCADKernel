//! The light members of `V3d_Viewer`: defined and active (global) lights.
use super::{V3dAmbientLight, V3dDirectionalLight, V3dTypeOfOrientation, V3dViewer};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::Graphic3dCLight;

impl V3dViewer {
    /// `SetDefaultLights`: removes the defined lights, then defines and activates a white
    /// headlight along -Z and a white ambient light. Example: `viewer.set_default_lights()?`.
    pub fn set_default_lights(&self) -> Result<(), Failure> {
        loop {
            let first = self.state.read().defined_lights.first().cloned();
            let Some(light) = first else {
                break;
            };
            self.del_light(&light)?;
        }
        let white = Color::named(NamedColor::WHITE);
        let dir_light = V3dDirectionalLight::new(V3dTypeOfOrientation::Zneg, white, false)?;
        dir_light.set_name(&AsciiString::from("headlight"));
        dir_light.set_headlight(true)?;
        let amb_light = V3dAmbientLight::new(white);
        amb_light.set_name(&AsciiString::from("amblight"));
        let (dir_light, amb_light): (Handle<Graphic3dCLight>, Handle<Graphic3dCLight>) =
            (Handle::new(dir_light).cast(), Handle::new(amb_light).cast());
        self.add_light(&dir_light);
        self.add_light(&amb_light);
        self.set_light_on_one(&dir_light)?;
        self.set_light_on_one(&amb_light)
    }
    /// `SetLightOn(light)`: activates a light globally and in every active view.
    /// Example: `viewer.set_light_on_one(&light)?`.
    pub fn set_light_on_one(&self, light: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        let views = {
            let mut state = self.state.write();
            if !state.active_lights.contains(light) {
                state.active_lights.push(light.clone());
            }
            state.active_views.clone()
        };
        views
            .iter()
            .filter_map(Handle::get)
            .try_for_each(|view| view.set_light_on_one(light))
    }
    /// `SetLightOn()`: activates every defined light; `viewer.set_light_on()?`.
    pub fn set_light_on(&self) -> Result<(), Failure> {
        let lights = self.state.read().defined_lights.clone();
        for light in &lights {
            let views = {
                let mut state = self.state.write();
                if state.active_lights.contains(light) {
                    continue;
                }
                state.active_lights.push(light.clone());
                state.active_views.clone()
            };
            views
                .iter()
                .filter_map(Handle::get)
                .try_for_each(|view| view.set_light_on_one(light))?;
        }
        Ok(())
    }
    /// `SetLightOff(light)`: deactivates a light globally and in every active view.
    /// Example: `viewer.set_light_off_one(&light)?`.
    pub fn set_light_off_one(&self, light: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        let views = {
            let mut state = self.state.write();
            state.active_lights.retain(|l| l != light);
            state.active_views.clone()
        };
        views
            .iter()
            .filter_map(Handle::get)
            .try_for_each(|view| view.set_light_off_one(light))
    }
    /// `SetLightOff()`: deactivates every active light; `viewer.set_light_off()?`.
    pub fn set_light_off(&self) -> Result<(), Failure> {
        let (lights, views) = {
            let state = self.state.read();
            (state.active_lights.clone(), state.active_views.clone())
        };
        for light in &lights {
            views
                .iter()
                .filter_map(Handle::get)
                .try_for_each(|view| view.set_light_off_one(light))?;
        }
        self.state.write().active_lights.clear();
        Ok(())
    }
    /// `AddLight`: defines a light; `viewer.add_light(&light)`.
    pub fn add_light(&self, light: &Handle<Graphic3dCLight>) {
        let mut state = self.state.write();
        if !state.defined_lights.contains(light) {
            state.defined_lights.push(light.clone());
        }
    }
    /// `DelLight`: deactivates and forgets a light; `viewer.del_light(&light)?`.
    pub fn del_light(&self, light: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        self.set_light_off_one(light)?;
        self.state.write().defined_lights.retain(|l| l != light);
        Ok(())
    }
    /// `UpdateLights`: pushes the lights of every active view to its driver view.
    /// Example: `viewer.update_lights()`.
    pub fn update_lights(&self) {
        let views = self.state.read().active_views.clone();
        views
            .iter()
            .filter_map(Handle::get)
            .for_each(|view| view.update_lights());
    }
    /// `IsGlobalLight`: whether a light is active in the viewer; `viewer.is_global_light(&light)`.
    pub fn is_global_light(&self, light: &Handle<Graphic3dCLight>) -> bool {
        self.state.read().active_lights.contains(light)
    }
    /// `ActiveLights`; `viewer.active_lights()`.
    pub fn active_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().active_lights.clone()
    }
    /// `DefinedLights`; `viewer.defined_lights()`.
    pub fn defined_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().defined_lights.clone()
    }
}
