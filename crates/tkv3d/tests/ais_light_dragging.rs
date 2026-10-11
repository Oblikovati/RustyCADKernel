//! Native drag lifecycle replay uses a real context, view, and sphere selector.
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
#[test]
fn real_directional_drag_lifecycle_matches_native() {
    for line in include_str!("fixtures/light-dragging-native.txt").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let actual = drag_snapshot(&expected);
        assert_eq!(actual.len(), expected.len() - 3);
        for (actual, expected) in actual.iter().zip(&expected[3..]) {
            assert!(
                (actual - expected).abs() < 1e-11,
                "actual {actual} expected {expected}: {line}"
            );
        }
    }
}
fn drag_snapshot(expected: &[f64]) -> Vec<f64> {
    let (viewer, view) = drag_view();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let object = drag_object(expected);
    context.display(&object.cast(), false).unwrap();
    let owner = AisLightSourceOwner::new(&object.cast(), 15);
    let target = match expected[2] as i32 {
        0 => 128,
        1 => 140,
        _ => 400,
    };
    replay_drag_actions(DragLifecycleProbe::new(
        &object,
        &context,
        &view,
        &owner.cast(),
        target,
    ))
}
fn replay_drag_actions(probe: DragLifecycleProbe<'_>) -> Vec<f64> {
    let mut actual = Vec::new();
    for action in DRAG_ACTIONS {
        actual.push(f64::from(probe.step(action)));
        append_drag_transform(probe.object, &mut actual);
        append_drag_direction(probe.object, &mut actual);
    }
    actual
}
struct DragLifecycleProbe<'a> {
    object: &'a AisLightSource,
    context: &'a Handle<AisInteractiveContext>,
    view: &'a Handle<V3dView>,
    owner: &'a Handle<dyn rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource>,
    target: i32,
}
impl DragLifecycleProbe<'_> {
    fn new<'a>(
        object: &'a AisLightSource,
        context: &'a Handle<AisInteractiveContext>,
        view: &'a Handle<V3dView>,
        owner: &'a Handle<dyn rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource>,
        target: i32,
    ) -> DragLifecycleProbe<'a> {
        DragLifecycleProbe {
            object,
            context,
            view,
            owner,
            target,
        }
    }
    fn step(&self, action: AisDragAction) -> bool {
        self.object.process_dragging(
            self.context,
            self.view,
            self.owner,
            Vec2::new(128, 128),
            Vec2::new(self.target, 128),
            action,
        )
    }
}

fn drag_view() -> (Handle<V3dViewer>, Handle<V3dView>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(256, 256);
    view.set_window(&window.cast(), 0).unwrap();
    let camera = view.camera();
    camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
    camera.set_eye(&Point3::new(0.0, 0.0, 100.0)).unwrap();
    camera.set_center(&Point3::default()).unwrap();
    camera.set_up(&Direction3::Y);
    camera.set_scale(100.0);
    camera.set_z_range(1.0, 200.0).unwrap();
    (viewer, view)
}
fn append_drag_transform(object: &AisLightSource, values: &mut Vec<f64>) {
    let transform = object.local_transformation();
    for row in 1..=3 {
        for column in 1..=4 {
            values.push(transform.value(row, column).unwrap());
        }
    }
}

fn drag_object(expected: &[f64]) -> Handle<AisLightSource> {
    let light = fixture_source::fixture_light(expected[0] as usize);
    if expected[0] == 1.0 || expected[0] == 3.0 {
        light.set_direction(&Direction3::Z.reversed()).unwrap();
    }
    let object = AisLightSource::new(&light).unwrap();
    object.set_zoomable(true);
    object.set_draggable(expected[1] != 0.0);
    object
}
fn append_drag_direction(object: &AisLightSource, actual: &mut Vec<f64>) {
    let light = object.light();
    if matches!(
        light.light_type(),
        Graphic3dTypeOfLightSource::Directional | Graphic3dTypeOfLightSource::Spot
    ) {
        let direction = light.direction().unwrap();
        actual.extend([direction.x(), direction.y(), direction.z()]);
    } else {
        actual.extend([0.0; 3]);
    }
}

#[path = "support/light_fixture_source.rs"]
mod fixture_source;

const DRAG_ACTIONS: [AisDragAction; 5] = [
    AisDragAction::Start,
    AisDragAction::Confirmed,
    AisDragAction::Update,
    AisDragAction::Abort,
    AisDragAction::Stop,
];
