//! Retained section passes follow live plane, group and inherited appearance state.
#[path = "support/capping_scene.rs"]
mod capping_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
fn setup(
    mode: &str,
) -> (
    scene::ViewerScene,
    RecordingDevice,
    Handle<Graphic3dClipPlane>,
) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    let plane = capping_scene::configure_capping(&scene, mode);
    (scene, device, plane)
}
#[test]
fn cap_masks_snapshot_live_equations_and_preserve_ordinary_geometry() {
    let (scene, device, plane) = setup("cap-basic");
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert_eq!(
        frame
            .draws()
            .iter()
            .map(|d| d.capping_pass())
            .collect::<Vec<_>>(),
        [
            RenderCappingPass::None,
            RenderCappingPass::MaskStart,
            RenderCappingPass::Surface
        ]
    );
    assert_eq!(frame.draws()[0].face_culling(), RenderFaceCulling::None);
    assert_eq!(frame.draws()[2].face_culling(), RenderFaceCulling::None);
    assert_eq!(frame.draws()[0].vertices(), frame.draws()[1].vertices());
    assert_eq!(
        frame.draws()[2]
            .vertices()
            .iter()
            .filter(|v| v.weight == 0.)
            .count(),
        8
    );
    plane.set_equation(Vec4::new(0., 0., -1., -2.)).unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(frame.draws()[1].clipping().unwrap().chains()[0][0][3], 0.);
    assert_eq!(
        device.last_frame().draws()[1].clipping().unwrap().chains()[0][0][3],
        -2.
    );
}
#[test]
fn open_disabled_outside_empty_and_override_states_do_not_generate_caps() {
    for mode in [
        "cap-open",
        "cap-disabled",
        "cap-off",
        "cap-outside",
        "cap-empty",
        "cap-override",
    ] {
        let (scene, device, _) = setup(mode);
        scene.render_view().try_redraw().unwrap();
        assert!(
            device
                .last_frame()
                .draws()
                .iter()
                .all(|d| d.capping_pass() == RenderCappingPass::None),
            "{mode}"
        );
    }
}
#[test]
fn closed_group_masks_accumulate_unless_object_properties_require_separate_surfaces() {
    for (mode, passes) in [
        (
            "cap-overlap",
            vec![
                RenderCappingPass::MaskStart,
                RenderCappingPass::MaskContinue,
                RenderCappingPass::Surface,
            ],
        ),
        (
            "cap-overlap-object",
            vec![
                RenderCappingPass::MaskStart,
                RenderCappingPass::Surface,
                RenderCappingPass::MaskStart,
                RenderCappingPass::Surface,
            ],
        ),
    ] {
        let (scene, device, _) = setup(mode);
        scene.render_view().try_redraw().unwrap();
        assert_eq!(
            device.last_frame().draws()[2..]
                .iter()
                .map(|d| d.capping_pass())
                .collect::<Vec<_>>(),
            passes
        );
    }
}
#[test]
fn child_on_flag_does_not_disable_a_cap_in_an_enabled_chain() {
    let (scene, device, _) = setup("cap-chain-disabled-child");
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    let caps: Vec<_> = frame
        .draws()
        .iter()
        .filter(|d| d.capping_pass() == RenderCappingPass::Surface)
        .collect();
    assert_eq!(caps.len(), 2);
    assert_eq!(
        caps[0].clipping().unwrap().chains(),
        [vec![[-1., 0., 0., 0.]]]
    );
    assert_eq!(
        caps[1].clipping().unwrap().chains(),
        [vec![[0., 0., 1., 0.]]]
    );
}
#[test]
fn material_inheritance_copies_interior_color_without_changing_plane_or_shading() {
    let (scene, device, plane) = setup("cap-object");
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[2].vertices()[0].color,
        [1., 0., 0., 1.]
    );
    assert_eq!(plane.capping_aspect().interior_color().green(), 1.);
    scene.group.aspects().set_interior_color_rgba(
        rustycad_tkernel::quantity::ColorRgba::new(1., 1., 0., 1.).unwrap(),
    );
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[2].vertices()[0].color,
        [1., 1., 0., 1.]
    );
}
#[test]
fn cap_plane_model_replaces_group_and_structure_transforms() {
    let (scene, device, _) = setup("cap-transformed");
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert_eq!(
        &frame.draws()[1].clipping().unwrap().world_from_object()[12..16],
        &[0., 0., 0., 1.]
    );
    let (plain, other, _) = setup("cap-basic");
    plain.render_view().try_redraw().unwrap();
    assert_eq!(
        frame.draws()[2].clip_transform(),
        other.last_frame().draws()[2].clip_transform()
    );
}

#[test]
fn shadow_and_own_groups_keep_separate_masks_even_when_their_volumes_overlap() {
    let (scene, device, _) = setup("cap-basic");
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    scene.structure.erase();
    let own = shadow.new_group().unwrap();
    own.set_group_primitives_aspect(&scene::unlit_style([1., 1., 0., 1.]));
    capping_scene::cube(&*own, 0.);
    shadow.calculate_bound_box();
    shadow.display();
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert_eq!(frame.draws().len(), 6);
    assert_eq!(
        frame.draws()[2].capping_pass(),
        RenderCappingPass::MaskStart
    );
    assert_eq!(
        frame.draws()[4].capping_pass(),
        RenderCappingPass::MaskStart
    );
    assert_eq!(frame.draws()[2].vertices()[0].color, [1., 1., 0., 1.]);
    assert_eq!(frame.draws()[4].vertices()[0].color, [1., 0., 0., 1.]);
}

#[test]
fn persistent_parent_uses_local_caps_but_disables_global_fragment_planes() {
    for (mode, caps) in [
        ("cap-pers-global", 0),
        ("cap-pers-local", 1),
        ("cap-group-pers", 1),
    ] {
        let (scene, device, _) = setup(mode);
        scene.render_view().try_redraw().unwrap();
        assert_eq!(
            device
                .last_frame()
                .draws()
                .iter()
                .filter(|d| d.capping_pass() == RenderCappingPass::Surface)
                .count(),
            caps,
            "{mode}"
        );
    }
}

#[test]
fn lit_caps_keep_native_normals_and_unlit_masks_for_every_legacy_model() {
    for (model, shading) in [
        (
            Graphic3dTypeOfShadingModel::PhongFacet,
            RenderShading::PhongFacet,
        ),
        (Graphic3dTypeOfShadingModel::Gouraud, RenderShading::Gouraud),
        (Graphic3dTypeOfShadingModel::Phong, RenderShading::Phong),
    ] {
        assert_lit_cap_snapshot(model, shading);
    }
}

#[test]
fn cap_default_resolves_live_view_model_and_rejects_pbr() {
    let (scene, device, plane) = setup("cap-basic");
    plane
        .capping_aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Default);
    scene
        .view
        .set_shading_model(Graphic3dTypeOfShadingModel::Phong)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[2].lighting().unwrap().shading(),
        RenderShading::Phong
    );
    scene
        .view
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert!(device.last_frame().draws()[2].lighting().is_none());
    plane
        .capping_aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Pbr);
    assert!(scene.render_view().try_redraw().is_err());
}

fn assert_lit_cap_snapshot(model: Graphic3dTypeOfShadingModel, shading: RenderShading) {
    let (scene, device, plane) = setup("cap-transformed");
    plane.capping_aspect().set_shading_model(model);
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    assert!(frame.draws()[1].lighting().is_none());
    let cap = frame.draws()[2].lighting().unwrap();
    assert_eq!(cap.shading(), shading);
    assert_eq!(cap.normals(), &[[0., -1., 0.]; 12]);
    let (plain, other, basic) = setup("cap-basic");
    basic.capping_aspect().set_shading_model(model);
    plain.render_view().try_redraw().unwrap();
    assert_eq!(cap, other.last_frame().draws()[2].lighting().unwrap());
}
