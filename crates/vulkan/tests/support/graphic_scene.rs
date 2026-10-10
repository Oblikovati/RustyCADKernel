//! Named Graphic3d scene fixture owns the live parent required by retained groups.
use martensite_vulkan::Graphic3dDrawGroup;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
pub struct GraphicScene {
    pub structure: Handle<Graphic3dStructure>,
    pub group: Handle<Graphic3dDrawGroup>,
    pub camera: Graphic3dCamera,
}
impl GraphicScene {
    pub fn new() -> Self {
        let manager =
            Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
        let structure = Graphic3dStructure::new(&manager, &Handle::default()).unwrap();
        let group = Graphic3dDrawGroup::new(&structure.cast());
        structure.set_visible(true);
        group.set_group_primitives_aspect(&unlit_style([1., 0., 0., 1.]));
        let camera = Graphic3dCamera::default();
        Self {
            structure,
            group,
            camera,
        }
    }
    pub fn triangle(&self) -> Handle<Graphic3dArrayOfPrimitives> {
        let array = Handle::new(
            Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 3, 1, 3, 0)
                .unwrap(),
        );
        for [x, y, z] in [[-0.8, -0.8, 0.], [0.8, -0.8, 0.], [0., 0.8, 0.]] {
            array.add_vertex(x, y, z).unwrap();
        }
        array.add_edges3(1, 2, 3).unwrap();
        array.add_bound(3).unwrap();
        self.group.add_primitive_array(&array, true);
        array
    }
}
mod unlit_aspects;
pub use unlit_aspects::unlit_style;
