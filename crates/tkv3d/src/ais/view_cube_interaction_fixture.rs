//! Named CPU scene and injected clock exercise the real widget without GPU or sleeping.
use super::*;
use crate::v3d::*;
use rustycad_tkernel::{
    osd::{Chronometer, Timer, WallClock},
    standard::Handle,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, media::MediaTimer, null_driver::NullGraphicDriver,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
pub(super) const STATE: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-selection-state.txt");
pub(super) const TARGETS: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-camera-targets.txt");
pub(super) const ORTHO_FRAMES: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-frames-orthographic.txt");
pub(super) const PERSPECTIVE_FRAMES: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-frames-perspective.txt");
pub(super) const ORTHO_STEPS: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-steps-orthographic.txt");
pub(super) const PERSPECTIVE_STEPS: &str =
    include_str!("../../tests/fixtures/view-cube-interaction-steps-perspective.txt");
pub(super) fn widget_rows<'a>(
    fixture: &'a str,
    kind: &'a str,
) -> impl Iterator<Item = Vec<&'a str>> + 'a {
    fixture
        .lines()
        .filter(move |line| line.split_whitespace().next() == Some(kind))
        .map(|line| line.split_whitespace().skip(1).collect())
}
pub(super) struct WidgetClock(AtomicU64);
impl WidgetClock {
    pub(super) fn set(&self, seconds: f64) {
        self.0.store(seconds.to_bits(), Ordering::SeqCst);
    }
}
impl WallClock for WidgetClock {
    fn now(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::SeqCst))
    }
}
pub(super) struct WidgetEvents {
    pub(super) frames: AtomicUsize,
    pub(super) finished: AtomicUsize,
    pub(super) finish_attached: AtomicBool,
    pub(super) advance_clock: AtomicBool,
    clock: Arc<WidgetClock>,
    animation: Handle<AisAnimationCamera>,
}
impl AisViewCubeAnimationEvents for WidgetEvents {
    fn after_animation(&self) {
        self.frames.fetch_add(1, Ordering::SeqCst);
        if self.advance_clock.load(Ordering::SeqCst) {
            self.clock.set(self.clock.now() + 0.1);
        }
    }
    fn animation_finished(&self) {
        self.finished.fetch_add(1, Ordering::SeqCst);
        self.finish_attached
            .store(!self.animation.target_view().is_null(), Ordering::SeqCst);
    }
}
pub(super) struct WidgetScene {
    pub(super) viewer: Handle<V3dViewer>,
    pub(super) view: Handle<V3dView>,
    pub(super) context: Handle<AisInteractiveContext>,
    pub(super) cube: Handle<AisViewCube>,
    pub(super) clock: Arc<WidgetClock>,
    pub(super) events: Arc<WidgetEvents>,
}
impl WidgetScene {
    pub(super) fn new(perspective: bool) -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let view = viewer.create_view().unwrap();
        configure_widget_view(&view, perspective);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        context.fields.write().last_active_view = view.downgrade();
        let cube = AisViewCube::new().unwrap();
        cube.set_context(&context);
        let clock = Arc::new(WidgetClock(AtomicU64::new(0)));
        let events = configure_widget_clock(&cube, &clock);
        Self {
            viewer,
            view,
            context,
            cube,
            clock,
            events,
        }
    }
    pub(super) fn clouds(&self, policy: i32) {
        if policy == 0 {
            return;
        }
        let points = Handle::new(Graphic3dArrayOfPoints::new(2, 0).unwrap());
        points.add_vertex(-2., -3., -4.).unwrap();
        points.add_vertex(1., 5., 7.).unwrap();
        let cloud = AisPointCloud::new().unwrap();
        cloud.set_points(&points.cast()).unwrap();
        self.context.display(&cloud.cast(), false).unwrap();
        if policy == 2 {
            self.context
                .set_selected_object(&cloud.cast(), false)
                .unwrap();
        }
    }
}
fn configure_widget_clock(cube: &AisViewCube, clock: &Arc<WidgetClock>) -> Arc<WidgetEvents> {
    let timer = Timer::with_clocks(Chronometer::default(), clock.clone());
    let animation = cube.view_animation();
    animation.set_timer(&Handle::new(MediaTimer::with_timer(timer)));
    let events = Arc::new(WidgetEvents {
        frames: AtomicUsize::new(0),
        finished: AtomicUsize::new(0),
        finish_attached: AtomicBool::new(false),
        advance_clock: AtomicBool::new(false),
        clock: clock.clone(),
        animation,
    });
    cube.set_animation_events(Some(events.clone()));
    events
}
fn configure_widget_view(view: &V3dView, perspective: bool) {
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    view.set_auto_z_fit_mode(false, 1.).unwrap();
    let camera = view.camera();
    camera
        .set_eye_and_center(&Point3::new(10., 20., 200.), &Point3::new(2., 3., 4.))
        .unwrap();
    camera.set_up(&Direction3::new(1., 3., 1.).unwrap());
    camera.orthogonalize_up().unwrap();
    camera.set_scale(200.);
    camera.set_aspect(4. / 3.);
    camera.set_projection_type(if perspective {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
}
pub(super) fn widget_camera_channels(camera: &Graphic3dCamera) -> Vec<f64> {
    let mut values = Vec::new();
    values.extend(camera.eye().xyz().coordinates());
    values.extend(camera.center().xyz().coordinates());
    values.extend(camera.up().coordinates());
    values.extend(camera.direction().coordinates());
    values.extend([
        camera.scale(),
        camera.distance(),
        camera.aspect(),
        camera.projection_type() as i32 as f64,
    ]);
    values
}
pub(super) fn assert_widget_camera(camera: &Graphic3dCamera, row: &[&str]) {
    let expected = row[1..]
        .iter()
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(widget_camera_channels(camera).len(), expected.len());
    for (index, (actual, expected)) in widget_camera_channels(camera)
        .iter()
        .zip(&expected)
        .enumerate()
    {
        assert!(
            (actual - expected).abs() <= 1e-8,
            "camera channel {index}: actual={actual}; expected={expected}; row={row:?}"
        );
    }
}
pub(super) fn configure_widget_target(scene: &WidgetScene, id: i32) -> Handle<AisViewCubeOwner> {
    scene.clouds((id / 26) % 3);
    scene
        .cube
        .view_cube_presentation_base()
        .settings
        .write()
        .is_y_up = id % 2 == 1;
    scene.cube.set_reset_camera(id % 3 == 1);
    scene.cube.set_fit_selected(id % 4 != 0);
    AisViewCubeOwner::new(
        &scene.cube,
        V3dTypeOfOrientation::ALL[(id % 26) as usize],
        5,
    )
}
