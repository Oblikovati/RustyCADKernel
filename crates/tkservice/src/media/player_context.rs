//! `Media_IFrameQueue` and `Media_PlayerContext`: the video player with its worker thread, as
//! the build without FFmpeg compiles it (no input opens, so the worker only handles events).
use super::{MediaFormatContext, MediaFrame, MediaTimer};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::{Osd, SignalMode};
use rustycad_tkernel::standard::{Handle, StandardCondition, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

/// `Media_IFrameQueue`: the frames a player fills (implemented by the viewer).
pub trait MediaIFrameQueue: Send + Sync {
    /// `LockFrame`: a free frame to fill, or null.
    fn lock_frame(&self) -> Handle<MediaFrame>;
    /// `ReleaseFrame`: hands a filled frame back.
    fn release_frame(&self, frame: &Handle<MediaFrame>);
}

/// `Media_PlayerEvent` (private natively): the pending playback event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MediaPlayerEvent {
    /// `Media_PlayerEvent_NONE`.
    None,
    /// `Media_PlayerEvent_PAUSE`.
    Pause,
    /// `Media_PlayerEvent_RESUME`.
    Resume,
    /// `Media_PlayerEvent_SEEK`.
    Seek,
    /// `Media_PlayerEvent_NEXT`.
    Next,
}

/// The state behind `myMutex`.
struct PlayerState {
    timer: MediaTimer,
    duration: f64,
    input_path: AsciiString,
    seek_to: f64,
    play_event: MediaPlayerEvent,
}

/// The state shared with the worker thread.
struct PlayerShared {
    frame_queue: Option<Arc<dyn MediaIFrameQueue>>,
    state: Mutex<PlayerState>,
    wake_event: StandardCondition,
    next_event: StandardCondition,
    to_shut_down: AtomicBool,
}

impl PlayerShared {
    fn lock(&self) -> MutexGuard<'_, PlayerState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
    /// `pushPlayEvent` (under the lock).
    fn push_play_event(
        state: &mut PlayerState,
        wake_event: &StandardCondition,
        event: MediaPlayerEvent,
    ) {
        state.play_event = event;
        wake_event.set();
    }
}

/// A video player; `MediaPlayerContext::new(None)` starts its worker thread.
pub struct MediaPlayerContext {
    transient: StandardTransient,
    shared: Arc<PlayerShared>,
    /// `myToForceRgb`.
    to_force_rgb: AtomicBool,
    /// `myThread`.
    thread: Mutex<Option<JoinHandle<()>>>,
}
rustycad_tkernel::impl_transient!(
    MediaPlayerContext,
    "Media_PlayerContext",
    StandardTransient,
    transient
);

impl Drop for MediaPlayerContext {
    /// The destructor stops and joins the worker thread.
    fn drop(&mut self) {
        self.shared.to_shut_down.store(true, Ordering::SeqCst);
        self.shared.wake_event.set();
        let handle = self
            .thread
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(handle) = handle {
            // A worker panic is already reported by the thread; the player is going away.
            let _ = handle.join();
        }
    }
}

impl MediaPlayerContext {
    /// A player over a frame queue (may be absent) with its worker thread started;
    /// `MediaPlayerContext::new(None)`.
    pub fn new(frame_queue: Option<Arc<dyn MediaIFrameQueue>>) -> Self {
        let state = PlayerState {
            timer: MediaTimer::default(),
            duration: 0.0,
            input_path: AsciiString::default(),
            seek_to: 0.0,
            play_event: MediaPlayerEvent::None,
        };
        let shared = Arc::new(PlayerShared {
            frame_queue,
            state: Mutex::new(state),
            wake_event: StandardCondition::new(false),
            next_event: StandardCondition::new(false),
            to_shut_down: AtomicBool::new(false),
        });
        let worker = shared.clone();
        let thread = std::thread::spawn(move || do_thread_loop(&worker));
        Self {
            transient: StandardTransient::default(),
            shared,
            to_force_rgb: AtomicBool::new(true),
            thread: Mutex::new(Some(thread)),
        }
    }
    /// `DumpFirstFrame(src, mediaInfo)`: null without FFmpeg (the input does not open);
    /// `MediaPlayerContext::dump_first_frame(&path, &mut info)`.
    pub fn dump_first_frame(
        src_video: &AsciiString,
        media_info: &mut AsciiString,
    ) -> Handle<MediaFrame> {
        *media_info = AsciiString::default();
        let format = MediaFormatContext::default();
        let _ = format.open_input(src_video);
        Handle::default()
    }
    /// `DumpFirstFrame(src, outImage, mediaInfo, maxSize)`: false without FFmpeg;
    /// `MediaPlayerContext::dump_first_frame_to_image(&path, &out, &mut info, 0)`.
    pub fn dump_first_frame_to_image(
        src_video: &AsciiString,
        out_image: &AsciiString,
        media_info: &mut AsciiString,
        max_size: i32,
    ) -> bool {
        let _ = (out_image, max_size);
        // The first frame is always null without FFmpeg, so nothing is scaled or saved.
        let _ = Self::dump_first_frame(src_video, media_info);
        false
    }
    /// `SetInput`: hands the next input to the worker, waiting for it when asked;
    /// `player.set_input(&path, true)`.
    pub fn set_input(&self, input_path: &AsciiString, to_wait: bool) {
        {
            let mut state = self.shared.lock();
            if to_wait {
                self.shared.next_event.reset();
            }
            state.input_path = input_path.clone();
            state.play_event = MediaPlayerEvent::Next;
            self.shared.wake_event.set();
        }
        if to_wait {
            self.shared.next_event.wait();
        }
    }
    /// `PlaybackState`: (paused, progress, duration); `player.playback_state()`.
    pub fn playback_state(&self) -> (bool, f64, f64) {
        let state = self.shared.lock();
        (
            !state.timer.is_started(),
            state.timer.elapsed_time(),
            state.duration,
        )
    }
    /// `PlayPause`: toggles the playback; (paused, progress, duration) as it becomes;
    /// `player.play_pause()`.
    pub fn play_pause(&self) -> (bool, f64, f64) {
        let mut state = self.shared.lock();
        let (progress, duration) = (state.timer.elapsed_time(), state.duration);
        let pause = state.timer.is_started();
        let event = if pause {
            MediaPlayerEvent::Pause
        } else {
            MediaPlayerEvent::Resume
        };
        PlayerShared::push_play_event(&mut state, &self.shared.wake_event, event);
        (pause, progress, duration)
    }
    /// `Seek`; `player.seek(12.5)`.
    pub fn seek(&self, position: f64) {
        let mut state = self.shared.lock();
        state.seek_to = position;
        PlayerShared::push_play_event(&mut state, &self.shared.wake_event, MediaPlayerEvent::Seek);
    }
    /// `Pause`; `player.pause()`.
    pub fn pause(&self) {
        PlayerShared::push_play_event(
            &mut self.shared.lock(),
            &self.shared.wake_event,
            MediaPlayerEvent::Pause,
        );
    }
    /// `Resume`; `player.resume()`.
    pub fn resume(&self) {
        PlayerShared::push_play_event(
            &mut self.shared.lock(),
            &self.shared.wake_event,
            MediaPlayerEvent::Resume,
        );
    }
    /// `ToForceRgb`; `player.to_force_rgb()`.
    pub fn to_force_rgb(&self) -> bool {
        self.to_force_rgb.load(Ordering::SeqCst)
    }
    /// `SetForceRgb`; `player.set_force_rgb(false)`.
    pub fn set_force_rgb(&self, to_force: bool) {
        self.to_force_rgb.store(to_force, Ordering::SeqCst);
    }
    /// Whether a frame queue was given (`myFrameQueue != NULL`); `player.has_frame_queue()`.
    pub fn has_frame_queue(&self) -> bool {
        self.shared.frame_queue.is_some()
    }
}

/// `doThreadLoop`: waits for events; a new input is taken and opened (which fails without
/// FFmpeg, so no packet is ever decoded); it ends at shut-down.
fn do_thread_loop(shared: &PlayerShared) {
    Osd::set_thread_local_signal(SignalMode::Set, false);
    loop {
        shared.wake_event.wait();
        shared.wake_event.reset();
        if shared.to_shut_down.load(Ordering::SeqCst) {
            return;
        }
        let input = {
            let mut state = shared.lock();
            let input = std::mem::take(&mut state.input_path);
            if state.play_event == MediaPlayerEvent::Next {
                state.play_event = MediaPlayerEvent::None;
            }
            input
        };
        shared.next_event.set();
        if input.is_empty() {
            continue;
        }
        let format = MediaFormatContext::default();
        if !format.open_input(&input) {
            continue;
        }
        // Natively the video stream is found by FFmpeg code; without it there is none.
        let mut message = AsciiString::from("FFmpeg: no video stream in '");
        message.append(&input);
        message.append(&AsciiString::from("'"));
        Message::send_fail_text(&message);
    }
}
