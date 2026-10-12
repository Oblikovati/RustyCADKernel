//! Derived camera atoms retain actual native base state and live handle references.
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, StandardTransient};
use rustycad_tkservice::{graphic3d::Graphic3dCamera, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, v3d::*};
use std::sync::atomic::{AtomicI32, Ordering};
struct ProtocolCameraSubclass {
    transient: StandardTransient,
    base: AisAnimationCameraBase,
    timer_calls: AtomicI32,
    timer_failure: AtomicI32,
}
rustycad_tkernel::impl_transient!(ProtocolCameraSubclass,"ProtocolCameraSubclass",dyn AisAnimationCameraSource,transient,interfaces:[dyn AisAnimationCameraSource,dyn AisAnimationSource]);
impl AisAnimationCameraSource for ProtocolCameraSubclass {
    fn camera_animation_base(&self) -> &AisAnimationCameraBase {
        // Both static handle types expose the actual owned native camera base.
        &self.base
    }
}
impl AisAnimationSource for ProtocolCameraSubclass {
    fn animation_base(&self) -> &AisAnimationBase {
        // Inherited timeline operations use the camera's actual timeline fields.
        &self.base.animation
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        // Native subclasses inherit the real camera atom unless they override it.
        ais_animation_camera_update(self, progress)
    }
    fn update_timer(&self) -> Result<f64, Failure> {
        self.timer_calls.fetch_add(1, Ordering::Relaxed);
        if self.timer_failure.load(Ordering::Relaxed) != 0 {
            return Err(Failure::new(
                "camera timer=1; expected successful virtual update",
            ));
        }
        Ok(0.25)
    }
}
fn protocol_camera(view: &Handle<V3dView>) -> Handle<ProtocolCameraSubclass> {
    Handle::new(ProtocolCameraSubclass {
        transient: StandardTransient::default(),
        base: AisAnimationCameraBase::new("protocol", view),
        timer_calls: AtomicI32::new(0),
        timer_failure: AtomicI32::new(0),
    })
}
#[test]
fn derived_camera_and_timeline_projections_preserve_virtual_timer_failure_retry() {
    let derived = protocol_camera(&Handle::default());
    let source = derived.cast::<dyn AisAnimationCameraSource>();
    let timeline = source.cast::<dyn AisAnimationSource>();
    assert!(!timeline.is_null());
    assert!(derived.is_kind_name("AIS_AnimationCamera"));
    assert!(derived.is_kind_name("AIS_Animation"));
    derived.timer_failure.store(1, Ordering::Relaxed);
    assert!(timeline.update_timer().is_err());
    derived.timer_failure.store(0, Ordering::Relaxed);
    assert_eq!(source.update_timer().unwrap(), 0.25);
    assert_eq!(derived.timer_calls.load(Ordering::Relaxed), 2);
    assert_eq!(source.object_address(), derived.object_address());
}
#[test]
fn actual_endpoint_aliases_observe_replacement_and_native_reference_counts() {
    let derived = protocol_camera(&Handle::default());
    let source = derived.cast::<dyn AisAnimationCameraSource>();
    let start = source.camera_start_ref();
    let end = source.camera_end_ref();
    let view = source.target_view_ref();
    assert!(start.is_null() && end.is_null() && view.is_null());
    let camera = retained_protocol_endpoints(source.get().unwrap());
    assert_eq!(start.object_address(), camera.object_address());
    assert_eq!(end.object_address(), camera.object_address());
    let other = Handle::new(Graphic3dCamera::default());
    source.set_camera_start(&other);
    assert_eq!(camera.ref_count(), 2);
    assert_eq!(start.object_address(), other.object_address());
    assert_eq!(other.ref_count(), 2);
    source.set_camera_end(&Handle::default());
    assert!(end.is_null());
    assert_eq!(camera.ref_count(), 1);
    assert!(start.is_same_reference(&source.camera_start_ref()));
}
#[test]
fn derived_camera_atom_mutates_the_actual_target_camera_and_retains_target_alias() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let source = protocol_camera(&view).into_cast::<dyn AisAnimationCameraSource>();
    let alias = source.target_view_ref();
    let start = configure_protocol_camera_atom(source.get().unwrap(), &view);
    let retained = view.camera();
    source
        .update_own(&AisAnimationProgress {
            pts: 0.5,
            local_pts: 0.5,
            local_normalized: 0.5,
        })
        .unwrap();
    assert_eq!(view.camera(), retained);
    assert_eq!(view.camera().scale(), (start.scale() + 400.) / 2.);
    source.set_view(&Handle::default());
    assert!(alias.is_null());
    assert!(alias.is_same_reference(&source.target_view_ref()));
}

fn retained_protocol_endpoints(source: &dyn AisAnimationCameraSource) -> Handle<Graphic3dCamera> {
    let camera = Handle::new(Graphic3dCamera::default());
    source.set_camera_start(&camera);
    source.set_camera_end(&camera);
    assert_eq!(camera.ref_count(), 3);
    camera
}
fn configure_protocol_camera_atom(
    source: &dyn AisAnimationCameraSource,
    view: &V3dView,
) -> Handle<Graphic3dCamera> {
    let start = Handle::new(view.camera().copy_members());
    let end = Handle::new(start.copy_members());
    end.set_scale(400.);
    source.set_camera_start(&start);
    source.set_camera_end(&end);
    source.set_own_duration(1.);
    start
}
#[test]
fn source_protocol_matches_native_timer_rtti_endpoint_aliases_and_ownership_rows() {
    let fixture = include_str!("fixtures/animation-camera-source-native.txt");
    replay_protocol_timer_fixture(fixture);
    replay_protocol_endpoint_fixture(fixture);
    let source = protocol_camera(&Handle::default()).into_cast::<dyn AisAnimationCameraSource>();
    let alias = source.target_view_ref();
    assert_eq!(
        protocol_fixture_row(fixture, 2, 0),
        vec![
            2,
            0,
            i32::from(alias.is_null()),
            i32::from(alias.is_same_reference(&source.target_view_ref()))
        ]
    );
}
fn protocol_fixture_row(fixture: &str, group: i32, frame: i32) -> Vec<i32> {
    fixture
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|value| value.parse::<i32>().unwrap())
                .collect::<Vec<i32>>()
        })
        .find(|row| row[0] == group && row[1] == frame)
        .unwrap()
}
fn replay_protocol_timer_fixture(fixture: &str) {
    let camera = protocol_camera(&Handle::default());
    let source = camera.cast::<dyn AisAnimationCameraSource>();
    let timeline = source.cast::<dyn AisAnimationSource>();
    for frame in 0..2 {
        camera
            .timer_failure
            .store(i32::from(frame == 0), Ordering::Relaxed);
        let result = timeline.update_timer();
        let row = vec![
            0,
            frame,
            i32::from(result.is_err()),
            camera.timer_calls.load(Ordering::Relaxed),
            i32::from(camera.is_kind(AisAnimationCamera::static_type())),
            i32::from(camera.is_kind(AisAnimation::static_type())),
        ];
        assert_eq!(row, protocol_fixture_row(fixture, 0, frame));
    }
}
fn replay_protocol_endpoint_fixture(fixture: &str) {
    let source = protocol_camera(&Handle::default()).into_cast::<dyn AisAnimationCameraSource>();
    let camera = Handle::new(Graphic3dCamera::default());
    let mut other = Handle::default();
    let start = source.camera_start_ref();
    let end = source.camera_end_ref();
    for frame in 0..4 {
        assign_protocol_fixture_endpoints(source.get().unwrap(), &camera, &mut other, frame);
        let row =
            protocol_endpoint_row(source.get().unwrap(), &start, &end, &camera, &other, frame);
        assert_eq!(row, protocol_fixture_row(fixture, 1, frame));
    }
}
fn assign_protocol_fixture_endpoints(
    source: &dyn AisAnimationCameraSource,
    camera: &Handle<Graphic3dCamera>,
    other: &mut Handle<Graphic3dCamera>,
    frame: i32,
) {
    match frame {
        1 => {
            source.set_camera_start(camera);
            source.set_camera_end(camera);
        }
        2 => {
            *other = Handle::new(Graphic3dCamera::default());
            source.set_camera_start(other);
        }
        3 => source.set_camera_end(&Handle::default()),
        _ => {}
    }
}

type ProtocolEndpointAlias<'a> =
    rustycad_tkernel::standard::HandleFieldRef<'a, AisAnimationCameraFields, Graphic3dCamera>;
fn protocol_endpoint_row(
    source: &dyn AisAnimationCameraSource,
    start: &ProtocolEndpointAlias<'_>,
    end: &ProtocolEndpointAlias<'_>,
    camera: &Handle<Graphic3dCamera>,
    other: &Handle<Graphic3dCamera>,
    frame: i32,
) -> Vec<i32> {
    vec![
        1,
        frame,
        i32::from(start.is_null()),
        i32::from(end.is_null()),
        camera.ref_count(),
        other.ref_count(),
        i32::from(start.is_same_reference(&source.camera_start_ref())),
        i32::from(end.is_same_reference(&source.camera_end_ref())),
    ]
}
