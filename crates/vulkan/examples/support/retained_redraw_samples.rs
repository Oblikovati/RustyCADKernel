//! Shared fixed-count redraw/readback contract for retained Martensite performance examples.
use rustycad_tkservice::graphic3d::{GRAPHIC3D_ZLAYER_ID_DEFAULT, Graphic3dCViewMembers};
/// Measure 20 warmups and 100 complete redraw/readbacks; `measure_redraws(&scene, |_| {})`.
pub fn measure_redraws(
    scene: &crate::scene::ViewerScene,
    mut refresh: impl FnMut(usize),
) -> Vec<f64> {
    let view = scene.cview();
    view.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.set_z_layer_redraw_mode(true);
    let render = scene.render_view();
    let samples = (0..120)
        .filter_map(|frame| {
            let start = std::time::Instant::now();
            refresh(frame);
            render.try_redraw().unwrap();
            (frame >= 20).then(|| start.elapsed().as_secs_f64() * 1000.)
        })
        .collect();
    assert_timed_capture(&render.capture().unwrap());
    samples
}
fn assert_timed_capture(capture: &martensite_vulkan::FrameCapture) {
    assert!(
        capture
            .depth
            .iter()
            .all(|depth| depth.is_finite() && (0. ..=1.).contains(depth))
    );
}
/// Emit the fixed sampling contract; `report_samples("mesh-edge-solid", samples)`.
pub fn report_samples(workload: &str, samples: Vec<f64>) {
    assert_eq!(samples.len(), 100, "workload={workload}");
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{workload}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_contract_executes_every_refresh_and_retains_only_post_warmup_samples() {
        let recorder = crate::recording_device::RecordingDevice::default();
        let driver =
            martensite_vulkan::Graphic3dRenderDriver::with_backend(recorder.clone(), 128).unwrap();
        let scene = crate::scene::ViewerScene::new(&driver);
        let mut refresh_count = 0;
        let samples = measure_redraws(&scene, |_| refresh_count += 1);
        assert_eq!(refresh_count, 120);
        assert_eq!(recorder.frames.read().len(), 120);
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|value| value.is_finite() && *value > 0.));
        report_samples("sample-contract", samples);
        scene.view.remove();
    }
}
