//! Named null-driver fixture records actual graphic callbacks and injected group failures.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::AspectWindow;
use rustycad_tkservice::{graphic3d::*, null_driver::*};
use std::sync::Arc;
#[path = "sensitive_display_group.rs"]
mod recording_group;
use recording_group::SensitiveDisplayGroup;
#[derive(Default)]
pub struct SensitiveDisplayTrace {
    pub events: Vec<f64>,
    pub fail_group: i32,
    pub groups: i32,
    pub append_selection: Handle<rustycad_tkv3d::select_mgr::SelectMgrSelection>,
    pub append_object: Handle<dyn rustycad_tkv3d::select_mgr::SelectMgrSelectableObject>,
}
pub type DisplayTrace = Arc<SharedState<SensitiveDisplayTrace>>;
pub struct SensitiveDisplayDriver {
    transient: StandardTransient,
    inner: Handle<dyn Graphic3dGraphicDriver>,
    trace: DisplayTrace,
}
rustycad_tkernel::impl_transient!(SensitiveDisplayDriver, "SensitiveDisplayDriver", StandardTransient, transient, interfaces: [dyn Graphic3dGraphicDriver]);
impl SensitiveDisplayDriver {
    pub fn new_handle(fail_group: i32) -> (Handle<dyn Graphic3dGraphicDriver>, DisplayTrace) {
        let trace = Arc::new(SharedState::new(SensitiveDisplayTrace {
            fail_group,
            ..Default::default()
        }));
        let driver = Self {
            transient: StandardTransient::default(),
            inner: NullGraphicDriver::new_handle(),
            trace: trace.clone(),
        };
        (Handle::new(driver).cast(), trace)
    }
}
impl Graphic3dGraphicDriver for SensitiveDisplayDriver {
    fn graphic_driver_base(&self) -> &Graphic3dGraphicDriverBase {
        self.inner.graphic_driver_base()
    }
    fn inquire_limit(&self, limit: Graphic3dTypeOfLimit) -> i32 {
        self.inner.inquire_limit(limit)
    }
    fn create_structure(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        self.trace.write().events.push(1.);
        Ok(Handle::new(SensitiveDisplayStructure::new(manager, self.trace.clone())?).cast())
    }
    fn remove_structure(&self, cstructure: &mut Handle<dyn Graphic3dCStructure>) {
        self.inner.remove_structure(cstructure)
    }
    fn create_view(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCView>, Failure> {
        self.inner.create_view(manager)
    }
    fn remove_view(&self, view: &Handle<dyn Graphic3dCView>) {
        self.inner.remove_view(view)
    }
    fn enable_vbo(&self, status: bool) {
        self.inner.enable_vbo(status)
    }
    fn is_vertical_sync(&self) -> bool {
        self.inner.is_vertical_sync()
    }
    fn set_vertical_sync(&self, to_enable: bool) {
        self.inner.set_vertical_sync(to_enable)
    }
    fn memory_info(&self, free_bytes: &mut usize, info: &mut AsciiString) -> bool {
        self.inner.memory_info(free_bytes, info)
    }
    fn default_text_height(&self) -> f32 {
        self.inner.default_text_height()
    }
    fn text_size(&self, view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3] {
        self.inner.text_size(view, text, height)
    }
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.inner
            .insert_layer_before(new_layer_id, settings, layer_after)
    }
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.inner
            .insert_layer_after(new_layer_id, settings, layer_before)
    }
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
        self.inner.remove_z_layer(layer_id)
    }
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        self.inner.set_z_layer_settings(layer_id, settings)
    }
    fn view_exists(
        &self,
        window: &Handle<dyn AspectWindow>,
        view: &mut Handle<dyn Graphic3dCView>,
    ) -> bool {
        self.inner.view_exists(window, view)
    }
}
struct SensitiveDisplayStructure {
    transient: StandardTransient,
    inner: NullCStructure,
    trace: DisplayTrace,
}
rustycad_tkernel::impl_transient!(SensitiveDisplayStructure, "SensitiveDisplayStructure", StandardTransient, transient, interfaces: [dyn Graphic3dCStructure]);
impl SensitiveDisplayStructure {
    fn new(
        manager: &Handle<Graphic3dStructureManager>,
        trace: DisplayTrace,
    ) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            inner: NullCStructure::new(manager)?,
            trace,
        })
    }
}
impl Graphic3dCStructure for SensitiveDisplayStructure {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        self.inner.cstructure_base()
    }
    fn on_visibility_changed(&self) {
        self.inner.on_visibility_changed()
    }
    fn clear(&self) {
        self.inner.clear()
    }
    fn connect(&self, structure: &dyn Graphic3dCStructure) {
        self.inner.connect(structure)
    }
    fn disconnect(&self, structure: &dyn Graphic3dCStructure) {
        self.inner.disconnect(structure)
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        self.inner.graphic_highlight(style)
    }
    fn graphic_unhighlight(&self) {
        self.inner.graphic_unhighlight()
    }
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        self.inner.shadow_link(manager)
    }
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        {
            let mut trace = self.trace.write();
            trace.events.push(2.);
            trace.groups += 1;
            if trace.groups == trace.fail_group {
                return Err(rustycad_tkernel::standard::exceptions::program_error(
                    "injected group failure; expected a successful group",
                ));
            }
        }
        let group: Handle<dyn Graphic3dGroup> =
            Handle::new(SensitiveDisplayGroup::new(structure, self.trace.clone())).cast();
        self.inner
            .cstructure_base()
            .fields
            .write()
            .groups
            .push(group.clone());
        Ok(group)
    }
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        self.inner.remove_group(group)
    }
}
