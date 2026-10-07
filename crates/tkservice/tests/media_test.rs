//! The Media package without FFmpeg: the player context and its worker thread, the format
//! context and the frame dump.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::media::{
    MediaFormatContext, MediaFrame, MediaIFrameQueue, MediaPlayerContext,
};
use std::sync::Arc;

/// A frame queue that hands out one frame.
struct FakeFrameQueue {
    frame: Handle<MediaFrame>,
}
impl MediaIFrameQueue for FakeFrameQueue {
    fn lock_frame(&self) -> Handle<MediaFrame> {
        self.frame.clone()
    }
    fn release_frame(&self, _frame: &Handle<MediaFrame>) {}
}

#[test]
fn player_handles_events_without_ffmpeg() {
    let queue: Arc<dyn MediaIFrameQueue> = Arc::new(FakeFrameQueue {
        frame: Handle::new(MediaFrame::default()),
    });
    let player = MediaPlayerContext::new(Some(queue));
    assert!(player.has_frame_queue() && player.to_force_rgb());
    player.set_input(&AsciiString::from("missing.mp4"), true);
    assert_eq!(player.playback_state(), (true, 0.0, 0.0));
    assert_eq!(player.play_pause(), (false, 0.0, 0.0));
    player.seek(3.0);
    player.pause();
    player.resume();
    player.set_force_rgb(false);
    assert!(!player.to_force_rgb());
    drop(player);
}

#[test]
fn inputs_do_not_open_without_ffmpeg() {
    let format = MediaFormatContext::default();
    assert!(!format.open_input(&AsciiString::from("clip.mp4")));
    assert_eq!(
        format.stream(0).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    let mut info = AsciiString::from("old");
    assert!(
        MediaPlayerContext::dump_first_frame(&AsciiString::from("clip.mp4"), &mut info).is_null()
    );
    assert!(info.is_empty());
    let text = MediaFormatContext::format_time_progress(3725.0, 7322.0);
    assert_eq!(
        String::from_utf8_lossy(text.as_bytes()),
        "01:02:05 / 02:02:02"
    );
}
