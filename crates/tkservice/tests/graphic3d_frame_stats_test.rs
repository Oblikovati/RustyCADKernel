//! Graphic3d_FrameStats formatting and the data frame operations.
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dFrameStats, Graphic3dFrameStatsBase, Graphic3dFrameStatsCounter as Counter,
    Graphic3dFrameStatsData, Graphic3dFrameStatsDataTmp, Graphic3dFrameStatsDict,
    Graphic3dFrameStatsMembers, Graphic3dFrameStatsTimer as Timer,
    Graphic3dRenderingParamsPerfCounters as Perf,
};

#[derive(Default)]
struct FakeFrameStats {
    transient: StandardTransient,
    base: Graphic3dFrameStatsBase,
}
rustycad_tkernel::impl_transient!(FakeFrameStats, "FakeFrameStats", dyn Graphic3dFrameStats, transient,
    interfaces: [dyn Graphic3dFrameStats]);
impl Graphic3dFrameStats for FakeFrameStats {
    fn frame_stats_base(&self) -> &Graphic3dFrameStatsBase {
        &self.base
    }
}

fn text_of(text: &AsciiString) -> String {
    String::from_utf8_lossy(text.as_bytes()).into_owned()
}

fn stats_with_frame() -> Handle<FakeFrameStats> {
    let stats = Handle::new(FakeFrameStats::default());
    stats.active_data_frame(&mut |tmp| {
        *tmp.change_frame_rate() = 59.94;
        tmp[Counter::NbLayers] = 1500;
        tmp[Counter::NbLayersNotCulled] = 3;
        tmp[Timer::ElapsedFrame] = 3725.5;
        tmp[Timer::CpuFrame] = 0.016;
    });
    let active = (*stats.base.fields.read().counters_tmp).clone();
    stats.change_data_frames(&mut |frames| frames[0] = active.clone());
    stats
}

#[test]
fn formats_text_lines() {
    let stats = stats_with_frame();
    assert_eq!(
        text_of(&stats.format_stats(Perf::FRAME_RATE)),
        "FPS:      59.9\n"
    );
    let text = text_of(&stats.format_stats(Perf(Perf::LAYERS.0 | Perf::FRAME_TIME.0)));
    assert_eq!(
        text,
        "Layers:       1.5k\nRendered\n    Layers:     3 \nTimers Average\n Elapsed Frame: 01:02:05\n     CPU Frame:  16.0 ms\n"
    );
    stats.set_long_line_format(true);
    let text = text_of(&stats.format_stats(Perf::LAYERS));
    assert_eq!(text, "Layers:    1.5k [rendered:     3]\n");
    assert!(stats.has_culled_layers() && !stats.has_culled_structs());
}

#[test]
fn formats_dictionary_in_place() {
    let stats = stats_with_frame();
    let mut dict = Graphic3dFrameStatsDict::default();
    dict.add("FPS".into(), "old".into());
    stats.format_stats_dict(
        &mut dict,
        Perf(Perf::FRAME_RATE.0 | Perf::LAYERS.0 | Perf::FRAME_TIME.0),
    );
    let pairs: Vec<(String, String)> = dict
        .items()
        .map(|(k, v)| (text_of(k), text_of(v)))
        .collect();
    let expected = [
        ("FPS", "6e+01"),
        ("Layers", "1500"),
        ("Rendered layers", "3"),
        ("Elapsed Frame (average)", "01:02:05"),
        ("CPU Frame (average)", "2e+01"),
    ];
    assert_eq!(pairs, expected.map(|(k, v)| (k.to_string(), v.to_string())));
}

#[test]
fn data_frames_flush_fill_and_reset() {
    let mut tmp = Graphic3dFrameStatsDataTmp::default();
    tmp[Timer::CpuFrame] = 0.5;
    tmp.flush_timers(2, true);
    assert_eq!(tmp.timer_value(Timer::CpuFrame), 0.25);
    let mut max = Graphic3dFrameStatsData::default();
    max.fill_max(&tmp);
    assert_eq!(max.timer_value(Timer::CpuFrame), 0.5);
    tmp[Counter::NbStructs] = 4;
    max.fill_max(&tmp);
    assert_eq!(max.counter_value(Counter::NbStructs), 4);
    max.reset();
    assert_eq!(max, Graphic3dFrameStatsData::default());
    tmp.reset();
    assert_eq!(tmp.frame_rate(), -1.0);
}
