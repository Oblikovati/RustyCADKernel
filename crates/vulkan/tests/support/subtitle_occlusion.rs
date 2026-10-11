//! A later green surface proves that transparent Subtitle backgrounds retain native depth writes.
use super::TextScene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
const OCCLUSION_SQUARE: [[f32; 3]; 6] = [
    [-20., -20., -1.],
    [20., -20., -1.],
    [20., 20., -1.],
    [-20., -20., -1.],
    [20., 20., -1.],
    [-20., 20., -1.],
];
/// Append a later depth witness; `append_subtitle_occluder(&scene, "subtitle-zero-alpha-depth")`.
pub fn append_subtitle_occluder(scene: &TextScene, mode: &str) {
    if !mode.ends_with("-depth") && mode != "subtitle-zero-alpha-no-depth-write" {
        return;
    }
    let vertices = subtitle_occluding_square();
    scene
        .scene
        .group
        .set_primitives_aspect(&super::super::scene::unlit_style([0., 1., 0., 1.]));
    scene.scene.group.add_primitive_array(&vertices, true);
    configure_subtitle_depth_writes(scene, mode);
}

fn subtitle_occluding_square() -> Handle<Graphic3dArrayOfPrimitives> {
    let vertices = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 6, 0, 0, 0)
            .unwrap(),
    );
    for [x, y, z] in OCCLUSION_SQUARE {
        vertices.add_vertex(x, y, z).unwrap();
    }
    vertices
}

fn configure_subtitle_depth_writes(scene: &TextScene, mode: &str) {
    let mut settings = scene
        .scene
        .viewer
        .z_layer_settings(GRAPHIC3D_ZLAYER_ID_DEFAULT)
        .unwrap();
    settings.set_enable_depth_write(mode != "subtitle-zero-alpha-no-depth-write");
    scene
        .scene
        .viewer
        .set_z_layer_settings(GRAPHIC3D_ZLAYER_ID_DEFAULT, &settings)
        .unwrap();
}
