//! Retained line lighting preserves live normal snapshots and native nodal fallback.
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::RenderShading;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

fn normal_segments(scene: &scene::GraphicScene) -> Handle<Graphic3dArrayOfPrimitives> {
    let vertices = normal_segment_storage();
    for x in [-1., 1.] {
        vertices
            .add_vertex_normal([x, 0., 0.], [0., 0., 1.])
            .unwrap();
    }
    scene.group.add_primitive_array(&vertices, true);
    vertices
}

#[test]
fn line_normals_remain_live_without_mutating_previous_draws() {
    let scene = scene::GraphicScene::new();
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    let vertices = normal_segments(&scene);
    let before = scene.group.primitive_draws(&scene.camera).unwrap();
    vertices.set_vertex_normal(2, 1., 0., 0.).unwrap();
    let after = scene.group.primitive_draws(&scene.camera).unwrap();
    assert_eq!(before[0].lighting().unwrap().normals()[1], [0., 0., 1.]);
    assert_eq!(after[0].lighting().unwrap().normals()[1], [1., 0., 0.]);
    assert_eq!(after[0].lighting().unwrap().shading(), RenderShading::Phong);
    assert!(after[0].line_style().is_some());
}

#[test]
fn line_nodal_models_select_gouraud_and_phong() {
    let scene = scene::GraphicScene::new();
    normal_segments(&scene);
    for (model, expected) in [
        (Graphic3dTypeOfShadingModel::Gouraud, RenderShading::Gouraud),
        (Graphic3dTypeOfShadingModel::Phong, RenderShading::Phong),
    ] {
        scene.group.aspects().set_shading_model(model);
        let draws = scene.group.primitive_draws(&scene.camera).unwrap();
        assert_eq!(draws[0].lighting().unwrap().shading(), expected);
    }
}

#[test]
fn facet_lines_keep_the_native_unlit_fallback() {
    let scene = scene::GraphicScene::new();
    normal_segments(&scene);
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::PhongFacet);
    let draws = scene.group.primitive_draws(&scene.camera).unwrap();
    assert!(draws[0].lighting().is_none());
}

fn normal_segment_storage() -> Handle<Graphic3dArrayOfPrimitives> {
    Handle::new(
        Graphic3dArrayOfPrimitives::new(
            Graphic3dTypeOfPrimitiveArray::Segments,
            2,
            0,
            0,
            GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
        )
        .unwrap(),
    )
}
