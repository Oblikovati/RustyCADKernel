//! A live V3d viewer exercises ordinary driver, structure and group creation.
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{quantity::Color, standard::Handle};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectWindowMembers},
    graphic3d::*,
};
use rustycad_tkv3d::v3d::{V3dView, V3dViewer};
#[path = "unlit_aspects.rs"]
mod unlit_aspects;
pub use unlit_aspects::unlit_style;
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
fn configure_view(view: &V3dView) -> Handle<AspectNeutralWindow> {
    view.set_immediate_update(false);
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_virtual(true);
    window.set_size(64, 64);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_auto_z_fit_mode(false, 1.).unwrap();
    configure_camera(&view.camera());
    view.set_background_color(
        Color::new(0., 0., 1., rustycad_tkernel::quantity::ColorSpace::Rgb).unwrap(),
    );
    window
}
fn configure_camera(camera: &Graphic3dCamera) {
    camera.set_eye(&Point3::new(0., 0., 20.)).unwrap();
    camera.set_center(&Point3::default()).unwrap();
    camera.set_up(&Direction3::new(0., 1., 0.).unwrap());
    camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
    camera.set_scale(20.);
    camera.set_aspect(1.);
    camera.set_z_range(1., 100.).unwrap();
}
fn triangle_vertices() -> Handle<Graphic3dArrayOfPrimitives> {
    let vertices = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 3, 0, 3, 0)
            .unwrap(),
    );
    for [x, y, z] in [[-8., -8., 0.], [8., -8., 0.], [0., 8., 0.]] {
        vertices.add_vertex(x, y, z).unwrap();
    }
    vertices.add_edges3(1, 2, 3).unwrap();
    vertices
}
