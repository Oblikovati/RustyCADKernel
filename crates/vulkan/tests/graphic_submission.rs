//! Graphic3d submissions retain style sequencing and live geometry until frame extraction.
#[path = "support/graphic_scene.rs"]
mod scene;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use scene::*;
#[test]
fn group_and_primitive_aspects_keep_the_native_sequence() {
    let scene = GraphicScene::new();
    let _first = scene.triangle();
    scene
        .group
        .set_primitives_aspect(&unlit_style([0., 1., 0., 1.]));
    let _second = scene.triangle();
    scene
        .group
        .set_group_primitives_aspect(&unlit_style([0., 0., 1., 1.]));
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws.len(), 2);
    assert_eq!(draws[0].vertices()[0].color, [0., 0., 1., 1.]);
    assert_eq!(draws[1].vertices()[0].color, [0., 1., 0., 1.]);
    assert_eq!(draws[0].indices(), [0, 1, 2]);
}
#[test]
fn retained_buffers_visibility_and_clear_are_observed() {
    let scene = GraphicScene::new();
    let array = scene.triangle();
    array.set_vertice(1, 0.25, 0.5, 0.75).unwrap();
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0].vertices()[0].position,
        [0.25, 0.5, 0.75]
    );
    scene.structure.set_visible(false);
    assert_no_triangles(&scene);
    scene.structure.set_visible(true);
    scene.group.clear(false);
    assert_no_triangles(&scene);
    assert!(scene.group.is_empty());
}
#[test]
fn unsupported_shading_and_invalid_indices_fail_explicitly() {
    let scene = GraphicScene::new();
    let array = scene.triangle();
    array.indices().set_index(0, 99);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    array.indices().set_index(0, 0);
    scene
        .group
        .set_group_primitives_aspect(&Handle::new(Graphic3dAspects::default()).cast());
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
}
#[test]
fn replacing_aspects_updates_all_retained_slots() {
    let scene = GraphicScene::new();
    scene.triangle();
    let previous = scene.group.aspects();
    scene.group.set_primitives_aspect(&previous);
    scene.triangle();
    let mut replacements = Graphic3dAspectsReplaceMap::new();
    replacements.insert(
        previous.object_address() as usize,
        (previous, unlit_style([0., 1., 0., 1.])),
    );
    scene.group.replace_aspects(&replacements);
    scene.group.synchronize_aspects();
    for draw in scene.group.triangle_draws(&scene.camera).unwrap() {
        assert_eq!(draw.vertices()[0].color, [0., 1., 0., 1.]);
    }
}
#[test]
fn unsupported_raster_states_fail_instead_of_being_ignored() {
    let scene = GraphicScene::new();
    scene.triangle();
    let style = scene.group.aspects();
    style.set_polygon_offsets(1, 1., 1.);
    assert!(
        scene
            .group
            .triangle_draws(&scene.camera)
            .unwrap_err()
            .to_string()
            .contains("polygon offset")
    );
    style.set_polygon_offsets(0, 0., 0.);
    style.set_face_culling(Graphic3dTypeOfBackfacingModel::BackCulled);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    style.set_face_culling(Graphic3dTypeOfBackfacingModel::Auto);
    scene.group.set_closed(true);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    scene.group.set_closed(false);
    assert!(scene.group.triangle_draws(&scene.camera).is_ok());
}
#[test]
fn unsupported_group_commands_remain_ordered_until_clear() {
    let scene = GraphicScene::new();
    scene.triangle();
    scene.group.set_stencil_test_options(true);
    scene.group.set_stencil_test_options(false);
    assert!(scene.group.triangle_draws(&scene.camera).is_err());
    scene.group.clear(true);
    scene
        .group
        .set_group_primitives_aspect(&unlit_style([1.; 4]));
    scene.triangle();
    assert_eq!(scene.group.triangle_draws(&scene.camera).unwrap().len(), 1);
}

fn assert_no_triangles(scene: &GraphicScene) {
    assert!(
        scene
            .group
            .triangle_draws(&scene.camera)
            .unwrap()
            .is_empty()
    );
}
