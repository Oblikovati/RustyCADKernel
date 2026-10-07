//! Media: the video player classes (OCCT package Media) as the oracle build compiles them,
//! without FFmpeg (`HAVE_FFMPEG` off): the timer, buffer pool, packet, frame, scaler, codec
//! and format contexts and the player context keep their FFmpeg-free behaviour; inputs do not
//! open and no frame is ever decoded.
mod context;
mod ffmpeg;
mod frame;
mod packet;
mod player_context;
mod timer;

pub use context::{
    MediaCodecContext, MediaCodecContextFields, MediaFormatContext, MediaFormatContextFields,
};
pub use ffmpeg::{
    MediaAvBufferPool, MediaAvBufferRef, MediaAvCodec, MediaAvCodecContext, MediaAvFormatContext,
    MediaAvFrame, MediaAvPacket, MediaAvRational, MediaAvStream, MediaSwsContext,
};
pub use frame::{MediaFrame, MediaFrameFields, MediaScaler, MediaScalerFields};
pub use packet::{MediaBufferPool, MediaBufferPoolFields, MediaPacket, MediaPacketFields};
pub use player_context::{MediaIFrameQueue, MediaPlayerContext};
pub use timer::{MediaTimer, MediaTimerFields};
