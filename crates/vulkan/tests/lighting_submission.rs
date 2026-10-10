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
fn pbr_and_transparent_back_materials_are_rejected_without_phong_substitution() {
    let scene = scene::GraphicScene::new();
    add_normals(&scene);
    let aspect = scene.group.aspects();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Pbr);
    assert!(scene.group.primitive_draws(&scene.camera).is_err());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspect.set_distinguish(true);
    let mut back = aspect.back_material();
    back.set_transparency(0.5).unwrap();
    aspect.set_back_material(back);
    assert!(scene.group.primitive_draws(&scene.camera).is_err());
}
