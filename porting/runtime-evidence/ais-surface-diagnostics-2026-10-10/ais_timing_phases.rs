//! Temporary phase probe; backend time includes allocation, GPU waits, readback and scheduling.
#[path = "../tests/support/ais_scene.rs"] mod scene;
#[path = "../tests/support/view_setup.rs"] mod view_setup;
#[path = "support/device_options.rs"] mod device_options;
use martensite_vulkan::*;
use std::{time::Instant, sync::{Arc,atomic::{AtomicU64,Ordering}}};
struct TimedVulkan { renderer: VulkanRenderer, elapsed_ns: Arc<AtomicU64> }
impl RenderBackend for TimedVulkan {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let start=Instant::now();
        let capture=self.renderer.render(frame);
        self.elapsed_ns.store(start.elapsed().as_nanos() as u64,Ordering::Relaxed);
        capture
    }
}
fn main() {
    let mut options=device_options::validation_options(); options.validation=false;
    let renderer=VulkanRenderer::new(options).unwrap();
    println!("MARTENSITE_DEVICE={}",renderer.device_name());
    let limit=256; let elapsed_ns=Arc::new(AtomicU64::new(0));
    let driver=Graphic3dRenderDriver::with_backend(TimedVulkan{renderer,elapsed_ns:elapsed_ns.clone()},limit).unwrap();
    let scene=scene::AisViewerScene::new(&driver);
    scene.window.set_size(256,256); scene.view.must_be_resized();
    for _ in 0..20 { cycle(&scene,&elapsed_ns); }
    let before=std::fs::read_to_string("/proc/thread-self/schedstat").unwrap();
    let samples:Vec<[f64;6]>=(0..100).map(|_|cycle(&scene,&elapsed_ns)).collect();
    let after=std::fs::read_to_string("/proc/thread-self/schedstat").unwrap();
    assert_eq!(scene.object.computations.load(Ordering::Relaxed),1);
    assert_eq!(scene.viewer.active_views().len(),1);
    assert!(scene.render_view().capture().is_ok());
    println!("MARTENSITE_PHASES={{\"samples_ms\":{samples:?},\"schedstat_before\":\"{}\",\"schedstat_after\":\"{}\"}}",before.trim(),after.trim());
    scene.view.remove();
}
fn cycle(scene:&scene::AisViewerScene, backend:&AtomicU64)->[f64;6] {
    let start=Instant::now();
    scene.context.erase(&scene.object.cast(),false).unwrap(); let erased=start.elapsed();
    scene.render_view().try_redraw().unwrap(); let first=start.elapsed();
    let first_backend=backend.load(Ordering::Relaxed);
    scene.context.display(&scene.object.cast(),false).unwrap(); let displayed=start.elapsed();
    scene.render_view().try_redraw().unwrap(); let last=start.elapsed();
    let backend_ms=(first_backend+backend.load(Ordering::Relaxed)) as f64/1e6;
    [erased.as_secs_f64()*1e3,(first-erased).as_secs_f64()*1e3,(displayed-first).as_secs_f64()*1e3,(last-displayed).as_secs_f64()*1e3,backend_ms,last.as_secs_f64()*1e3]
}
