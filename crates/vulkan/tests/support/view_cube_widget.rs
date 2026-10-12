//! Concrete widget detection and click consumption use actual context selection, with deterministic timer stepping.
use super::view_cube_scene::ViewCubeCaptureScene;
use rustycad_tkernel::{
    osd::{Chronometer, Timer, WallClock},
    quantity::{Color, ColorSpace},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesSource;
use rustycad_tkservice::media::MediaTimer;
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
/// Injected wall clock drives the existing native animation timer without sleeping.
pub struct CubeWidgetClock {
    seconds: AtomicU64,
    dispatch: [AtomicUsize; 3],
}
impl WallClock for CubeWidgetClock {
    fn now(&self) -> f64 {
        f64::from_bits(self.seconds.load(Ordering::SeqCst))
    }
}
impl CubeWidgetClock {
    /// Record actual source override execution; `clock.record_dispatch(1)`.
    pub fn record_dispatch(&self, slot: usize) {
        self.dispatch[slot].fetch_add(1, Ordering::SeqCst);
    }
    /// Read StartTimer/UpdateTimer/updateOwn counts; `clock.dispatch_counts()`.
    pub fn dispatch_counts(&self) -> [usize; 3] {
        self.dispatch
            .each_ref()
            .map(|counter| counter.load(Ordering::SeqCst))
    }
    /// Seek the actual timer's wall clock; `clock.seek(0.25)`.
    pub fn seek(&self, seconds: f64) {
        self.seconds.store(seconds.to_bits(), Ordering::SeqCst);
    }
}
/// Install event-loop stepping on the real camera animation; `let clock=configure_widget_timer(&scene)`.
pub fn configure_widget_timer(scene: &ViewCubeCaptureScene) -> Arc<CubeWidgetClock> {
    let clock = Arc::new(CubeWidgetClock {
        seconds: AtomicU64::new(0),
        dispatch: std::array::from_fn(|_| AtomicUsize::new(0)),
    });
    let timer = Timer::with_clocks(Chronometer::default(), clock.clone());
    let animation = super::view_cube_widget_animation::CubeWidgetAnimation::new(&clock);
    animation.set_timer(&Handle::new(MediaTimer::with_timer(timer)));
    scene.cube.set_view_animation(&animation.cast());
    scene.cube.set_fixed_animation_loop(false);
    clock
}
/// Detect a real point-sensitive owner through the context; `detect_widget_center(&scene)?`.
pub fn detect_widget_center(scene: &ViewCubeCaptureScene) -> Result<i32, Failure> {
    // Native Topmost highlight shares the ordinary layer; TopOSD exposes the cyan draw for capture.
    scene
        .cube
        .dynamic_hilight_attributes()
        .set_z_layer(rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOP_OSD);
    scene.context.move_to(100, 220, &scene.view, false)?;
    let owner = scene.context.detected_owner().cast::<AisViewCubeOwner>();
    assert!(
        !owner.is_null(),
        "ViewCube detected owner=null at pixel=(100,220); expected a real cube part"
    );
    assert_eq!(scene.cube.selection(0).entities().len(), 26);
    Ok(owner.main_orientation() as i32)
}
/// Consume detected-owner click before global selection; `click_widget_center(&scene)?`.
pub fn click_widget_center(scene: &ViewCubeCaptureScene) -> Result<(), Failure> {
    assert_eq!(
        scene.context.select_detected(AisSelectionScheme::Replace)?,
        AisStatusOfPick::NothingSelected
    );
    assert_eq!(scene.context.nb_selected(), 0);
    assert!(scene.cube.has_animation()?);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_widget_context_detection_highlight_click_and_camera_animation_are_submitted() {
        let device = crate::recording_device::RecordingDevice::default();
        let driver = widget_recording_driver(&device);
        let scene = ViewCubeCaptureScene::new(&driver).unwrap();
        let clock = configure_widget_timer(&scene);
        scene.render_view().try_redraw().unwrap();
        let original = scene.view.camera().orientation_matrix();
        detect_widget_center(&scene).unwrap();
        scene.render_view().try_redraw().unwrap();
        assert_cyan_widget_draw(&device.last_frame());
        click_widget_center(&scene).unwrap();
        assert_widget_camera_progress(&scene, &clock, original);
        assert_widget_appearance_submission(&scene, &device);
        scene.view.remove();
    }
    fn widget_recording_driver(
        device: &crate::recording_device::RecordingDevice,
    ) -> Handle<martensite_vulkan::Graphic3dRenderDriver> {
        let fonts = Arc::new(crate::fixed_glyph_source::FixedGlyphSource::default());
        martensite_vulkan::Graphic3dRenderDriver::with_backend_and_fonts(device.clone(), 512, fonts)
            .unwrap()
    }
    fn assert_widget_camera_progress(
        scene: &ViewCubeCaptureScene,
        clock: &CubeWidgetClock,
        original: rustycad_tkernel::ncollection::Mat4<f64>,
    ) {
        clock.seek(0.25);
        assert!(scene.cube.update_animation(false).unwrap());
        assert_ne!(scene.view.camera().orientation_matrix(), original);
        clock.seek(0.5);
        assert!(!scene.cube.update_animation(false).unwrap());
        assert!(scene.cube.view_animation().target_view().is_null());
        assert_eq!(clock.dispatch_counts(), [1, 2, 3]);
    }
    fn assert_widget_appearance_submission(
        scene: &ViewCubeCaptureScene,
        device: &crate::recording_device::RecordingDevice,
    ) {
        let before = device.last_frame();
        apply_widget_appearance(scene).unwrap();
        scene.render_view().try_redraw().unwrap();
        let after = device.last_frame();
        assert_ne!(before.draws()[0].vertices(), after.draws()[0].vertices());
        assert_eq!(scene.cube.selection(0).entities().len(), 26);
    }
    fn assert_cyan_widget_draw(frame: &martensite_vulkan::RenderFrame) {
        assert!(
            frame.draws().iter().any(|draw| {
                !draw.vertices().is_empty()
                    && draw.vertices().iter().all(|vertex| {
                        vertex.color[0] == 0.0 && vertex.color[1] == 1.0 && vertex.color[2] == 1.0
                    })
            }),
            "ViewCube hover frame lacks actual cyan part draw"
        );
    }
}

/// Emit complete camera and fitting state for native diagnosis; `trace_widget_view("source", &scene)?`.
pub fn trace_widget_view(label: &str, scene: &ViewCubeCaptureScene) -> Result<(), Failure> {
    trace_widget_camera(label, &scene.view.camera());
    let state = [
        scene.view.auto_z_fit_mode(),
        scene.view.is_invalidated(),
        scene.view.is_invalidated_immediate(),
    ];
    println!(
        "MARTENSITE_VIEW_STATE={label},{state:?},{}",
        scene.view.auto_z_fit_scale_factor()
    );
    trace_widget_bounds(label, scene)?;
    Ok(())
}
/// Emit retained endpoints without changing cameras; `trace_widget_camera("end", &camera)`.
pub fn trace_widget_camera(label: &str, camera: &rustycad_tkservice::graphic3d::Graphic3dCamera) {
    let vectors = [
        camera.eye().xyz(),
        camera.center().xyz(),
        camera.up().xyz(),
        camera.direction().xyz(),
    ];
    let mut channels: Vec<f64> = vectors
        .into_iter()
        .flat_map(|point| [point.x(), point.y(), point.z()])
        .collect();
    channels.extend(widget_camera_mapping(camera));
    let fields = channels
        .iter()
        .map(|value| format!("{value:.17}"))
        .collect::<Vec<_>>()
        .join(",");
    println!("MARTENSITE_CAMERA={label},{fields}");
}

fn trace_widget_bounds(label: &str, scene: &ViewCubeCaptureScene) -> Result<(), Failure> {
    for auxiliary in [false, true] {
        let bounds = scene.view.get().unwrap().view().min_max_values(auxiliary)?;
        let values = if bounds.is_void() {
            None
        } else {
            Some(bounds.get()?)
        };
        println!(
            "MARTENSITE_VIEW_BOUNDS={label},{auxiliary},{},{values:?}",
            bounds.is_void()
        );
    }
    Ok(())
}

fn widget_camera_mapping(camera: &rustycad_tkservice::graphic3d::Graphic3dCamera) -> [f64; 7] {
    [
        camera.scale(),
        camera.fov_y(),
        camera.fov_2d(),
        camera.aspect(),
        camera.projection_type() as i32 as f64,
        camera.z_near(),
        camera.z_far(),
    ]
}

/// Mutate live geometry, labels and aspects before normal context redisplay; `apply_widget_appearance(&scene)?`.
pub fn apply_widget_appearance(scene: &ViewCubeCaptureScene) -> Result<(), Failure> {
    scene.context.clear_detected(false);
    scene.cube.set_size(88.0, true);
    scene.cube.set_round_radius(0.15)?;
    scene.cube.set_axes_radius(1.4);
    scene.cube.set_font_height(14.0);
    scene
        .cube
        .set_box_color(Color::new(0.2, 0.4, 0.6, ColorSpace::Rgb)?)?;
    scene
        .cube
        .set_text_color(Color::new(0.9, 0.8, 0.1, ColorSpace::Rgb)?);
    scene.cube.set_box_side_label(
        rustycad_tkv3d::v3d::V3dTypeOfOrientation::Zpos,
        &AsciiString::from("ROOF"),
    )?;
    scene.context.redisplay(&scene.cube.cast(), false, false)
}

/// Stable real animation seek frames; `for (label,time) in widget_animation_frames()`.
pub fn widget_animation_frames() -> [(&'static str, f64); 3] {
    [
        ("animation-quarter", 0.125),
        ("animation-half", 0.25),
        ("animation-end", 0.5),
    ]
}

/// Verify actual derived dispatch occurred; `verify_widget_camera_dispatch(&clock)` after three frames.
pub fn verify_widget_camera_dispatch(clock: &CubeWidgetClock) {
    let counts = clock.dispatch_counts();
    assert!(
        counts[0] == 1 && counts[1] >= 3 && counts[2] == counts[1] + 1,
        "ViewCube derived dispatch counts={counts:?}; expected one StartTimer, at least three UpdateTimer and one additional updateOwn"
    );
    println!("{{\"martensite_widget_camera_dispatch\":{counts:?}}}");
}
