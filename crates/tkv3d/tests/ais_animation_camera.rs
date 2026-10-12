//! Camera animation preserves endpoint identity and the existing live view camera.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{graphic3d::Graphic3dCamera, null_driver::NullGraphicDriver};
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::{ais::*, v3d::V3dViewer};
fn view() -> Handle<dyn V3dViewSource> {
    V3dViewer::new(&NullGraphicDriver::new_handle())
        .unwrap()
        .create_view()
        .unwrap()
}
fn endpoints() -> (Handle<Graphic3dCamera>, Handle<Graphic3dCamera>) {
    let start = Handle::new(Graphic3dCamera::default());
    start.set_eye(&Point3::new(0., 0., 20.)).unwrap();
    start.set_center(&Point3::default()).unwrap();
    start.set_up(&Direction3::new(0., 1., 0.).unwrap());
    start.set_scale(20.);
    let end = Handle::new(Graphic3dCamera::default());
    end.copy(&start);
    end.set_eye(&Point3::new(8., 0., 20.)).unwrap();
    end.set_center(&Point3::new(8., 0., 0.)).unwrap();
    end.set_scale(40.);
    (start, end)
}
#[test]
fn camera_handle_members_accept_detached_targets_and_keep_native_rtti() {
    let target = view();
    let animation = Handle::new(AisAnimationCamera::new("orbit", &target));
    assert_eq!(animation.target_view(), target);
    assert!(animation.camera_start().is_null() && animation.camera_end().is_null());
    assert!(
        animation.is_kind_name("AIS_Animation") && animation.is_kind_name("AIS_AnimationCamera")
    );
    let (start, end) = endpoints();
    animation.set_camera_start(&start);
    animation.set_camera_end(&end);
    assert_eq!(animation.camera_start(), start);
    assert_eq!(animation.camera_end(), end);
    let other = view();
    animation.set_view(&other);
    assert_eq!(animation.target_view(), other);
    animation.set_view(&Handle::default());
    animation.start(false).unwrap();
    animation.update(0.).unwrap();
    assert!(animation.target_view().is_null());
    assert_eq!(
        animation
            .cast::<dyn AisAnimationSource>()
            .cast::<AisAnimationCamera>(),
        animation
    );
}
#[test]
fn animation_mutates_live_camera_preserving_identity_and_immediate_update_mode() {
    let target = view();
    let live = target.camera();
    let (start, end) = endpoints();
    let animation = AisAnimationCamera::new("orbit", &target);
    animation.set_camera_start(&start);
    animation.set_camera_end(&end);
    animation.set_own_duration(2.);
    for immediate in [false, true] {
        target.set_immediate_update(immediate);
        animation.start(false).unwrap();
        assert!(animation.update(1.).unwrap());
        assert_eq!(target.camera(), live);
        assert_eq!(live.eye(), Point3::new(4., 0., 20.));
        assert_eq!(live.center(), Point3::new(4., 0., 0.));
        assert_eq!(live.scale(), 30.);
        assert_eq!(target.set_immediate_update(immediate), immediate);
    }
    end.set_scale(60.);
    animation.update(1.).unwrap();
    assert_eq!(live.scale(), 40.);
    animation.pause().unwrap();
    end.set_scale(80.);
    animation.update(1.).unwrap();
    assert_eq!(live.scale(), 40.);
}
#[test]
fn missing_endpoints_are_noops_and_zero_duration_applies_end_camera() {
    let target = view();
    let live = target.camera();
    let before = live.eye();
    let (start, end) = endpoints();
    let animation = AisAnimationCamera::new("orbit", &target);
    animation.set_own_duration(10.);
    animation.start(false).unwrap();
    animation.update(1.).unwrap();
    assert_eq!(live.eye(), before);
    animation.set_camera_start(&start);
    animation.update(1.).unwrap();
    assert_eq!(live.eye(), before);
    animation.set_camera_end(&end);
    animation.set_camera_start(&Handle::default());
    animation.update(1.).unwrap();
    assert_eq!(live.eye(), before);
    animation.set_camera_start(&start);
    animation.set_own_duration(0.);
    animation.start(false).unwrap();
    assert!(!animation.update(0.).unwrap());
    assert_eq!(live.eye(), end.eye());
    assert_eq!(live.scale(), end.scale());
    assert!(animation.is_stopped());
}
