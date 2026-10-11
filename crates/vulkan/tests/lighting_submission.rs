//! Live normals and shading-model policy are read through the retained Graphic3d seam.
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::RenderShading;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
fn add_normals(scene: &scene::GraphicScene) -> Handle<Graphic3dArrayOfPrimitives> {
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(
            Graphic3dTypeOfPrimitiveArray::Triangles,
            3,
            0,
            3,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )
        .unwrap(),
    );
    for point in [[-0.8, -0.8, 0.], [0.8, -0.8, 0.], [0., 0.8, 0.]] {
        array.add_vertex_normal(point, [0., 0., 1.]).unwrap();
    }
    array.add_edges3(1, 2, 3).unwrap();
    scene.group.add_primitive_array(&array, true);
    array
}
#[test]
fn live_nodal_normals_snapshot_without_mutating_previous_draws() {
    let scene = scene::GraphicScene::new();
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    let array = add_normals(&scene);
    let before = scene.group.primitive_draws(&scene.camera).unwrap();
    array.set_vertex_normal(2, 1., 0., 0.).unwrap();
    let after = scene.group.primitive_draws(&scene.camera).unwrap();
    assert_eq!(before[0].lighting().unwrap().normals()[1], [0., 0., 1.]);
    assert_eq!(after[0].lighting().unwrap().normals()[1], [1., 0., 0.]);
    assert_eq!(after[0].lighting().unwrap().shading(), RenderShading::Phong);
}
#[test]
fn default_and_missing_normals_resolve_using_native_surface_policy() {
    let scene = scene::GraphicScene::new();
    scene.triangle();
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Default);
    assert_eq!(
        scene.group.primitive_draws(&scene.camera).unwrap()[0]
            .lighting()
            .unwrap()
            .shading(),
        RenderShading::PhongFacet
    );
    scene.group.clear(false);
    scene
        .group
        .set_group_primitives_aspect(&scene::unlit_style([1.; 4]));
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Gouraud);
    add_normals(&scene);
    assert_eq!(
        scene.group.primitive_draws(&scene.camera).unwrap()[0]
            .lighting()
            .unwrap()
            .shading(),
        RenderShading::Gouraud
    );
}
#[test]
fn pbr_is_rejected_without_phong_substitution() {
    let scene = scene::GraphicScene::new();
    add_normals(&scene);
    let aspect = scene.group.aspects();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Pbr);
    assert!(scene.group.primitive_draws(&scene.camera).is_err());
}

fn opacity_aspect(scene: &scene::GraphicScene, front_alpha: f32, back_alpha: f32) {
    let aspect = scene.group.aspects();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    let mut front = aspect.front_material();
    front.set_transparency(1. - front_alpha).unwrap();
    aspect.set_front_material(front);
    let mut back = aspect.back_material();
    back.set_transparency(1. - back_alpha).unwrap();
    aspect.set_back_material(back);
}
#[test]
fn live_material_opacity_changes_passes_without_mutating_earlier_draws() {
    let scene = scene::GraphicScene::new();
    add_normals(&scene);
    opacity_aspect(&scene, 1., 1.);
    let before = scene.group.primitive_draws(&scene.camera).unwrap();
    opacity_aspect(&scene, 0.5, 1.);
    let after = scene.group.primitive_draws(&scene.camera).unwrap();
    assert!(!before[0].is_surface_blended());
    assert!(after[0].is_surface_blended());
    opacity_aspect(&scene, 1., 1.);
    assert!(!scene.group.primitive_draws(&scene.camera).unwrap()[0].is_surface_blended());
    assert!(after[0].is_surface_blended());
}
#[test]
fn back_face_opacity_is_automatic_only_when_materials_are_distinguished() {
    let scene = scene::GraphicScene::new();
    add_normals(&scene);
    opacity_aspect(&scene, 1., 0.5);
    assert!(!scene.group.primitive_draws(&scene.camera).unwrap()[0].is_surface_blended());
    scene.group.aspects().set_distinguish(true);
    assert!(scene.group.primitive_draws(&scene.camera).unwrap()[0].is_surface_blended());
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
    assert!(!scene.group.primitive_draws(&scene.camera).unwrap()[0].is_surface_blended());
    assert_eq!(scene.group.aspects().back_material().transparency(), 0.5);
}
#[test]
fn explicit_blend_keeps_opaque_materials_in_the_deferred_pass() {
    let scene = scene::GraphicScene::new();
    add_normals(&scene);
    opacity_aspect(&scene, 1., 1.);
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
    assert!(scene.group.primitive_draws(&scene.camera).unwrap()[0].is_surface_blended());
}
