//! `Graphic3d_FrameStats::FormatStats`: the statistics of the last data frame as text
//! lines or as a dictionary, section by section in the native order.
use super::frame_stats::Graphic3dFrameStatsFields;
use super::frame_stats_text::{
    Graphic3dFrameStatsDict, add_info_real, add_info_size, add_time_info, fixed1, format_bytes,
    format_counter, format_counter_pair, format_time, pad,
};
use super::{
    Graphic3dFrameStatsCounter as Counter, Graphic3dFrameStatsData,
    Graphic3dFrameStatsTimer as Timer, Graphic3dRenderingParamsPerfCounters as Perf,
};

/// `aValWidth`: the width of the values.
const VALUE_WIDTH: usize = 5;

/// Whether `flags` has any bit of `bit`.
fn has(flags: Perf, bit: Perf) -> bool {
    (flags.0 & bit.0) != 0
}

/// `FormatStats(PerfCounters)`: the text lines; `format_stats_text(&fields, Perf::BASIC)`.
pub(crate) fn format_stats_text(fields: &Graphic3dFrameStatsFields, flags: Perf) -> String {
    let mut out = String::new();
    let stats = fields.last_data_frame();
    push_frame_rates(&mut out, fields.is_long_line_format, stats, flags);
    push_scene_counts(&mut out, fields, flags);
    let has_immediate =
        stats[Counter::NbLayersImmediate] != 0 || stats.immediate_frame_rate() > 0.0;
    push_rendered_header(&mut out, fields.is_long_line_format, has_immediate, flags);
    push_rendered_counts(
        &mut out,
        fields.is_long_line_format,
        stats,
        has_immediate,
        flags,
    );
    if has(flags, Perf::ESTIM_MEM) {
        out.push_str("GPU Memory\n");
        format_bytes(
            &mut out,
            VALUE_WIDTH,
            "  Geometry: ",
            stats[Counter::EstimatedBytesGeom],
            "\n",
        );
        format_bytes(
            &mut out,
            VALUE_WIDTH,
            "  Textures: ",
            stats[Counter::EstimatedBytesTextures],
            "\n",
        );
        format_bytes(
            &mut out,
            VALUE_WIDTH,
            "    Frames: ",
            stats[Counter::EstimatedBytesFbos],
            "\n",
        );
    }
    if has(flags, Perf::FRAME_TIME) {
        push_timers(
            &mut out,
            stats,
            &fields.counters_max,
            has(flags, Perf::FRAME_TIME_MAX),
        );
    }
    out
}

/// The FPS line(s): one long line with both rates, or a line per rate with the
/// immediate rate in brackets when positive.
fn push_frame_rates(
    out: &mut String,
    long_line: bool,
    stats: &Graphic3dFrameStatsData,
    flags: Perf,
) {
    let compact = flags == Perf::FRAME_RATE;
    let width = |wide: usize| if compact { VALUE_WIDTH } else { wide };
    if long_line && has(flags, Perf::FRAME_RATE) && has(flags, Perf::CPU) {
        out.push_str(&format!(
            "FPS: {} [CPU: {}]\n",
            pad(&fixed1(stats.frame_rate()), width(9)),
            pad(&fixed1(stats.frame_rate_cpu()), width(10))
        ));
        return;
    }
    let rows = [
        (
            Perf::FRAME_RATE,
            "FPS:     ",
            stats.frame_rate(),
            stats.immediate_frame_rate(),
        ),
        (
            Perf::CPU,
            "CPU FPS: ",
            stats.frame_rate_cpu(),
            stats.immediate_frame_rate_cpu(),
        ),
    ];
    for (_, label, rate, immediate) in rows.into_iter().filter(|row| has(flags, row.0)) {
        out.push_str(label);
        out.push_str(&pad(&fixed1(rate), width(VALUE_WIDTH + 3)));
        if immediate > 0.0 {
            out.push_str(&format!(" ({})", fixed1(immediate)));
        }
        out.push('\n');
    }
}

/// The layer and structure counts of the scene: with the rendered count in brackets
/// in the long line format, alone otherwise.
fn push_scene_counts(out: &mut String, fields: &Graphic3dFrameStatsFields, flags: Perf) {
    let stats = fields.last_data_frame();
    let rows = [
        (
            Perf::LAYERS,
            "Layers:  ",
            Counter::NbLayers,
            Counter::NbLayersNotCulled,
            fields.has_culled_layers(),
        ),
        (
            Perf::STRUCTURES,
            "Structs: ",
            Counter::NbStructs,
            Counter::NbStructsNotCulled,
            fields.has_culled_structs(),
        ),
    ];
    for (_, label, total, rendered, culled) in rows.into_iter().filter(|row| has(flags, row.0)) {
        if !fields.is_long_line_format {
            format_counter(out, VALUE_WIDTH + 3, Some(label), stats[total], Some("\n"));
            continue;
        }
        format_counter(out, VALUE_WIDTH, Some(label), stats[total], None);
        if culled {
            format_counter(
                out,
                VALUE_WIDTH,
                Some(" [rendered: "),
                stats[rendered],
                Some("]"),
            );
        }
        out.push('\n');
    }
}

/// The "Rendered" header, before any rendered count.
fn push_rendered_header(out: &mut String, long_line: bool, has_immediate: bool, flags: Perf) {
    let detail = Perf(
        Perf::GROUPS.0 | Perf::GROUP_ARRAYS.0 | Perf::TRIANGLES.0 | Perf::LINES.0 | Perf::POINTS.0,
    );
    let scene = Perf(Perf::STRUCTURES.0 | Perf::LAYERS.0);
    if !has(flags, detail) && (long_line || !has(flags, scene)) {
        return;
    }
    out.push_str(if has_immediate {
        "Rendered          (imm.)\n"
    } else {
        "Rendered\n"
    });
}

/// The rendered counts, each with its immediate count when the frame has immediate layers.
fn push_rendered_counts(
    out: &mut String,
    long_line: bool,
    stats: &Graphic3dFrameStatsData,
    has_immediate: bool,
    flags: Perf,
) {
    let rows = [
        (
            !long_line && has(flags, Perf::LAYERS),
            "    Layers: ",
            Counter::NbLayersNotCulled,
            Counter::NbLayersImmediate,
        ),
        (
            !long_line && has(flags, Perf::STRUCTURES),
            "   Structs: ",
            Counter::NbStructsNotCulled,
            Counter::NbStructsImmediate,
        ),
        (
            has(flags, Perf::GROUPS),
            "    Groups: ",
            Counter::NbGroupsNotCulled,
            Counter::NbGroupsImmediate,
        ),
        (
            has(flags, Perf::GROUP_ARRAYS),
            "    Arrays: ",
            Counter::NbElemsNotCulled,
            Counter::NbElemsImmediate,
        ),
        (
            has(flags, Perf::GROUP_ARRAYS),
            "    [fill]: ",
            Counter::NbElemsFillNotCulled,
            Counter::NbElemsFillImmediate,
        ),
        (
            has(flags, Perf::GROUP_ARRAYS),
            "    [line]: ",
            Counter::NbElemsLineNotCulled,
            Counter::NbElemsLineImmediate,
        ),
        (
            has(flags, Perf::GROUP_ARRAYS),
            "   [point]: ",
            Counter::NbElemsPointNotCulled,
            Counter::NbElemsPointImmediate,
        ),
        (
            has(flags, Perf::GROUP_ARRAYS),
            "    [text]: ",
            Counter::NbElemsTextNotCulled,
            Counter::NbElemsTextImmediate,
        ),
        (
            has(flags, Perf::TRIANGLES),
            " Triangles: ",
            Counter::NbTrianglesNotCulled,
            Counter::NbTrianglesImmediate,
        ),
        (
            has(flags, Perf::LINES),
            "     Lines: ",
            Counter::NbLinesNotCulled,
            Counter::NbLinesImmediate,
        ),
        (
            has(flags, Perf::POINTS),
            "    Points: ",
            Counter::NbPointsNotCulled,
            Counter::NbPointsImmediate,
        ),
    ];
    for (_, label, rendered, immediate) in rows.into_iter().filter(|row| row.0) {
        format_counter_pair(
            out,
            VALUE_WIDTH,
            label,
            stats[rendered],
            stats[immediate],
            has_immediate,
        );
    }
}

/// The CPU timers shown only when their maximum is set: picking and culling when
/// positive, dynamics when non-zero (the native tests it as a boolean).
fn optional_timers(
    max: &Graphic3dFrameStatsData,
) -> impl Iterator<Item = (Timer, &'static str)> + '_ {
    [
        (Timer::CpuPicking, "Picking"),
        (Timer::CpuCulling, "Culling"),
        (Timer::CpuDynamics, "Dynamics"),
    ]
    .into_iter()
    .filter(|(timer, _)| match timer {
        Timer::CpuDynamics => max[*timer] != 0.0,
        _ => max[*timer] > 0.0,
    })
}

/// The "Timers Average" section, then "Timers Max" when requested.
fn push_timers(
    out: &mut String,
    stats: &Graphic3dFrameStatsData,
    max: &Graphic3dFrameStatsData,
    with_max: bool,
) {
    out.push_str("Timers Average\n");
    format_time(
        out,
        VALUE_WIDTH,
        " Elapsed Frame: ",
        stats[Timer::ElapsedFrame],
        "\n",
    );
    format_time(
        out,
        VALUE_WIDTH,
        "     CPU Frame: ",
        stats[Timer::CpuFrame],
        "\n",
    );
    for (timer, name) in optional_timers(max) {
        format_time(
            out,
            VALUE_WIDTH,
            &format!("{:>14}: ", format!("CPU {name}")),
            stats[timer],
            "\n",
        );
    }
    if !with_max {
        return;
    }
    out.push_str("Timers Max\n");
    format_time(
        out,
        VALUE_WIDTH,
        "     CPU Frame: ",
        max[Timer::CpuFrame],
        "\n",
    );
    for (timer, name) in optional_timers(max) {
        format_time(
            out,
            VALUE_WIDTH,
            &format!("{:>14}: ", format!("CPU {name}")),
            max[timer],
            "\n",
        );
    }
}

/// `FormatStats(dict, PerfCounters)`: the key/value pairs; `format_stats_dict(&fields, &mut dict, Perf::ALL)`.
pub(crate) fn format_stats_dict(
    fields: &Graphic3dFrameStatsFields,
    dict: &mut Graphic3dFrameStatsDict,
    flags: Perf,
) {
    let stats = fields.last_data_frame();
    if has(flags, Perf::FRAME_RATE) {
        add_info_real(dict, "FPS", stats.frame_rate());
    }
    if has(flags, Perf::CPU) {
        add_info_real(dict, "CPU FPS", stats.frame_rate_cpu());
    }
    if has(flags, Perf::LAYERS) {
        add_info_size(dict, "Layers", stats[Counter::NbLayers]);
        if fields.has_culled_layers() {
            add_info_size(dict, "Rendered layers", stats[Counter::NbLayersNotCulled]);
        }
    }
    if has(flags, Perf::STRUCTURES) {
        add_info_size(dict, "Structs", stats[Counter::NbStructs]);
        if fields.has_culled_structs() {
            add_info_size(dict, "Rendered structs", stats[Counter::NbStructsNotCulled]);
        }
    }
    add_rendered_info(dict, stats, flags);
    if has(flags, Perf::FRAME_TIME) {
        add_timer_info(
            dict,
            stats,
            &fields.counters_max,
            has(flags, Perf::FRAME_TIME_MAX),
        );
    }
}

/// The rendered counts and the GPU memory of the dictionary.
fn add_rendered_info(
    dict: &mut Graphic3dFrameStatsDict,
    stats: &Graphic3dFrameStatsData,
    flags: Perf,
) {
    let rows = [
        (Perf::GROUPS, "Rendered groups", Counter::NbGroupsNotCulled),
        (
            Perf::GROUP_ARRAYS,
            "Rendered arrays",
            Counter::NbElemsNotCulled,
        ),
        (
            Perf::GROUP_ARRAYS,
            "Rendered [fill] arrays",
            Counter::NbElemsFillNotCulled,
        ),
        (
            Perf::GROUP_ARRAYS,
            "Rendered [line] arrays",
            Counter::NbElemsLineNotCulled,
        ),
        (
            Perf::GROUP_ARRAYS,
            "Rendered [point] arrays",
            Counter::NbElemsPointNotCulled,
        ),
        (
            Perf::GROUP_ARRAYS,
            "Rendered [text] arrays",
            Counter::NbElemsTextNotCulled,
        ),
        (
            Perf::TRIANGLES,
            "Rendered triangles",
            Counter::NbTrianglesNotCulled,
        ),
        (Perf::LINES, "Rendered lines", Counter::NbLinesNotCulled),
        (Perf::POINTS, "Rendered points", Counter::NbPointsNotCulled),
        (
            Perf::ESTIM_MEM,
            "GPU Memory [geometry]",
            Counter::EstimatedBytesGeom,
        ),
        (
            Perf::ESTIM_MEM,
            "GPU Memory [textures]",
            Counter::EstimatedBytesTextures,
        ),
        (
            Perf::ESTIM_MEM,
            "GPU Memory [frames]",
            Counter::EstimatedBytesFbos,
        ),
    ];
    for (_, key, counter) in rows.into_iter().filter(|row| has(flags, row.0)) {
        add_info_size(dict, key, stats[counter]);
    }
}

/// The average timers and, when requested, the maximal timers of the dictionary.
fn add_timer_info(
    dict: &mut Graphic3dFrameStatsDict,
    stats: &Graphic3dFrameStatsData,
    max: &Graphic3dFrameStatsData,
    with_max: bool,
) {
    add_time_info(dict, "Elapsed Frame (average)", stats[Timer::ElapsedFrame]);
    add_time_info(dict, "CPU Frame (average)", stats[Timer::CpuFrame]);
    for (timer, name) in optional_timers(max) {
        add_time_info(dict, &format!("CPU {name} (average)"), stats[timer]);
    }
    if !with_max {
        return;
    }
    add_time_info(dict, "CPU Frame (max)", max[Timer::CpuFrame]);
    for (timer, name) in optional_timers(max) {
        add_time_info(dict, &format!("CPU {name} (max)"), max[timer]);
    }
}
