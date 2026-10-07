//! `Media_BufferPool` and `Media_Packet` as the build without FFmpeg compiles them: no pool
//! and no packet object, so every query gives the empty value.
use super::{MediaAvBufferPool, MediaAvBufferRef, MediaAvPacket};
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// The fields of a buffer pool.
#[derive(Clone, Copy, Debug, Default)]
pub struct MediaBufferPoolFields {
    /// `myPool`: always null without FFmpeg.
    pub pool: Option<MediaAvBufferPool>,
    /// `myBufferSize`.
    pub buffer_size: i32,
}

/// A pool of frame buffers; `MediaBufferPool::default().buffer_size() == 0`.
#[derive(Default)]
pub struct MediaBufferPool {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaBufferPoolFields>,
}
rustycad_tkernel::impl_transient!(
    MediaBufferPool,
    "Media_BufferPool",
    StandardTransient,
    transient
);

impl MediaBufferPool {
    /// `Release`: drops a pool (none without FFmpeg, so the size is kept); `pool.release()`.
    pub fn release(&self) {
        let mut fields = self.fields.write();
        if fields.pool.is_some() {
            *fields = MediaBufferPoolFields::default();
        }
    }
    /// `Init`: a pool of buffers of `buffer_size` bytes; true for the current size or 0;
    /// otherwise the size is recorded and, without FFmpeg, false. Example: `pool.init(1024)`.
    pub fn init(&self, buffer_size: i32) -> bool {
        if self.fields.read().buffer_size == buffer_size {
            return true;
        }
        self.release();
        if buffer_size == 0 {
            return true;
        }
        let mut fields = self.fields.write();
        fields.buffer_size = buffer_size;
        fields.pool.is_some()
    }
    /// `BufferSize`; `pool.buffer_size()`.
    pub fn buffer_size(&self) -> i32 {
        self.fields.read().buffer_size
    }
    /// `GetBuffer`: null without FFmpeg; `pool.get_buffer().is_none()`.
    pub fn get_buffer(&self) -> Option<MediaAvBufferRef> {
        None
    }
}

/// The fields of a packet.
#[derive(Clone, Copy, Debug, Default)]
pub struct MediaPacketFields {
    /// `myPacket`: always null without FFmpeg.
    pub packet: Option<MediaAvPacket>,
    /// `myDurationSec` (uninitialised natively until set; 0 here).
    pub duration_sec: f64,
}

/// A demuxed packet; `MediaPacket::default().size() == 0`.
#[derive(Default)]
pub struct MediaPacket {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaPacketFields>,
}
rustycad_tkernel::impl_transient!(MediaPacket, "Media_Packet", StandardTransient, transient);

impl MediaPacket {
    /// `Unref`: nothing without FFmpeg; `packet.unref()`.
    pub fn unref(&self) {}
    /// `Packet` / `ChangePacket`: null without FFmpeg; `packet.packet().is_none()`.
    pub fn packet(&self) -> Option<MediaAvPacket> {
        self.fields.read().packet
    }
    /// `Data` / `ChangeData`: null without FFmpeg; `packet.data().is_none()`.
    pub fn data(&self) -> Option<&[u8]> {
        None
    }
    /// `Size`: 0 without FFmpeg; `packet.size()`.
    pub fn size(&self) -> i32 {
        0
    }
    /// `Pts`: 0 without FFmpeg; `packet.pts()`.
    pub fn pts(&self) -> i64 {
        0
    }
    /// `Dts`: 0 without FFmpeg; `packet.dts()`.
    pub fn dts(&self) -> i64 {
        0
    }
    /// `Duration`: 0 without FFmpeg; `packet.duration()`.
    pub fn duration(&self) -> i64 {
        0
    }
    /// `DurationSeconds`; `packet.duration_seconds()`.
    pub fn duration_seconds(&self) -> f64 {
        self.fields.read().duration_sec
    }
    /// `SetDurationSeconds`; `packet.set_duration_seconds(0.04)`.
    pub fn set_duration_seconds(&self, duration: f64) {
        self.fields.write().duration_sec = duration;
    }
    /// `StreamIndex`: 0 without FFmpeg; `packet.stream_index()`.
    pub fn stream_index(&self) -> i32 {
        0
    }
    /// `IsKeyFrame`: false without FFmpeg; `packet.is_key_frame()`.
    pub fn is_key_frame(&self) -> bool {
        false
    }
    /// `SetKeyFrame`: nothing without FFmpeg; `packet.set_key_frame()`.
    pub fn set_key_frame(&self) {}
}
