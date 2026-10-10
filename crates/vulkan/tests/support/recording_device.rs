//! Named fake device preserves complete submitted frames and injects device failures.
use martensite_vulkan::*;
use rustycad_tkernel::standard::SharedState;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Default)]
pub struct RecordingDevice {
    pub frames: Arc<SharedState<Vec<RenderFrame>>>,
    pub fail: Arc<AtomicBool>,
}
impl RenderBackend for RecordingDevice {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        if self.fail.load(Ordering::Relaxed) {
            return Err(RenderError::rejected(
                "injected failure",
                "successful device submission",
            ));
        }
        self.frames.write().push(frame.clone());
        let [width, height] = frame.extent();
        let rgba = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, 91, 255]))
            .collect();
        Ok(FrameCapture {
            width,
            height,
            rgba,
            depth: vec![0.25; (width * height) as usize],
        })
    }
}
impl RecordingDevice {
    pub fn last_frame(&self) -> RenderFrame {
        self.frames.read().last().unwrap().clone()
    }
}
