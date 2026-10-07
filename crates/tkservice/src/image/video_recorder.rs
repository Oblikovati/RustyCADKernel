//! `Image_VideoRecorder`: records RGBA frames into a video file through FFmpeg.
//!
//! The oracle build has `USE_FFMPEG=OFF`, so the port is the branch that configuration
//! compiles (visualization-design.md section 2). Native quirk kept: `Open` succeeds and
//! does nothing, then every `PushFrame` fails.
use super::ImagePixMap;
use rustycad_tkernel::standard::{SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::HashMap;

/// The video parameters (`Image_VideoParams`).
///
/// Example: `let mut params = ImageVideoParams::default(); params.set_framerate(30);`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageVideoParams {
    /// `Format`: the container, empty to guess it from the file name.
    pub format: AsciiString,
    /// `VideoCodec`: the codec, empty for the container's default.
    pub video_codec: AsciiString,
    /// `PixelFormat`: the codec pixel format, empty for the codec's default.
    pub pixel_format: AsciiString,
    /// `Width` of a frame.
    pub width: i32,
    /// `Height` of a frame.
    pub height: i32,
    /// `FpsNum`: frame rate numerator.
    pub fps_num: i32,
    /// `FpsDen`: frame rate denominator.
    pub fps_den: i32,
    /// `VideoCodecParams`: advanced codec parameters.
    pub video_codec_params: HashMap<AsciiString, AsciiString>,
}
impl Default for ImageVideoParams {
    /// Empty texts, zero size, frame rate 0/1.
    fn default() -> Self {
        Self {
            format: AsciiString::default(),
            video_codec: AsciiString::default(),
            pixel_format: AsciiString::default(),
            width: 0,
            height: 0,
            fps_num: 0,
            fps_den: 1,
            video_codec_params: HashMap::new(),
        }
    }
}
impl ImageVideoParams {
    /// Sets the frame rate as a fraction; `params.set_framerate_fraction(25, 2)`.
    pub fn set_framerate_fraction(&mut self, numerator: i32, denominator: i32) {
        (self.fps_num, self.fps_den) = (numerator, denominator);
    }
    /// Sets an integer frame rate; `params.set_framerate(30)`.
    pub fn set_framerate(&mut self, value: i32) {
        self.set_framerate_fraction(value, 1);
    }
}

/// A rational frame rate (`Image_VideoRecorder::VideoRational`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageVideoRational {
    /// `num`.
    pub num: i32,
    /// `den`.
    pub den: i32,
}

/// A video recorder; `ImageVideoRecorder::default().frame_count() == 0`.
pub struct ImageVideoRecorder {
    transient: StandardTransient,
    /// `myImgSrcRgba`: the frame the application draws into.
    frame: ImagePixMap,
    /// `myFrameRate` and `myFrameCount`.
    state: SharedState<(ImageVideoRational, i64)>,
}
impl Default for ImageVideoRecorder {
    /// No file, frame rate 1/30, no frame recorded.
    fn default() -> Self {
        let rate = ImageVideoRational { num: 1, den: 30 };
        Self {
            transient: StandardTransient::default(),
            frame: ImagePixMap::default(),
            state: SharedState::new((rate, 0)),
        }
    }
}
rustycad_tkernel::impl_transient!(
    ImageVideoRecorder,
    "Image_VideoRecorder",
    StandardTransient,
    transient
);

impl ImageVideoRecorder {
    /// Closes the file; nothing to close without FFmpeg; `recorder.close()`.
    pub fn close(&self) {}
    /// `Open`: true without FFmpeg (native quirk: nothing is opened).
    ///
    /// Example: `recorder.open("movie.avi", &params)`.
    pub fn open(&self, _file_name: &str, _params: &ImageVideoParams) -> bool {
        true
    }
    /// The frame to draw before `push_frame`; `recorder.change_frame().init_trash(..)`.
    pub fn change_frame(&self) -> &ImagePixMap {
        &self.frame
    }
    /// The number of frames recorded; `recorder.frame_count()`.
    pub fn frame_count(&self) -> i64 {
        self.state.read().1
    }
    /// The frame rate; `recorder.frame_rate()`.
    pub fn frame_rate(&self) -> ImageVideoRational {
        self.state.read().0
    }
    /// Records the frame; false without FFmpeg; `recorder.push_frame()`.
    pub fn push_frame(&self) -> bool {
        self.write_video_frame(false)
    }
    /// `formatAvError`: the error number as text without FFmpeg; `recorder.format_av_error(-22)`.
    pub fn format_av_error(&self, error: i32) -> AsciiString {
        AsciiString::from(error.to_string().as_str())
    }
    /// `getCodecContext`: the codec context address, 0 without FFmpeg.
    ///
    /// Example: `recorder.get_codec_context() == 0`.
    pub fn get_codec_context(&self) -> usize {
        0
    }
    /// `addVideoStream`: false without FFmpeg; `recorder.add_video_stream(&params, 0)`.
    pub fn add_video_stream(&self, _params: &ImageVideoParams, _default_codec_id: i32) -> bool {
        false
    }
    /// `openVideoCodec`: false without FFmpeg; `recorder.open_video_codec(&params)`.
    pub fn open_video_codec(&self, _params: &ImageVideoParams) -> bool {
        false
    }
    /// `writeVideoFrame`: false without FFmpeg; `recorder.write_video_frame(false)`.
    pub fn write_video_frame(&self, _to_flush: bool) -> bool {
        false
    }
}
