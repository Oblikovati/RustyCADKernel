//! The null graphic driver: a named test double of the Graphic3d driver interfaces
//! (visualization-design.md section 1) that keeps the scene data (structures, groups,
//! layers) and draws nothing. The tests and the native probes of the driver-facing data
//! model run on it; the C++ twin is `SvDriver` in tools/oracle/tkservice_probe_driver.cxx.
mod structure;
mod view;

pub use structure::{NullCStructure, NullGroup};
pub use view::NullView;

use crate::aspect::AspectWindow;
use crate::graphic3d::{
    Graphic3dCStructure, Graphic3dCView, Graphic3dGraphicDriver, Graphic3dGraphicDriverBase,
    Graphic3dGraphicDriverMembers as _, Graphic3dStructureManager, Graphic3dTypeOfLimit,
    Graphic3dZLayerId, Graphic3dZLayerSettings,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A driver that keeps the scene and draws nothing; `NullGraphicDriver::new_handle()`.
pub struct NullGraphicDriver {
    transient: StandardTransient,
    base: Graphic3dGraphicDriverBase,
    vertical_sync: SharedState<bool>,
}
rustycad_tkernel::impl_transient!(NullGraphicDriver, "NullGraphicDriver", StandardTransient, transient,
    interfaces: [dyn Graphic3dGraphicDriver]);

impl NullGraphicDriver {
    /// A driver without display connection, as a driver handle; `NullGraphicDriver::new_handle()`.
    pub fn new_handle() -> Handle<dyn Graphic3dGraphicDriver> {
        let driver = Self {
            transient: StandardTransient::default(),
            base: Graphic3dGraphicDriverBase::new(&Handle::default()),
            vertical_sync: SharedState::new(true),
        };
        Handle::new(driver).cast()
    }
}

impl Graphic3dGraphicDriver for NullGraphicDriver {
    fn graphic_driver_base(&self) -> &Graphic3dGraphicDriverBase {
        &self.base
    }
    /// 8 lights and clip planes, 32 views, 0 otherwise.
    fn inquire_limit(&self, limit: Graphic3dTypeOfLimit) -> i32 {
        match limit {
            Graphic3dTypeOfLimit::MaxNbLights | Graphic3dTypeOfLimit::MaxNbClipPlanes => 8,
            Graphic3dTypeOfLimit::MaxNbViews => 32,
            _ => 0,
        }
    }
    fn create_structure(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        Ok(Handle::new(NullCStructure::new(manager)?).cast())
    }
    fn remove_structure(&self, cstructure: &mut Handle<dyn Graphic3dCStructure>) {
        *cstructure = Handle::default();
    }
    /// A view with a copy of the driver layers.
    fn create_view(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCView>, Failure> {
        NullView::create(manager, &self.layers())
    }
    fn remove_view(&self, view: &Handle<dyn Graphic3dCView>) {
        let _ = view;
    }
    fn enable_vbo(&self, status: bool) {
        let _ = status;
    }
    fn is_vertical_sync(&self) -> bool {
        *self.vertical_sync.read()
    }
    fn set_vertical_sync(&self, to_enable: bool) {
        *self.vertical_sync.write() = to_enable;
    }
    fn memory_info(&self, free_bytes: &mut usize, info: &mut AsciiString) -> bool {
        (*free_bytes, *info) = (0, AsciiString::default());
        false
    }
    fn default_text_height(&self) -> f32 {
        16.0
    }
    fn text_size(&self, view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3] {
        let _ = (view, text, height);
        [0.0; 3]
    }
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.base
            .insert_layer_before(new_layer_id, settings, layer_after)
    }
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.base
            .insert_layer_after(new_layer_id, settings, layer_before)
    }
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
        self.base.remove_z_layer(layer_id)
    }
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        self.base.set_z_layer_settings(layer_id, settings)
    }
    fn view_exists(
        &self,
        window: &Handle<dyn AspectWindow>,
        view: &mut Handle<dyn Graphic3dCView>,
    ) -> bool {
        let _ = (window, view);
        false
    }
}
