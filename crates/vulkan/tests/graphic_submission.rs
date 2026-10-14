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
    let unsupported = Handle::new(Graphic3dAspects::default());
    unsupported.set_shading_model(Graphic3dTypeOfShadingModel::Pbr);
    scene.group.set_group_primitives_aspect(&unsupported.cast());
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
    style.set_polygon_offsets(1, f32::NAN, 1.);
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
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0].face_culling(),
        martensite_vulkan::RenderFaceCulling::Back
    );
    style.set_face_culling(Graphic3dTypeOfBackfacingModel::Auto);
    scene.group.set_closed(true);
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0].face_culling(),
        martensite_vulkan::RenderFaceCulling::Back
    );
    scene.group.set_closed(false);
    assert!(scene.group.triangle_draws(&scene.camera).is_ok());
}
#[test]
fn ordered_stencil_commands_do_not_poison_retained_geometry() {
    let scene = GraphicScene::new();
    scene.triangle();
    scene.group.set_stencil_test_options(true);
    scene.group.set_stencil_test_options(false);
    assert_eq!(scene.group.triangle_draws(&scene.camera).unwrap().len(), 1);
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
#[test]
fn live_polygon_offset_aspects_are_retained_and_can_be_disabled() {
    let scene = GraphicScene::new();
    scene.triangle();
    let style = scene.group.aspects();
    style.set_polygon_offsets(1, 2., 3.);
    let draws = scene.group.triangle_draws(&scene.camera).unwrap();
    assert_eq!(draws[0].polygon_offset().unwrap().parameters(), (1, 2., 3.));
    style.set_polygon_offsets(8, 9., 10.);
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0]
            .polygon_offset()
            .unwrap()
            .parameters(),
        (8, 9., 10.)
    );
    style.set_polygon_offsets(0, 0., 0.);
    assert_eq!(
        scene.group.triangle_draws(&scene.camera).unwrap()[0]
            .polygon_offset()
            .unwrap()
            .parameters(),
        (0, 0., 0.)
    );
}
