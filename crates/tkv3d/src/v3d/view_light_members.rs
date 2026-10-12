//! Native view-light membership uses genuine retained list nodes and the real virtual driver setter.
use super::V3dView;
use rustycad_tkernel::standard::exceptions::type_mismatch;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleFieldRef, HandleReference, SharedState, catch_native_failure,
    raise_native_failure,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dGraphicDriverMembers as _, Graphic3dLightSet,
};
impl V3dView {
    /// Send actual light keys through the borrowed virtual driver slot; `view.update_lights()`.
    pub fn update_lights(&self) {
        let lights = SharedState::new(Handle::new(Graphic3dLightSet::default()));
        let mut cursor = self.active_light_iterator();
        while cursor.more() {
            raise_native_failure(
                cursor
                    .value_ref()
                    .with_handle(|light| lights.read().add(light)),
            );
            cursor.next();
        }
        self.view_ref()
            .observe()
            .set_lights(&HandleFieldRef::new(&lights, |slot| slot));
    }
    /// Append the referenced target once, preserving native failure mutations; `view.set_light_on_one(&light)?`.
    pub fn set_light_on_one(
        &self,
        light: &dyn HandleReference<Graphic3dCLight>,
    ) -> Result<(), Failure> {
        let identity = light.object_address();
        if self.contains_active_light_identity(identity) {
            return Ok(());
        }
        let appended = light.clone_handle();
        self.state.write().active_lights.append(appended);
        self.try_update_lights()
    }
    /// Reject global lights before removal and update; `view.set_light_off_one(&light)?`.
    pub fn set_light_off_one(
        &self,
        light: &dyn HandleReference<Graphic3dCLight>,
    ) -> Result<(), Failure> {
        if self.is_viewer_global_light(light) {
            return Err(type_mismatch("V3d_View::SetLightOff, the light is global"));
        }
        let identity = light.object_address();
        super::viewer_light_mutation::remove_light_node(
            &mut self.state.write().active_lights,
            identity,
        );
        self.try_update_lights()
    }
    /// Check nonnull native target identity; `view.is_active_light(&light)`.
    pub fn is_active_light(&self, light: &dyn HandleReference<Graphic3dCLight>) -> bool {
        let identity = light.object_address();
        !light.is_null()
            && self
                .state
                .read()
                .active_lights
                .iter()
                .any(|stored| stored.object_address() == identity)
    }
    /// Activate the viewer's current defined nodes without snapshot owners; `view.set_light_on()`.
    pub fn set_light_on(&self) {
        let viewer = self.viewer_ref_observer();
        let Some(viewer) = viewer.get() else {
            return;
        };
        let mut cursor = viewer.defined_light_iterator();
        while cursor.more() {
            let light = cursor.value_ref();
            self.append_missing_active_light(&light);
            cursor.next();
        }
        self.update_lights();
    }
    /// Remove only nonglobal native nodes; `view.set_light_off()`.
    pub fn set_light_off(&self) {
        let viewer = self.viewer_ref_observer();
        let Some(viewer) = viewer.get() else {
            return;
        };
        self.remove_nonglobal_light_nodes(viewer);
        self.update_lights();
    }
    /// Explicit owning convenience, distinct from ActiveLights reference; `view.active_lights()`.
    pub fn active_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().active_lights.iter().cloned().collect()
    }
    /// Native detached active-light iterator; `view.active_light_iterator().value_ref()`.
    pub fn active_light_iterator(&self) -> super::V3dLightCursor<'_> {
        self.active_lights_ref().cursor()
    }
    /// Read actual driver light limit; `view.light_limit()`.
    pub fn light_limit(&self) -> i32 {
        self.viewer_ref_observer()
            .get()
            .and_then(|v| v.driver_ref().get().map(|d| d.inquire_light_limit()))
            .unwrap_or(0)
    }
    /// Check current retained extent against driver limit; `view.if_more_lights()`.
    pub fn if_more_lights(&self) -> bool {
        (self.active_lights_ref().len() as i32) < self.light_limit()
    }
    /// Initialize native current node; `view.init_active_lights()`.
    pub fn init_active_lights(&self) {
        let mut fields = self.state.write();
        fields.active_lights_cursor = fields.active_lights.cursor();
    }
    /// Test native current node; `view.more_active_lights()`.
    pub fn more_active_lights(&self) -> bool {
        self.state.read().active_lights_cursor.more()
    }
    /// Advance the retained native cursor; `view.next_active_lights()`.
    pub fn next_active_lights(&self) {
        let mut fields = self.state.write();
        let mut cursor = fields.active_lights_cursor;
        cursor.next(&fields.active_lights);
        fields.active_lights_cursor = cursor;
    }
    /// Explicit owning convenience; native ActiveLight binds active_light_ref; `view.active_light()`.
    pub fn active_light(&self) -> Handle<Graphic3dCLight> {
        self.active_light_ref().clone_handle()
    }
    fn remove_nonglobal_light_nodes(&self, viewer: &super::V3dViewer) {
        let mut fields = self.state.write();
        let mut cursor = fields.active_lights.cursor();
        while cursor.more() {
            advance_or_remove_view_light(viewer, &mut fields.active_lights, &mut cursor);
        }
    }
    fn contains_active_light_identity(&self, identity: *const ()) -> bool {
        super::viewer_light_mutation::contains_light_node(
            &self.state.read().active_lights,
            identity,
        )
    }
    fn append_missing_active_light(&self, light: &dyn HandleReference<Graphic3dCLight>) {
        // Native Contains accepts null; IsActiveLight intentionally does not (3d097a0).
        if self.contains_active_light_identity(light.object_address()) {
            return;
        }
        let appended = light.clone_handle();
        self.state.write().active_lights.append(appended);
    }
    fn is_viewer_global_light(&self, light: &dyn HandleReference<Graphic3dCLight>) -> bool {
        self.viewer_ref_observer()
            .get()
            .is_some_and(|viewer| viewer.is_global_light(light))
    }
    fn try_update_lights(&self) -> Result<(), Failure> {
        catch_native_failure(|| {
            self.update_lights();
            Ok(())
        })
    }
    fn viewer_ref_observer(&self) -> rustycad_tkernel::standard::HandleObserver<super::V3dViewer> {
        self.state.read().viewer.observe()
    }
}

fn advance_or_remove_view_light(
    viewer: &super::V3dViewer,
    lights: &mut rustycad_tkernel::ncollection::AllocatorList<Handle<Graphic3dCLight>>,
    cursor: &mut rustycad_tkernel::ncollection::ListCursor,
) {
    if viewer.is_global_light(cursor.value(lights).unwrap()) {
        cursor.next(lights);
        return;
    }
    raise_native_failure(lights.remove(cursor));
}
