//! Real sensitive sphere picks drive the native directional owner highlight replay.
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::*, select_basics::*, select_mgr::*, select3d::*, v3d::*,
};
#[path = "support/sensitive_display_driver.rs"]
mod recording_driver;
#[test]
fn directional_highlight_arcs_and_layers_match_native() {
    for line in include_str!("fixtures/light-highlight-native.txt").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = highlight_snapshot(&expected);
        assert_eq!(actual.len(), expected.len() - 4, "{line}");
        for (actual, expected) in actual.iter().zip(&expected[4..]) {
            assert!(
                (actual - expected).abs() < 2e-12,
                "actual {actual} expected {expected}"
            );
        }
    }
}
fn highlight_snapshot(expected: &[f64]) -> Vec<f64> {
    let (driver, trace) = recording_driver::SensitiveDisplayDriver::new_handle(-1);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let light = Handle::new(Graphic3dCLight::new(
        Graphic3dTypeOfLightSource::Directional,
    ));
    let object = AisLightSource::new(&light).unwrap();
    object.set_arc_size(expected[3] as i32);
    let (_selection, sphere) = selected_sphere(&object);
    pick_highlight_sphere(&sphere, expected[0] as i32);
    let style = highlight_style(&manager, expected);
    let prs = object.get_hilight_presentation(&manager).unwrap();
    reset_highlight_trace(&trace);
    sphere
        .owner_id()
        .hilight_with_color(&manager, &style, 1)
        .unwrap();
    captured_highlight(&prs, trace.read().events.clone())
}
fn captured_highlight(prs: &Handle<dyn Graphic3dStructureSource>, events: Vec<f64>) -> Vec<f64> {
    let mut actual = vec![
        f64::from(prs.get_z_layer()),
        f64::from(prs.is_displayed()),
        events.len() as f64,
    ];
    actual.extend(events);
    actual.push(prs.groups().len() as f64);
    actual
}
fn selected_sphere(
    object: &AisLightSource,
) -> (Handle<SelectMgrSelection>, Handle<Select3DSensitiveSphere>) {
    let selection = Handle::new(SelectMgrSelection::new(0));
    object.compute_selection(&selection, 0).unwrap();
    let sphere = selection.entities()[0].base_sensitive().cast();
    (selection, sphere)
}
fn pick_highlight_sphere(sphere: &Select3DSensitiveSphere, axis: i32) {
    if axis < 0 {
        return;
    }
    let (point, direction) = if axis == 0 {
        (Point3::new(100.0, 0.0, 0.0), Direction3::X.reversed())
    } else {
        (Point3::new(0.0, 0.0, 100.0), Direction3::Z.reversed())
    };
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_axis_selecting_volume(Axis1::new(point, direction));
    volume.build_selecting_volume().unwrap();
    assert!(
        sphere
            .matches(&volume, &mut SelectBasicsPickResult::default())
            .unwrap()
    );
}
#[test]
fn native_bool_click_adapter_retains_typed_failure_and_mutation() {
    let (light, object, owner) = invalid_direction_click_scene();
    assert!(owner.handle_light_click(8192, 0).unwrap());
    assert!(!light.is_enabled());
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        owner.handle_mouse_click(
            rustycad_tkernel::ncollection::Vec2::new(0, 0),
            8192,
            0,
            false,
        )
    }))
    .unwrap_err();
    assert!(failure.downcast_ref::<Failure>().is_some());
    assert!(light.is_enabled());
    assert_eq!(
        object.attributes().point_aspect().aspect().interior_color(),
        light.color()
    );
}

fn highlight_style(manager: &PrsMgrPresentationManager, expected: &[f64]) -> Handle<Prs3dDrawer> {
    let style = Handle::new(Prs3dDrawer::new());
    style.set_arrow_aspect(Handle::new(Prs3dArrowAspect::new().unwrap()));
    style.set_color(Color::named(NamedColor::RED));
    style.set_z_layer(expected[2] as i32);
    if expected[1] != 0.0 {
        manager.begin_immediate_draw();
    }
    style
}

fn invalid_direction_click_scene() -> (
    Handle<Graphic3dCLight>,
    Handle<AisLightSource>,
    Handle<AisLightSourceOwner>,
) {
    let light = Handle::new(Graphic3dCLight::new(
        Graphic3dTypeOfLightSource::Directional,
    ));
    light.set_headlight(true).unwrap();
    let object = AisLightSource::new(&light).unwrap();
    let owner = AisLightSourceOwner::new(&object.cast(), 15);
    (light, object, owner)
}

fn reset_highlight_trace(
    trace: &rustycad_tkernel::standard::SharedState<recording_driver::SensitiveDisplayTrace>,
) {
    trace.write().events.clear();
    trace.write().groups = 0;
}
