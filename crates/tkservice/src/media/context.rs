//! `Media_CodecContext` and `Media_FormatContext` as the build without FFmpeg compiles them:
//! no input opens and no codec initializes; the time formatting is complete.
use super::{
    MediaAvCodec, MediaAvCodecContext, MediaAvFormatContext, MediaAvRational, MediaAvStream,
    MediaFrame, MediaPacket,
};
use crate::graphic3d::split_time;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The fields of a codec context.
#[derive(Clone, Copy, Debug)]
pub struct MediaCodecContextFields {
    /// `myCodecCtx`: always null without FFmpeg.
    pub codec_ctx: Option<MediaAvCodecContext>,
    /// `myCodec`: always null without FFmpeg.
    pub codec: Option<MediaAvCodec>,
    /// `myPtsStartBase`.
    pub pts_start_base: f64,
    /// `myPtsStartStream`.
    pub pts_start_stream: f64,
    /// `myTimeBase`.
    pub time_base: f64,
    /// `myStreamIndex`.
    pub stream_index: i32,
    /// `myPixelAspectRatio`.
    pub pixel_aspect_ratio: f32,
}

/// A decoder of one stream; `MediaCodecContext::default().size_x() == 0`.
pub struct MediaCodecContext {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaCodecContextFields>,
}
rustycad_tkernel::impl_transient!(
    MediaCodecContext,
    "Media_CodecContext",
    StandardTransient,
    transient
);

impl Default for MediaCodecContext {
    fn default() -> Self {
        let fields = MediaCodecContextFields {
            codec_ctx: None,
            codec: None,
            pts_start_base: 0.0,
            pts_start_stream: 0.0,
            time_base: 1.0,
            stream_index: 0,
            pixel_aspect_ratio: 1.0,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

impl MediaCodecContext {
    /// `Context`: null without FFmpeg; `codec.context().is_none()`.
    pub fn context(&self) -> Option<MediaAvCodecContext> {
        self.fields.read().codec_ctx
    }
    /// `Init(stream, ptsStartBase, nbThreads)`: no stream exists without FFmpeg.
    pub fn init(&self, stream: &MediaAvStream, pts_start_base: f64, nb_threads: i32) -> bool {
        self.init_codec(stream, pts_start_base, nb_threads, 0)
    }
    /// `Init(stream, ptsStartBase, nbThreads, codecId)`: no stream exists without FFmpeg.
    pub fn init_codec(
        &self,
        stream: &MediaAvStream,
        pts_start_base: f64,
        nb_threads: i32,
        codec_id: i32,
    ) -> bool {
        let _ = (pts_start_base, nb_threads, codec_id);
        match *stream {}
    }
    /// `Close`: nothing to close without FFmpeg; `codec.close()`.
    pub fn close(&self) {}
    /// `SizeX`: 0 without FFmpeg; `codec.size_x()`.
    pub fn size_x(&self) -> i32 {
        0
    }
    /// `SizeY`: 0 without FFmpeg; `codec.size_y()`.
    pub fn size_y(&self) -> i32 {
        0
    }
    /// `StreamIndex`; `codec.stream_index()`.
    pub fn stream_index(&self) -> i32 {
        self.fields.read().stream_index
    }
    /// `Flush`: nothing without FFmpeg; `codec.flush()`.
    pub fn flush(&self) {}
    /// `CanProcessPacket`: a packet of the codec stream; `codec.can_process_packet(&packet)`.
    pub fn can_process_packet(&self, packet: &Handle<MediaPacket>) -> bool {
        packet
            .get()
            .is_some_and(|p| self.stream_index() == p.stream_index())
    }
    /// `SendPacket`: false without FFmpeg; `codec.send_packet(&packet)`.
    pub fn send_packet(&self, packet: &Handle<MediaPacket>) -> bool {
        let _ = self.can_process_packet(packet);
        false
    }
    /// `ReceiveFrame`: false without FFmpeg; `codec.receive_frame(&frame)`.
    pub fn receive_frame(&self, frame: &Handle<MediaFrame>) -> bool {
        let _ = frame;
        false
    }
}

/// The fields of a format context.
#[derive(Clone, Copy, Debug, Default)]
pub struct MediaFormatContextFields {
    /// `myFormatCtx`: always null without FFmpeg.
    pub format_ctx: Option<MediaAvFormatContext>,
    /// `myPtsStartBase`.
    pub pts_start_base: f64,
    /// `myDuration`.
    pub duration: f64,
}

/// A demuxer of a media input; `MediaFormatContext::default().nb_steams() == 0`.
#[derive(Default)]
pub struct MediaFormatContext {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaFormatContextFields>,
}
rustycad_tkernel::impl_transient!(
    MediaFormatContext,
    "Media_FormatContext",
    StandardTransient,
    transient
);

impl MediaFormatContext {
    /// `FormatAVErrorDescription`: "AVError #<code>" without FFmpeg;
    /// `MediaFormatContext::format_av_error_description(-2)`.
    pub fn format_av_error_description(error_code: i32) -> AsciiString {
        AsciiString::from(format!("AVError #{error_code}").as_str())
    }
    /// `FormatUnitsToSeconds`: 0 without FFmpeg.
    pub fn format_units_to_seconds(time_units: i64) -> f64 {
        let _ = time_units;
        0.0
    }
    /// `UnitsToSeconds`: no rational exists without FFmpeg.
    pub fn units_to_seconds(time_base: &MediaAvRational, time_units: i64) -> f64 {
        let _ = time_units;
        match *time_base {}
    }
    /// `StreamUnitsToSeconds`: no stream exists without FFmpeg.
    pub fn stream_units_to_seconds(stream: &MediaAvStream, time_units: i64) -> f64 {
        let _ = time_units;
        match *stream {}
    }
    /// `SecondsToUnits(double)`: 0 without FFmpeg.
    pub fn seconds_to_units(time_seconds: f64) -> i64 {
        let _ = time_seconds;
        0
    }
    /// `SecondsToUnits(AVRational, double)`: no rational exists without FFmpeg.
    pub fn seconds_to_units_in(time_base: &MediaAvRational, time_seconds: f64) -> i64 {
        let _ = time_seconds;
        match *time_base {}
    }
    /// `StreamSecondsToUnits`: no stream exists without FFmpeg.
    pub fn stream_seconds_to_units(stream: &MediaAvStream, time_seconds: f64) -> i64 {
        let _ = time_seconds;
        match *stream {}
    }
    /// `FormatTime`: "hh:mm:ss", "mm:ss", "s s" or "<ms> ms" (`%g` milliseconds);
    /// `MediaFormatContext::format_time(75.0)` is "01:15".
    pub fn format_time(seconds: f64) -> AsciiString {
        let (hours, minutes, whole, millis) = split_time(seconds);
        let text = if hours > 0 {
            format!("{hours:02}:{minutes:02}:{whole:02}")
        } else if minutes > 0 {
            format!("{minutes:02}:{whole:02}")
        } else if whole > 0 {
            format!("{whole:2} s")
        } else {
            let mut text = AsciiString::from_real(millis);
            text.append(&AsciiString::from(" ms"));
            return text;
        };
        AsciiString::from(text.as_str())
    }
    /// `FormatTimeProgress`: "mm:ss / mm:ss", with hours when either has some;
    /// `MediaFormatContext::format_time_progress(5.0, 60.0)` is "00:05 / 01:00".
    pub fn format_time_progress(progress: f64, duration: f64) -> AsciiString {
        let (h1, m1, s1, _) = split_time(progress);
        let (h2, m2, s2, _) = split_time(duration);
        let text = if h1 > 0 || h2 > 0 {
            format!("{h1:02}:{m1:02}:{s1:02} / {h2:02}:{m2:02}:{s2:02}")
        } else {
            format!("{m1:02}:{s1:02} / {m2:02}:{s2:02}")
        };
        AsciiString::from(text.as_str())
    }
    /// `Context`: null without FFmpeg; `format.context().is_none()`.
    pub fn context(&self) -> Option<MediaAvFormatContext> {
        self.fields.read().format_ctx
    }
    /// `OpenInput`: reports that FFmpeg is unavailable and fails; `format.open_input(&path)`.
    pub fn open_input(&self, input: &AsciiString) -> bool {
        let _ = input;
        Message::send_fail_text(&AsciiString::from("Error: FFmpeg library is unavailable"));
        false
    }
    /// `Close`: nothing to close without FFmpeg; `format.close()`.
    pub fn close(&self) {}
    /// `NbSteams`: 0 without FFmpeg; `format.nb_steams()`.
    pub fn nb_steams(&self) -> u32 {
        0
    }
    /// `Stream`: raises `Standard_ProgramError` without FFmpeg; `format.stream(0)`.
    pub fn stream(&self, index: u32) -> Result<&MediaAvStream, Failure> {
        Err(program_error(&format!(
            "Media_FormatContext::Stream(): no stream {index} without FFmpeg"
        )))
    }
    /// `StreamInfo`: empty without FFmpeg; `format.stream_info(0, None)`.
    pub fn stream_info(&self, index: u32, codec_ctx: Option<&MediaAvCodecContext>) -> AsciiString {
        let _ = (index, codec_ctx);
        AsciiString::default()
    }
    /// `PtsStartBase`; `format.pts_start_base()`.
    pub fn pts_start_base(&self) -> f64 {
        self.fields.read().pts_start_base
    }
    /// `Duration`; `format.duration()`.
    pub fn duration(&self) -> f64 {
        self.fields.read().duration
    }
    /// `ReadPacket`: false without FFmpeg; `format.read_packet(&packet)`.
    pub fn read_packet(&self, packet: &Handle<MediaPacket>) -> bool {
        let _ = packet;
        false
    }
    /// `SeekStream`: false without FFmpeg; `format.seek_stream(0, 1.5, false)`.
    pub fn seek_stream(&self, stream_id: u32, seek_pts: f64, to_seek_back: bool) -> bool {
        let _ = (stream_id, seek_pts, to_seek_back);
        false
    }
    /// `Seek`: false without FFmpeg; `format.seek(1.5, false)`.
    pub fn seek(&self, seek_pts: f64, to_seek_back: bool) -> bool {
        let _ = (seek_pts, to_seek_back);
        false
    }
}
