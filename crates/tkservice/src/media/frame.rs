//! `Media_Frame` and `Media_Scaler` as the build without FFmpeg compiles them: a frame has no
//! FFmpeg frame, so it is always empty, and the scaler never converts.
use super::{MediaAvFrame, MediaSwsContext};
use crate::image::{ImageFormat, ImagePixMap};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};

/// The fields of a frame.
#[derive(Clone, Copy, Debug)]
pub struct MediaFrameFields {
    /// `myFrame`: always null without FFmpeg.
    pub frame: Option<MediaAvFrame>,
    /// `myFramePts`: the presentation timestamp.
    pub frame_pts: f64,
    /// `myPixelRatio`.
    pub pixel_ratio: f32,
    /// `myIsLocked`.
    pub is_locked: bool,
}

/// A decoded frame; `MediaFrame::default().is_empty()`.
pub struct MediaFrame {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaFrameFields>,
}
rustycad_tkernel::impl_transient!(MediaFrame, "Media_Frame", StandardTransient, transient);

impl Default for MediaFrame {
    /// Pts 0, pixel ratio 1, unlocked.
    fn default() -> Self {
        let fields = MediaFrameFields {
            frame: None,
            frame_pts: 0.0,
            pixel_ratio: 1.0,
            is_locked: false,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

impl MediaFrame {
    /// `FormatFFmpeg2Occt`: unknown without FFmpeg; `MediaFrame::format_ffmpeg_2_occt(2)`.
    pub fn format_ffmpeg_2_occt(format: i32) -> ImageFormat {
        let _ = format;
        ImageFormat::Unknown
    }
    /// `FormatOcct2FFmpeg`: 0 without FFmpeg; `MediaFrame::format_occt_2_ffmpeg(ImageFormat::Rgb)`.
    pub fn format_occt_2_ffmpeg(format: ImageFormat) -> i32 {
        let _ = format;
        0
    }
    /// `Swap`: exchanges the FFmpeg frames of two frames; `MediaFrame::swap(&a, &b)`.
    pub fn swap(first: &Handle<MediaFrame>, second: &Handle<MediaFrame>) {
        let (Some(a), Some(b)) = (first.get(), second.get()) else {
            return;
        };
        if first == second {
            return;
        }
        let frame = a.fields.read().frame;
        a.fields.write().frame = b.fields.read().frame;
        b.fields.write().frame = frame;
    }
    /// `IsEmpty`: always true without FFmpeg; `frame.is_empty()`.
    pub fn is_empty(&self) -> bool {
        true
    }
    /// `Unref`: nothing without FFmpeg; `frame.unref()`.
    pub fn unref(&self) {}
    /// `Size`; `frame.size()`.
    pub fn size(&self) -> Vec2<i32> {
        Vec2::new(self.size_x(), self.size_y())
    }
    /// `SizeX`: 0 without FFmpeg; `frame.size_x()`.
    pub fn size_x(&self) -> i32 {
        0
    }
    /// `SizeY`: 0 without FFmpeg; `frame.size_y()`.
    pub fn size_y(&self) -> i32 {
        0
    }
    /// `Format`: 0 without FFmpeg; `frame.format()`.
    pub fn format(&self) -> i32 {
        0
    }
    /// `IsFullRangeYUV`: true without FFmpeg; `frame.is_full_range_yuv()`.
    pub fn is_full_range_yuv(&self) -> bool {
        true
    }
    /// `Plane`: null without FFmpeg; `frame.plane(0).is_none()`.
    pub fn plane(&self, plane_id: i32) -> Option<&[u8]> {
        let _ = plane_id;
        None
    }
    /// `LineSize`: 0 without FFmpeg; `frame.line_size(0)`.
    pub fn line_size(&self, plane_id: i32) -> i32 {
        let _ = plane_id;
        0
    }
    /// `BestEffortTimestamp`: 0 without FFmpeg; `frame.best_effort_timestamp()`.
    pub fn best_effort_timestamp(&self) -> i64 {
        0
    }
    /// `Frame` / `ChangeFrame`: null without FFmpeg; `frame.frame().is_none()`.
    pub fn frame(&self) -> Option<MediaAvFrame> {
        self.fields.read().frame
    }
    /// `Pts`; `frame.pts()`.
    pub fn pts(&self) -> f64 {
        self.fields.read().frame_pts
    }
    /// `SetPts`; `frame.set_pts(1.5)`.
    pub fn set_pts(&self, pts: f64) {
        self.fields.write().frame_pts = pts;
    }
    /// `PixelAspectRatio`; `frame.pixel_aspect_ratio()`.
    pub fn pixel_aspect_ratio(&self) -> f32 {
        self.fields.read().pixel_ratio
    }
    /// `SetPixelAspectRatio`; `frame.set_pixel_aspect_ratio(1.25)`.
    pub fn set_pixel_aspect_ratio(&self, ratio: f32) {
        self.fields.write().pixel_ratio = ratio;
    }
    /// `IsLocked`; `frame.is_locked()`.
    pub fn is_locked(&self) -> bool {
        self.fields.read().is_locked
    }
    /// `SetLocked`; `frame.set_locked(true)`.
    pub fn set_locked(&self, to_lock: bool) {
        self.fields.write().is_locked = to_lock;
    }
    /// `InitWrapper`: false without FFmpeg (and for a null image); `frame.init_wrapper(&pixmap)`.
    pub fn init_wrapper(&self, pix_map: &Handle<ImagePixMap>) -> bool {
        self.unref();
        let _ = pix_map;
        false
    }
}

/// The fields of a scaler.
#[derive(Clone, Copy, Debug)]
pub struct MediaScalerFields {
    /// `mySwsContext`: always null without FFmpeg.
    pub sws_context: Option<MediaSwsContext>,
    /// `mySrcDims`.
    pub src_dims: Vec2<i32>,
    /// `mySrcFormat`.
    pub src_format: i32,
    /// `myResDims`.
    pub res_dims: Vec2<i32>,
    /// `myResFormat`.
    pub res_format: i32,
}

/// A pixel format converter; `MediaScaler::default().is_valid() == false`.
pub struct MediaScaler {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaScalerFields>,
}
rustycad_tkernel::impl_transient!(MediaScaler, "Media_Scaler", StandardTransient, transient);

impl Default for MediaScaler {
    fn default() -> Self {
        let fields = MediaScalerFields {
            sws_context: None,
            src_dims: Vec2::new(0, 0),
            src_format: 0,
            res_dims: Vec2::new(0, 0),
            res_format: 0,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

impl MediaScaler {
    /// `Release`: drops the context; `scaler.release()`.
    pub fn release(&self) {
        self.fields.write().sws_context = None;
    }
    /// `Init`: false without FFmpeg; `scaler.init(src_dims, src_format, res_dims, res_format)`.
    pub fn init(
        &self,
        src_dims: Vec2<i32>,
        src_format: i32,
        res_dims: Vec2<i32>,
        res_format: i32,
    ) -> bool {
        let _ = (src_dims, src_format, res_dims, res_format);
        false
    }
    /// `Convert`: false for null, empty or identical frames, and without FFmpeg;
    /// `scaler.convert(&source, &result)`.
    pub fn convert(&self, source: &Handle<MediaFrame>, result: &Handle<MediaFrame>) -> bool {
        let (Some(src), Some(res)) = (source.get(), result.get()) else {
            return false;
        };
        if src.is_empty() || res.is_empty() || source == result {
            return false;
        }
        // Without FFmpeg nothing converts, whatever Init answers.
        let _ = self.init(src.size(), src.format(), res.size(), res.format());
        false
    }
    /// `IsValid`: a context exists; `scaler.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.fields.read().sws_context.is_some()
    }
}
