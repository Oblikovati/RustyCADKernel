//! A live V3d viewer exercises ordinary driver, structure and group creation.
pub use crate::view_setup::unlit_aspects::unlit_style;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::v3d::{V3dView, V3dViewer};
pub struct ViewerScene {
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub structure: Handle<Graphic3dStructure>,
    pub group: Handle<dyn Graphic3dGroup>,
    pub vertices: Handle<Graphic3dArrayOfPrimitives>,
}
impl ViewerScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = configure_view(&view);
        let structure =
            Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
        let group = structure.new_group().unwrap();
        group.set_group_primitives_aspect(&unlit_style([1., 0., 0., 1.]));
        let vertices = triangle_vertices();
        group.add_primitive_array(&vertices, true);
        structure.display();
        Self {
            viewer,
            view,
            window,
            structure,
            group,
            vertices,
        }
    }
    pub fn cview(&self) -> Handle<dyn Graphic3dCView> {
        self.view.get().unwrap().view()
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.cview().cast()
    }
}

use crate::view_setup;
use view_setup::{configure_view, triangle_vertices};
