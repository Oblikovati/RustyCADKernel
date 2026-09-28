//! Message text operations (twin of `tools/oracle/tkernel_message_text_probe.cxx`).
#![allow(unsafe_code)]
use super::{ascii, bit, bytes, ext, real, rest, units};
use rustycad_tkernel::message::{ExecStatus, Message, MetricType, Msg, MsgFile, Status};
use rustycad_tkernel::osd::MemInfoCounter;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;

/// The output of a text operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "msg_run" => msg_run(input),
        "msg_set_c" => {
            let mut message = Msg::new();
            message.set_text(&bytes(input.next().expect("text")));
            format!("{} {}", units(message.value()), units(message.original()))
        }
        "msg_fill_time" => {
            let hour: i32 = input.next().expect("hour").parse().expect("int");
            let minute: i32 = input.next().expect("minute").parse().expect("int");
            ascii(&Message::fill_time(
                hour,
                minute,
                real(input.next().expect("second")),
            ))
        }
        "msg_metrics" => metrics(),
        "msg_metric_from" => metric_from(input),
        "msg_exec_ops" => exec_ops(input),
        "msg_exec_static" => exec_static(input),
        "msg_exec_by_index" => exec_by_index(input),
        "msgfile_string" => msgfile_string(input),
        "msgfile_file" => msgfile_file(input),
        "msgfile_load" => msgfile_load(input),
        "msgfile_env" => msgfile_env(input),
        _ => return None,
    })
}

fn state(message: &Msg) -> String {
    format!(" {}/{}", units(message.value()), bit(message.is_edited()))
}

fn msg_step(message: &mut Msg, step: &str) -> Result<String, Failure> {
    let value = step.get(2..).unwrap_or_default();
    match step.as_bytes()[0] {
        b's' | b'a' => message.arg_text(&bytes(value)).map(|_| String::new()),
        b'e' => message.arg_extended(&ext(value)).map(|_| String::new()),
        b'i' => message
            .arg_integer(value.parse().expect("int"))
            .map(|_| String::new()),
        b'r' => message.arg_real(real(value)).map(|_| String::new()),
        _ => message.get().map(|text| format!(" {}", units(text))),
    }
}

fn msg_run(input: &mut SplitWhitespace<'_>) -> String {
    let mut message = Msg::new();
    message.set(&ext(input.next().expect("text")));
    let mut out = units(message.original());
    for step in rest(input) {
        match msg_step(&mut message, &step) {
            Ok(text) => out.push_str(&text),
            Err(failure) => {
                out.push_str(&format!(
                    " E{}{}",
                    failure.exception_type(),
                    state(&message)
                ));
                return out;
            }
        }
        out.push_str(&state(&message));
    }
    out
}

fn metrics() -> String {
    let mut out = String::new();
    for metric in MetricType::ALL {
        let counter = Message::to_osd_metric(metric);
        out.push_str(&format!(
            " {}:{}:{}",
            Message::metric_to_string(metric),
            bit(counter.is_some()),
            counter.map_or(-1, |counter| counter as i32)
        ));
    }
    let counters = [
        MemInfoCounter::Private,
        MemInfoCounter::Virtual,
        MemInfoCounter::WorkingSet,
        MemInfoCounter::WorkingSetPeak,
        MemInfoCounter::SwapUsage,
        MemInfoCounter::SwapUsagePeak,
        MemInfoCounter::HeapUsage,
    ];
    for counter in counters {
        let metric = Message::to_message_metric(counter);
        out.push_str(&format!(
            " {}:{}",
            bit(metric.is_some()),
            metric.unwrap_or(MetricType::None) as i32
        ));
    }
    out
}

fn metric_from(input: &mut SplitWhitespace<'_>) -> String {
    let name = bytes(input.next().expect("name"));
    // `const char*`: the name ends at its first NUL; the corpus sends UTF-8 free names.
    let name = String::from_utf8_lossy(name.split(|&byte| byte == 0).next().unwrap_or_default())
        .into_owned();
    let found = Message::metric_from_string(&name);
    format!(
        "{} {} {}",
        bit(found.is_some()),
        found.unwrap_or(MetricType::None) as i32,
        Message::metric_from_string_or_none(&name) as i32
    )
}

fn exec_step(status: &mut ExecStatus, step: &str) {
    let value = Status(
        step.get(1..)
            .filter(|text| !text.is_empty())
            .map_or(0, |text| text.parse().expect("int")),
    );
    match step.as_bytes()[0] {
        b's' => status.set(value),
        b'c' => status.clear_status(value),
        b'D' => status.set_all_done(),
        b'W' => status.set_all_warn(),
        b'A' => status.set_all_alarm(),
        b'F' => status.set_all_fail(),
        b'd' => status.clear_all_done(),
        b'w' => status.clear_all_warn(),
        b'a' => status.clear_all_alarm(),
        b'f' => status.clear_all_fail(),
        b'x' => status.clear(),
        b'|' => *status |= &ExecStatus::from_status(value),
        b'&' => *status &= &ExecStatus::from_status(value),
        b'+' => status.add(&ExecStatus::from_status(value)),
        _ => status.and(&ExecStatus::from_status(value)),
    }
}

fn exec_ops(input: &mut SplitWhitespace<'_>) -> String {
    let mut status = ExecStatus::new();
    for step in rest(input) {
        exec_step(&mut status, &step);
    }
    let mut out = format!(
        "{}{}{}{} ",
        bit(status.is_done()),
        bit(status.is_warn()),
        bit(status.is_alarm()),
        bit(status.is_fail())
    );
    for index in ExecStatus::FIRST_STATUS..ExecStatus::LAST_STATUS {
        out.push_str(&bit(status.is_set(ExecStatus::status_by_index(index))).to_string());
    }
    out
}

fn exec_static(input: &mut SplitWhitespace<'_>) -> String {
    let mut out = String::new();
    for token in rest(input) {
        let status = Status(token.parse().expect("int"));
        out.push_str(&format!(
            " {}:{}:{}",
            ExecStatus::status_index(status),
            ExecStatus::local_status_index(status),
            ExecStatus::type_of_status(status).0
        ));
    }
    out
}

fn exec_by_index(input: &mut SplitWhitespace<'_>) -> String {
    let low: i32 = input.next().expect("low").parse().expect("int");
    let high: i32 = input.next().expect("high").parse().expect("int");
    (low..=high)
        .map(|index| format!(" {}", ExecStatus::status_by_index(index).0))
        .collect()
}

fn keys(input: &mut SplitWhitespace<'_>) -> String {
    let catalog = MsgFile::global();
    let mut out = String::new();
    for key in rest(input) {
        let name = AsciiString::from_bytes(bytes(&key));
        let known = catalog.has_msg(&name);
        out.push_str(&format!(" {}:{}", bit(known), units(&catalog.msg(&name))));
    }
    out
}

fn loaded(result: Result<bool, Failure>) -> String {
    match result {
        Ok(value) => bit(value).to_string(),
        Err(failure) => format!("E{}", failure.exception_type()),
    }
}

fn text_arg(token: &str) -> String {
    String::from_utf8(bytes(token)).expect("UTF-8 path or name in the corpus")
}

fn msgfile_string(input: &mut SplitWhitespace<'_>) -> String {
    let content = bytes(input.next().expect("content"));
    let length: i32 = input.next().expect("length").parse().expect("int");
    let out = loaded(MsgFile::global().load_from_string(&content, length));
    out + &keys(input)
}

fn msgfile_file(input: &mut SplitWhitespace<'_>) -> String {
    let path = text_arg(input.next().expect("path"));
    let content = bytes(input.next().expect("content"));
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("probe directory");
    }
    std::fs::write(&path, content).expect("probe file");
    let out = loaded(MsgFile::global().load_file(Some(&path)));
    out + &keys(input)
}

fn msgfile_load(input: &mut SplitWhitespace<'_>) -> String {
    let directories = text_arg(input.next().expect("directories"));
    let file = text_arg(input.next().expect("file"));
    let out = loaded(MsgFile::global().load(Some(&directories), Some(&file)));
    out + &keys(input)
}

fn msgfile_env(input: &mut SplitWhitespace<'_>) -> String {
    let variable = input.next().expect("variable").to_owned();
    let value = input.next().expect("value");
    // SAFETY: the probe is single-threaded; the native twin calls setenv/unsetenv.
    unsafe {
        if value == "-" {
            std::env::remove_var(&variable);
        } else {
            std::env::set_var(&variable, text_arg(value));
        }
    }
    let lang = input.next().expect("lang");
    let lang = (lang != "N").then(|| {
        if lang == "-" {
            String::new()
        } else {
            text_arg(lang)
        }
    });
    let file = text_arg(input.next().expect("file"));
    let out = loaded(MsgFile::global().load_from_env(Some(&variable), &file, lang.as_deref()));
    out + &keys(input)
}
