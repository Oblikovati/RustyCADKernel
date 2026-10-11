//! Quad lowering preserves legacy assembly, source indices and bound restart behavior.
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
#[test]
fn indexed_quadrangles_keep_native_diagonal_and_ignore_incomplete_tail() {
    let scene = scene::GraphicScene::new();
    let array = Handle::new(Graphic3dArrayOfQuadrangles::new(6, 6, 0).unwrap());
    for i in 0..6 {
        array.add_vertex(i as f32, 0., 0.).unwrap();
    }
    for index in [4, 3, 2, 1, 5, 6] {
        array.add_edge(index).unwrap();
    }
    scene.group.add_primitive_array(&array.cast(), true);
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws[0].indices(), [3, 2, 1, 3, 1, 0]);
}
#[test]
fn quadrangle_strips_restart_at_each_bound_and_drop_unpaired_tail() {
    let scene = scene::GraphicScene::new();
    let array = Handle::new(
        Graphic3dArrayOfQuadrangleStrips::new(11, 2, GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR).unwrap(),
    );
    for i in 0..11 {
        array.add_vertex(i as f32, 0., 0.).unwrap();
    }
    array.add_bound_rgb(6, 1., 0., 0.).unwrap();
    array.add_bound_rgb(5, 0., 1., 0.).unwrap();
    scene.group.add_primitive_array(&array.cast(), true);
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws[0].indices(), [0, 1, 3, 2, 0, 3, 2, 3, 5, 4, 2, 5]);
    assert_eq!(draws[1].indices(), [6, 7, 9, 8, 6, 9]);
    assert_eq!(draws[1].vertices()[0].color, [0., 1., 0., 1.]);
}
