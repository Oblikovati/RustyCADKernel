//! The AIS path uses the same virtual view and geometry as the Graphic3d oracle scene.
#[path = "ais_triangle.rs"]
mod ais_triangle;
use crate::view_setup;
pub use ais_triangle::AisTriangle;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkv3d::{
    ais::AisInteractiveContext,
    v3d::{V3dView, V3dViewer},
};
pub struct AisViewerScene {
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub object: Handle<AisTriangle>,
}
impl AisViewerScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let styles = [
            view_setup::unlit_aspects::unlit_style([1., 0., 0., 1.]),
            view_setup::unlit_aspects::unlit_style([0., 1., 0., 1.]),
        ];
        let object = AisTriangle::new(view_setup::triangle_vertices(), styles);
        context.display(&object.cast(), false).unwrap();
        Self {
            viewer,
            view,
            window,
            context,
            object,
        }
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
}
