//! Native marker fallback and live nodal lighting use the production extraction seam.
#[allow(dead_code)]
#[path = "support/graphic_scene.rs"]
mod scene;
use martensite_vulkan::RenderShading;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
fn normal_points(scene: &scene::GraphicScene) -> Handle<Graphic3dArrayOfPoints> {
    let points = Handle::new(Graphic3dArrayOfPoints::with_options(2, false, true).unwrap());
    for x in [-1., 1.] {
        let index = points.add_vertex_f64(x, 0., 0.).unwrap();
        points.set_vertex_normal(index, 0., 0., 1.).unwrap();
    }
    scene.group.add_primitive_array(&points.cast(), true);
    points
}
#[test]
fn marker_normals_light_and_remain_live_without_changing_previous_draws() {
    let scene = scene::GraphicScene::new();
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    let points = normal_points(&scene);
    let before = scene.group.primitive_draws(&scene.camera).unwrap();
    points.set_vertex_normal(2, 1., 0., 0.).unwrap();
    let after = scene.group.primitive_draws(&scene.camera).unwrap();
    assert_eq!(before[0].lighting().unwrap().normals()[1], [0., 0., 1.]);
    assert_eq!(after[0].lighting().unwrap().normals()[1], [1., 0., 0.]);
    assert_eq!(after[0].lighting().unwrap().shading(), RenderShading::Phong);
    assert!(after[0].point_style().is_some());
}
#[test]
fn facet_marker_models_fall_back_to_unlit_with_normals() {
    let scene = scene::GraphicScene::new();
    normal_points(&scene);
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::PhongFacet);
    assert!(
        scene.group.primitive_draws(&scene.camera).unwrap()[0]
            .lighting()
            .is_none()
    );
}

#[test]
fn normal_free_markers_keep_native_unlit_fallback_under_phong() {
    let scene = scene::GraphicScene::new();
    let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0).unwrap());
    points.add_vertex_f64(0., 0., 0.).unwrap();
    scene.group.add_primitive_array(&points.cast(), true);
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    assert!(
        scene.group.primitive_draws(&scene.camera).unwrap()[0]
            .lighting()
            .is_none()
    );
}
