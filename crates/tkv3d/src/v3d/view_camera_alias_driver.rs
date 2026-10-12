//! The named camera driver changes only view construction; native driver operations are delegated.
use super::view_camera_alias_fixture::{BorrowedCameraDriver, BorrowedCameraView};
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::AspectWindow, graphic3d::*};
impl Graphic3dGraphicDriver for BorrowedCameraDriver {
    fn graphic_driver_base(&self) -> &Graphic3dGraphicDriverBase {
        self.delegate.graphic_driver_base()
    }
    fn create_view(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCView>, Failure> {
        let retained = Graphic3dRetainedView::new(manager, &self.delegate.layers())?;
        let view = Handle::new(BorrowedCameraView {
            transient: Default::default(),
            identity: self
                .control
                .light_identity
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1,
            retained,
            control: self.control.clone(),
        })
        .cast();
        graphic3d_cview_register(&view)?;
        Ok(view)
    }
    fn inquire_limit(&self, limit: Graphic3dTypeOfLimit) -> i32 {
        self.delegate.inquire_limit(limit)
    }
    fn create_structure(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        self.delegate.create_structure(manager)
    }
    fn remove_structure(&self, cstructure: &mut Handle<dyn Graphic3dCStructure>) {
        self.delegate.remove_structure(cstructure)
    }
    fn remove_view(&self, view: &Handle<dyn Graphic3dCView>) {
        self.delegate.remove_view(view)
    }
    fn enable_vbo(&self, status: bool) {
        self.delegate.enable_vbo(status)
    }
    fn is_vertical_sync(&self) -> bool {
        self.delegate.is_vertical_sync()
    }
    fn set_vertical_sync(&self, to_enable: bool) {
        self.delegate.set_vertical_sync(to_enable)
    }
    fn memory_info(&self, free_bytes: &mut usize, info: &mut AsciiString) -> bool {
        self.delegate.memory_info(free_bytes, info)
    }
    fn default_text_height(&self) -> f32 {
        self.delegate.default_text_height()
    }
    fn text_size(&self, view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3] {
        self.delegate.text_size(view, text, height)
    }
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.delegate
            .insert_layer_before(new_layer_id, settings, layer_after)
    }
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.delegate
            .insert_layer_after(new_layer_id, settings, layer_before)
    }
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
        self.delegate.remove_z_layer(layer_id)
    }
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        self.delegate.set_z_layer_settings(layer_id, settings)
    }
    fn view_exists(
        &self,
        window: &Handle<dyn AspectWindow>,
        view: &mut Handle<dyn Graphic3dCView>,
    ) -> bool {
        self.delegate.view_exists(window, view)
    }
}
