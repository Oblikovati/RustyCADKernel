//! Actual viewer layers prove when native transparent accumulations are shared or flushed.
use super::super::scene::ViewerScene;
use super::{oit_fill_aspect, oit_quad};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

/// Populate actual separate layers; `install_oit_layers(scene, "layer-clear")`.
pub fn install_oit_layers(scene: &ViewerScene, fixture: &str) -> Handle<Graphic3dStructure> {
    let mut settings = Graphic3dZLayerSettings::default();
    settings.set_clear_depth(fixture == "layer-clear");
    let layer = scene.viewer.add_z_layer(&settings).unwrap().unwrap();
    scene
        .group
        .set_group_primitives_aspect(&oit_fill_aspect(0, "two").cast());
    scene.group.add_primitive_array(&oit_quad(0, "two"), true);
    let second =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &Handle::default()).unwrap();
    second.set_z_layer(layer);
    let group = second.new_group().unwrap();
    group.set_group_primitives_aspect(&oit_fill_aspect(1, "two").cast());
    group.add_primitive_array(&oit_quad(1, "two"), true);
    second.display();
    second
}
