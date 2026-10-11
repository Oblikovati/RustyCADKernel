//! Ordered annotation brackets paired with martensite_vulkan_group_flipping.cxx.
use crate::scene::ViewerScene;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const MODES: [&str; 9] = [
    "flip-front",
    "flip-back",
    "flip-upside",
    "flip-offset",
    "flip-nested",
    "flip-pers",
    "flip-text",
    "flip-lit",
    "flip-cap",
];
/// Rebuild native flipping brackets without redrawing; `apply(&scene,"flip-back",0.)`.
pub fn apply(scene: &ViewerScene, mode: &str, offset: f64) {
    reset_flipping_scene(scene, offset);
    configure_view(scene, mode);
    add_bracketed_geometry(scene, mode, reference_plane(mode));
    scene.structure.calculate_bound_box();
}
fn reset_flipping_scene(scene: &ViewerScene, offset: f64) {
    scene.group.clear(false);
    scene
        .group
        .set_group_primitives_aspect(&crate::scene::unlit_style([1., 0., 0., 1.]));
    let transform = Transform3::translation_vector(Vector3::new(offset, 0., 0.));
    let datum = Handle::new(rustycad_tkmath::toploc::Datum3::new(transform));
    scene.structure.set_transformation(&datum);
}
fn reference_plane(mode: &str) -> Axis2 {
    let origin = if mode == "flip-offset" || mode == "flip-cap" {
        Point3::new(2., 1., 0.)
    } else {
        Point3::default()
    };
    Axis2::with_x_direction(
        origin,
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap()
}
fn add_bracketed_geometry(scene: &ViewerScene, mode: &str, plane: Axis2) {
    scene.group.set_flipping_options(true, &plane);
    if mode == "flip-nested" {
        scene.group.set_flipping_options(true, &plane);
    }
    add_geometry(scene, mode);
    if mode == "flip-nested" {
        scene.group.set_flipping_options(false, &plane);
    }
    scene.group.set_flipping_options(false, &plane);
}
fn configure_view(scene: &ViewerScene, mode: &str) {
    scene.view.camera().set_up(
        &Direction3::new(
            0.,
            if mode == "flip-front" || mode == "flip-back" {
                1.
            } else {
                -1.
            },
            0.,
        )
        .unwrap(),
    );
    if mode == "flip-back" {
        scene
            .view
            .camera()
            .set_eye(&Point3::new(0., 0., -20.))
            .unwrap();
    }
    if mode == "flip-pers" {
        scene.group.set_transform_persistence(
            &Handle::new(
                Graphic3dTransformPers::with_point(
                    Graphic3dTransModeFlags::ZOOM_PERS,
                    &Point3::new(2., 1., 0.),
                )
                .unwrap(),
            )
            .cast(),
        );
    }
    scene.view.change_rendering_params(&mut |params| {
        params.shading_model = if mode == "flip-lit" {
            Graphic3dTypeOfShadingModel::Phong
        } else {
            Graphic3dTypeOfShadingModel::Unlit
        };
        params.ray_tracing.frustum_culling_state = Graphic3dRenderingParamsFrustumCulling::Off;
    });
}
fn add_geometry(scene: &ViewerScene, mode: &str) {
    if mode == "flip-text" {
        return add_label(scene);
    }
    if mode == "flip-cap" {
        return add_cap(scene);
    }
    if mode == "flip-lit" {
        add_directional_lighting(scene);
    }
    scene.group.add_primitive_array(&scene.vertices, true);
}
fn add_label(scene: &ViewerScene) {
    let style = Handle::new(Graphic3dAspectText3d::new(
        Color::named(NamedColor::WHITE),
        Some("DejaVu Sans"),
        1.,
        0.,
        AspectTypeOfStyleText::Normal,
        AspectTypeOfDisplayText::Normal,
    ));
    let text = Handle::new(Graphic3dText::new(16.));
    text.set_text_str("AV");
    text.set_orientation(
        &Axis2::with_x_direction(
            Point3::new(-5., 0., 0.),
            Direction3::new(0., 0., 1.).unwrap(),
            Direction3::new(1., 0., 0.).unwrap(),
        )
        .unwrap(),
    );
    scene.group.set_primitives_aspect(&style.cast());
    scene.group.add_text(&text, true);
}
fn add_cap(scene: &ViewerScene) {
    crate::capping_scene::cube(&*scene.group, 0.);
    scene
        .group
        .set_transformation(&Transform3::translation_vector(Vector3::new(3., 0., 1.)));
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(crate::capping_scene::cap_plane([0., 0., -1., 0.]));
    scene.view.set_clip_planes(&planes);
}

fn add_directional_lighting(scene: &ViewerScene) {
    scene
        .group
        .aspects()
        .set_shading_model(Graphic3dTypeOfShadingModel::PhongFacet);
    scene.group.synchronize_aspects();
    let light = Handle::new(Graphic3dCLight::new(
        Graphic3dTypeOfLightSource::Directional,
    ));
    light
        .set_direction(&Direction3::new(0., 0., -1.).unwrap())
        .unwrap();
    scene.view.set_light_on_one(&light).unwrap();
}
