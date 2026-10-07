//! The FFmpeg structures the Media headers forward-declare. The oracle build has no FFmpeg
//! (`HAVE_FFMPEG` off), so no such object ever exists: each one is an uninhabited type and
//! every pointer to one is `None`.

/// Declares uninhabited FFmpeg structure types.
macro_rules! ffmpeg_types {
    ($($(#[$doc:meta])* $name:ident;)+) => {
        $(
            $(#[$doc])*
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub enum $name {}
        )+
    };
}

ffmpeg_types! {
    /// `AVBufferPool`: an FFmpeg buffer pool (none without FFmpeg).
    MediaAvBufferPool;
    /// `AVBufferRef`: a reference to an FFmpeg buffer (none without FFmpeg).
    MediaAvBufferRef;
    /// `AVFrame`: an FFmpeg frame (none without FFmpeg).
    MediaAvFrame;
    /// `AVPacket`: an FFmpeg packet (none without FFmpeg).
    MediaAvPacket;
    /// `SwsContext`: an FFmpeg scaling context (none without FFmpeg).
    MediaSwsContext;
    /// `AVCodec`: an FFmpeg codec (none without FFmpeg).
    MediaAvCodec;
    /// `AVCodecContext`: an FFmpeg codec context (none without FFmpeg).
    MediaAvCodecContext;
    /// `AVStream`: an FFmpeg stream (none without FFmpeg).
    MediaAvStream;
    /// `AVFormatContext`: an FFmpeg format context (none without FFmpeg).
    MediaAvFormatContext;
    /// `AVRational`: an FFmpeg rational (an incomplete type without FFmpeg).
    MediaAvRational;
}
