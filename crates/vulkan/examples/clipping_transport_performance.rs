//! Completed-frame timings include genuine borrowed clipping slots and live native collection mutation.
#[allow(dead_code)]
#[path = "../tests/support/clipping_scene.rs"]
mod clipping_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/retained_redraw_samples.rs"]
mod redraw_samples;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use redraw_samples::{measure_redraws, report_samples};
const CLIP_TRANSPORT_LIVE_CASES: [&str; 3] = [
    "clip-live-append",
    "clip-live-retirement",
    "clip-live-field-selfassignment",
];
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::{Graphic3dClipPlane, Graphic3dClipPlaneSource};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in [
        "clip-global",
        "clip-derived-source",
        "clip-derived-alias-clear",
        "clip-intersection",
        "clip-union",
    ] {
        let fixture = scene::ViewerScene::new(&driver);
        clipping_scene::configure_clipping(&fixture, mode);
        report_samples(mode, measure_redraws(&fixture, |_| {}));
        fixture.view.remove();
    }
    measure_live_collections(&driver);
    Ok(())
}
fn measure_live_collections(driver: &Handle<Graphic3dRenderDriver>) {
    let plane = Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(1., 0., 0., 0.)).unwrap())
        .into_cast::<dyn Graphic3dClipPlaneSource>();
    for mode in CLIP_TRANSPORT_LIVE_CASES {
        let fixture = scene::ViewerScene::new(driver);
        clipping_scene::configure_clipping(&fixture, "clip-derived-source");
        let samples = measure_redraws(&fixture, |frame| {
            refresh_live_clipping(&fixture, &plane, mode, frame)
        });
        report_samples(mode, samples);
        fixture.view.remove();
    }
}
fn refresh_live_clipping(
    scene: &scene::ViewerScene,
    plane: &Handle<dyn Graphic3dClipPlaneSource>,
    mode: &str,
    frame: usize,
) {
    let selected = scene.view.clip_planes_ref();
    if mode == "clip-live-field-selfassignment" {
        scene.view.set_clip_planes(selected.as_ref());
        return;
    }
    let collection = selected.observe();
    if mode == "clip-live-retirement" || frame.is_multiple_of(2) {
        collection.clear();
    }
    collection.append_reference(plane);
}

#[cfg(test)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_workloads_preserve_the_selected_slot_and_real_node_ownership() {
        let device = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
        let fixture = scene::ViewerScene::new(&driver);
        clipping_scene::configure_clipping(&fixture, "clip-derived-source");
        let plane =
            Handle::new(Graphic3dClipPlane::from_equation(Vec4::new(1., 0., 0., 0.)).unwrap())
                .into_cast::<dyn Graphic3dClipPlaneSource>();
        let alias = fixture.view.clip_planes_ref();
        for mode in CLIP_TRANSPORT_LIVE_CASES {
            assert_live_clip_mutation(&fixture, &plane, alias.as_ref(), mode);
            fixture.render_view().try_redraw().unwrap();
            assert!(device.last_frame().draws()[0].clipping().is_some());
        }
        fixture.view.remove();
    }
    fn assert_live_clip_mutation(
        fixture: &scene::ViewerScene,
        plane: &Handle<dyn Graphic3dClipPlaneSource>,
        alias: &dyn rustycad_tkernel::standard::HandleReference<
            dyn rustycad_tkservice::graphic3d::Graphic3dClipPlaneSequenceSource,
        >,
        mode: &str,
    ) {
        let identity = alias.object_address();
        refresh_live_clipping(fixture, plane, mode, 0);
        refresh_live_clipping(fixture, plane, mode, 1);
        assert_eq!(alias.object_address(), identity);
        let selected = alias.observe();
        assert_eq!(
            (selected.ref_count(), selected.size(), plane.ref_count()),
            (1, 1, 2)
        );
    }
}
