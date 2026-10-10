//! Closed cube sections paired with martensite_vulkan_capping.cxx.
#![allow(dead_code)]
use crate::scene::ViewerScene;
use rustycad_tkernel::{
    ncollection::Vec4,
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::graphic3d::*;
pub fn cube(group: &dyn Graphic3dGroup, offset: f32) {
    let array = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 8, 0, 36, 0)
            .unwrap(),
    );
    for [x, y, z] in [
        [-6., -6., -6.],
        [6., -6., -6.],
        [6., 6., -6.],
        [-6., 6., -6.],
        [-6., -6., 6.],
        [6., -6., 6.],
        [6., 6., 6.],
        [-6., 6., 6.],
    ] {
        array.add_vertex(x + offset, y, z).unwrap();
    }
    for [a, b, c] in [
        [1, 3, 2],
        [1, 4, 3],
        [5, 6, 7],
        [5, 7, 8],
        [1, 2, 6],
        [1, 6, 5],
        [4, 8, 7],
        [4, 7, 3],
        [1, 5, 8],
        [1, 8, 4],
        [2, 3, 7],
        [2, 7, 6],
    ] {
        array.add_edges3(a, b, c).unwrap();
    }
    group.add_primitive_array(&array, true);
    group.set_closed(true);
}
pub fn cap_plane(equation: [f64; 4]) -> Handle<Graphic3dClipPlane> {
    let [x, y, z, w] = equation;
    let plane = Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(x, y, z, w)).unwrap());
    plane.set_capping(true);
    plane
        .capping_aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    plane.capping_aspect().set_polygon_offsets(0, 0., 0.);
    plane
        .set_capping_color(Color::new(0., 1., 0., ColorSpace::Rgb).unwrap())
        .unwrap();
    plane
}
pub fn configure_capping(scene: &ViewerScene, mode: &str) -> Handle<Graphic3dClipPlane> {
    scene.group.clear(false);
    scene
        .group
        .set_group_primitives_aspect(&crate::scene::unlit_style([1., 0., 0., 1.]));
    cube(&*scene.group, 0.);
    scene.group.set_closed(mode != "cap-open");
    let plane = cap_plane(if mode == "cap-oblique" {
        [0.6, 0., -0.8, 0.]
    } else {
        [0., 0., -1., 0.]
    });
    configure_lit_cap(scene, &plane, mode);
    configure_plane(&plane, mode);
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(plane.clone());
    configure_chain(&planes, &plane, mode);
    if mode == "cap-local" {
        scene.structure.set_clip_planes(&planes);
    } else {
        scene.view.set_clip_planes(&planes);
    }
    configure_groups(scene, mode);
    configure_cap_culling(scene, mode);
    configure_cap_persistence(scene, mode, &plane);
    scene.structure.calculate_bound_box();
    plane
}
fn configure_plane(plane: &Graphic3dClipPlane, mode: &str) {
    if mode == "cap-off" {
        plane.set_capping(false);
    }
    if mode == "cap-disabled" {
        plane.set_on(false).unwrap();
    }
    if mode == "cap-moved" {
        plane.set_equation(Vec4::new(0., 0., -1., -2.)).unwrap();
    }
    if mode == "cap-outside" {
        plane.set_equation(Vec4::new(0., 0., -1., 8.)).unwrap();
    }
    if mode == "cap-empty" {
        plane.set_equation(Vec4::new(0., 0., -1., -8.)).unwrap();
    }
    if mode == "cap-object" || mode == "cap-overlap-object" {
        plane.set_use_object_material(true);
    }
}
fn configure_chain(
    planes: &Graphic3dSequenceOfHClipPlane,
    plane: &Handle<Graphic3dClipPlane>,
    mode: &str,
) {
    if mode == "cap-chain" || mode == "cap-chain-disabled-child" {
        let child = cap_plane([1., 0., 0., 0.]);
        if mode.ends_with("child") {
            child.set_on(false).unwrap();
        }
        Graphic3dClipPlane::set_chain_next_plane(plane, child).unwrap();
    }
    if mode == "cap-intersection" {
        planes.append(cap_plane([1., 0., 0., 0.]));
    }
}
fn configure_groups(scene: &ViewerScene, mode: &str) {
    if mode.starts_with("cap-overlap") {
        let second = scene.structure.new_group().unwrap();
        second.set_group_primitives_aspect(&crate::scene::unlit_style([1., 1., 0., 1.]));
        cube(&*second, 3.);
    }
    if mode == "cap-transformed" {
        scene
            .group
            .set_transformation(&Transform3::translation_vector(Vector3::new(2., 1., 1.)));
    }
    if mode == "cap-override" {
        let local = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        local.set_override_global(true);
        scene.structure.set_clip_planes(&local);
    }
}

pub fn configure_cap_persistence(
    scene: &ViewerScene,
    mode: &str,
    plane: &Handle<Graphic3dClipPlane>,
) {
    if !mode.contains("pers") {
        return;
    }
    let pers = Handle::new(
        Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &rustycad_tkmath::gp::Point3::new(1., 0., 0.),
        )
        .unwrap(),
    )
    .cast();
    if mode == "cap-group-pers" {
        scene.group.set_transform_persistence(&pers);
    } else {
        scene.structure.set_transform_persistence(&pers);
    }
    if mode == "cap-pers-local" {
        let local = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        local.append(plane.clone());
        scene.structure.set_clip_planes(&local);
    }
    scene.view.change_rendering_params(&mut |params| {
        params.ray_tracing.frustum_culling_state = Graphic3dRenderingParamsFrustumCulling::Off
    });
}
pub fn cap_shadow(scene: &ViewerScene) -> Handle<Graphic3dStructure> {
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    let group = shadow.new_group().unwrap();
    group.set_group_primitives_aspect(&crate::scene::unlit_style([1., 1., 0., 1.]));
    cube(&*group, 3.);
    scene.structure.erase();
    shadow.calculate_bound_box();
    shadow.display();
    shadow
}

fn configure_cap_culling(scene: &ViewerScene, mode: &str) {
    if mode == "cap-front" {
        scene
            .cview()
            .set_backfacing_model(Graphic3dTypeOfBackfacingModel::FrontCulled);
    }
    if mode == "cap-back" {
        scene
            .cview()
            .set_backfacing_model(Graphic3dTypeOfBackfacingModel::BackCulled);
    }
    if mode == "cap-mirrored" || mode == "cap-mirror-uncut" {
        scene
            .structure
            .set_transformation(&Handle::new(rustycad_tkmath::toploc::Datum3::new(
                Transform3::scaling(rustycad_tkmath::gp::Point3::default(), -1.).unwrap(),
            )));
    }
    if mode == "cap-mirror-uncut" {
        scene.view.set_clip_planes(&Handle::default());
    }
}

fn configure_lit_cap(scene: &ViewerScene, plane: &Graphic3dClipPlane, mode: &str) {
    let model = match mode {
        "cap-lit-facet" => Graphic3dTypeOfShadingModel::PhongFacet,
        "cap-lit-gouraud" => Graphic3dTypeOfShadingModel::Gouraud,
        "cap-lit-phong" | "cap-lit-unlit-view" => Graphic3dTypeOfShadingModel::Phong,
        _ => return,
    };
    plane.capping_aspect().set_shading_model(model);
    scene
        .view
        .set_shading_model(if mode == "cap-lit-unlit-view" {
            Graphic3dTypeOfShadingModel::Unlit
        } else {
            Graphic3dTypeOfShadingModel::Phong
        })
        .unwrap();
    let lights = Handle::new(Graphic3dLightSet::default());
    let light = Handle::new(Graphic3dCLight::new(
        Graphic3dTypeOfLightSource::Directional,
    ));
    light.set_direction_coords(0., 0., -1.).unwrap();
    lights.add(&light).unwrap();
    scene.cview().set_lights(&lights);
}
