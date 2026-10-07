//! The Media package without FFmpeg, the twin of tools/oracle/tkservice_probe_media.cxx.
use super::common::{Reader, end, flag, int, real, text};
use super::tried;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
use rustycad_tkservice::media::{
    MediaBufferPool, MediaCodecContext, MediaFormatContext, MediaFrame, MediaPacket, MediaScaler,
    MediaTimer,
};

fn frame_values(frame: &MediaFrame) {
    flag(frame.is_empty());
    [frame.size_x(), frame.size_y(), frame.format()]
        .into_iter()
        .for_each(int);
    flag(frame.is_full_range_yuv());
    flag(frame.plane(0).is_none());
    int(frame.line_size(0));
    int(frame.best_effort_timestamp() as i32);
    real(frame.pts());
    real(f64::from(frame.pixel_aspect_ratio()));
    flag(frame.is_locked());
}

fn frames(code: i32) -> (Handle<MediaFrame>, Handle<MediaFrame>) {
    let (frame, other) = (
        Handle::new(MediaFrame::default()),
        Handle::new(MediaFrame::default()),
    );
    frame_values(&frame);
    frame.set_pts(2.5);
    frame.set_pixel_aspect_ratio(1.25);
    frame.set_locked(true);
    MediaFrame::swap(&frame, &other);
    real(frame.pts());
    real(f64::from(frame.pixel_aspect_ratio()));
    flag(frame.is_locked());
    flag(frame.frame().is_none());
    flag(frame.init_wrapper(&Handle::default()));
    flag(frame.init_wrapper(&Handle::new(ImagePixMap::default())));
    int(MediaFrame::format_ffmpeg_2_occt(code) as i32);
    int(MediaFrame::format_occt_2_ffmpeg(ImageFormat::Rgb));
    (frame, other)
}

fn packet() -> Handle<MediaPacket> {
    let packet = Handle::new(MediaPacket::default());
    int(packet.size());
    [packet.pts(), packet.dts(), packet.duration()]
        .into_iter()
        .for_each(|v| int(v as i32));
    int(packet.stream_index());
    packet.set_key_frame();
    flag(packet.is_key_frame());
    flag(packet.data().is_none());
    packet.set_duration_seconds(0.04);
    real(packet.duration_seconds());
    packet
}

fn contexts(frame: &Handle<MediaFrame>, other: &Handle<MediaFrame>, packet: &Handle<MediaPacket>) {
    let scaler = MediaScaler::default();
    flag(scaler.init(Vec2::new(4, 4), 0, Vec2::new(2, 2), 0));
    flag(scaler.convert(frame, other));
    flag(scaler.is_valid());
    let codec = MediaCodecContext::default();
    [codec.size_x(), codec.size_y(), codec.stream_index()]
        .into_iter()
        .for_each(int);
    flag(codec.can_process_packet(packet));
    flag(codec.can_process_packet(&Handle::default()));
    flag(codec.send_packet(packet));
    flag(codec.receive_frame(frame));
    flag(codec.context().is_none());
    let format = MediaFormatContext::default();
    int(format.nb_steams() as i32);
    tried(format.stream(0).map(|_| ()));
    text(format.stream_info(0, None).as_bytes());
    flag(format.read_packet(packet));
    flag(format.seek_stream(0, 1.0, false));
    flag(format.seek(1.0, true));
    real(format.pts_start_base());
    real(format.duration());
    flag(format.context().is_none());
}

fn timer_and_pool(r: &mut Reader<'_>, t: f64, d: f64) {
    let timer = MediaTimer::default();
    timer.seek(t);
    real(timer.elapsed_time());
    timer.set_playback_speed(d);
    real(timer.playback_speed());
    real(timer.elapsed_time());
    timer.pause();
    real(timer.elapsed_time());
    timer.stop();
    real(timer.elapsed_time());
    flag(timer.is_started());
    let pool = MediaBufferPool::default();
    for _ in 0..r.integer() {
        flag(pool.init(r.integer()));
        int(pool.buffer_size());
    }
    flag(pool.get_buffer().is_none());
    pool.release();
    int(pool.buffer_size());
}

/// `sv_media`: the time texts, a stopped timer, a buffer pool and the FFmpeg-free objects.
pub fn media_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, d, code) = (r.real(), r.real(), r.integer());
    text(MediaFormatContext::format_time(t).as_bytes());
    text(MediaFormatContext::format_time_progress(t, d).as_bytes());
    text(MediaFormatContext::format_av_error_description(code).as_bytes());
    real(MediaFormatContext::format_units_to_seconds(i64::from(code)));
    int(MediaFormatContext::seconds_to_units(t) as i32);
    timer_and_pool(&mut r, t, d);
    let (frame, other) = frames(code);
    let packet = packet();
    contexts(&frame, &other, &packet);
    end();
}
