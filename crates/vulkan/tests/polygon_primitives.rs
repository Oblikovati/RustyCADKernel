//! Convex polygon lowering uses actual live buffers and restarts at each bound.
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
#[test]
fn closed_datum_polygon_retains_degenerate_closing_triangle() {
    let scene = scene::GraphicScene::new();
    let array = Handle::new(Graphic3dArrayOfPolygons::new(4, 0, 0, 0).unwrap());
    for [x, y] in [[-1., -1.], [1., -1.], [0., 1.], [-1., -1.]] {
        array.add_vertex(x, y, 0.).unwrap();
    }
    scene.group.add_primitive_array(&array.cast(), true);
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws.len(), 1);
    assert_eq!(draws[0].indices(), [0, 1, 2, 0, 2, 3]);
}
#[test]
fn indexed_polygons_restart_at_colored_bounds_and_ignore_short_tail() {
    let scene = scene::GraphicScene::new();
    let array = Handle::new(
        Graphic3dArrayOfPolygons::new(8, 3, 10, GRAPHIC3D_ARRAY_FLAGS_BOUND_COLOR).unwrap(),
    );
    for index in 0..8 {
        array.add_vertex(index as f32, 0., 0.).unwrap();
    }
    for (size, red, green) in [(4, 1., 0.), (4, 0., 1.), (2, 0., 0.)] {
        array.add_bound_rgb(size, red, green, 0.).unwrap();
    }
    for index in [4, 3, 2, 1, 5, 6, 7, 8, 1, 2] {
        array.add_edge(index).unwrap();
    }
    scene.group.add_primitive_array(&array.cast(), true);
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws[0].indices(), [3, 2, 1, 3, 1, 0]);
    assert_eq!(draws[1].indices(), [4, 5, 6, 4, 6, 7]);
    assert!(draws[2].indices().is_empty());
    assert_eq!(draws[0].vertices()[0].color, [1., 0., 0., 1.]);
    assert_eq!(draws[1].vertices()[0].color, [0., 1., 0., 1.]);
}
