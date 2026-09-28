//! OSD foundation probe operations (twin of `tkernel_osd_foundation_probe.cxx`).
use super::{bit, escape, integer, token};
use rustycad_tkernel::{
    osd::{self, OsdError, Protection, SingleProtection},
    standard::Failure,
};
use std::str::SplitWhitespace;

/// The output of a foundation operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_prot_row" => protection_row(single(input)),
        "osd_prot_default" => protection_text(&Protection::new()),
        "osd_prot_set" => protection_set(input),
        "osd_error" => error_report(input),
        "osd_exception" => exception_report(input),
        "osd_enums" => enum_counts(),
        _ => return None,
    })
}

fn single(input: &mut SplitWhitespace<'_>) -> SingleProtection {
    let raw: i64 = integer(input);
    SingleProtection::from_raw(raw as i32).expect("protection in 0..16")
}

fn raw(value: SingleProtection) -> i32 {
    value as i32
}

fn protection_text(protection: &Protection) -> String {
    let (s, u, g, w) = protection.values();
    format!("{} {} {} {}", raw(s), raw(u), raw(g), raw(w))
}

fn protection_row(left: SingleProtection) -> String {
    let protection: Protection = Protection::new();
    let cells: Vec<String> = SingleProtection::ALL
        .iter()
        .map(|&right| {
            let (mut added, mut removed) = (left, left);
            protection.add(&mut added, right);
            protection.sub(&mut removed, right);
            format!("{}/{}", raw(added), raw(removed))
        })
        .collect();
    cells.join(" ")
}

fn protection_set(input: &mut SplitWhitespace<'_>) -> String {
    let (s, u, g, w) = (single(input), single(input), single(input), single(input));
    let mut protection: Protection = Protection::with_values(s, u, g, w);
    let initial: String = protection_text(&protection);
    protection.set_system(w);
    protection.set_user(s);
    protection.set_group(u);
    protection.set_world(g);
    let rotated: String = format!(
        "{} {} {} {}",
        raw(protection.system()),
        raw(protection.user()),
        raw(protection.group()),
        raw(protection.world())
    );
    protection.set_values(g, w, s, u);
    format!("{initial} {rotated} {}", protection_text(&protection))
}

fn error_report(input: &mut SplitWhitespace<'_>) -> String {
    let errno: i32 = integer(input) as i32;
    let from: i32 = integer(input) as i32;
    let message: String = token(input);
    let mut error: OsdError = OsdError::new();
    let mut out: String = format!("{} {}", bit(error.failed()), error.error());
    error.set_value(errno, from, message.as_str());
    out += &format!(" {} {}", bit(error.failed()), error.error());
    out += &match error.perror() {
        Ok(()) => " ok".to_string(),
        Err(failure) => format!(" {}{}", failure.exception_type(), escape(failure.message())),
    };
    out += &format!(" {}", error.error());
    error.reset();
    out + &format!(" {} {}", bit(error.failed()), error.error())
}

fn exception_report(input: &mut SplitWhitespace<'_>) -> String {
    let index: i64 = integer(input);
    let message: String = token(input);
    let raise: fn(&str) -> Failure = match index {
        0 => osd::exception,
        1 => osd::exception_access_violation,
        2 => osd::exception_array_bounds_exceeded,
        3 => osd::exception_ctrl_break,
        4 => osd::exception_illegal_instruction,
        5 => osd::exception_in_page_error,
        6 => osd::exception_int_overflow,
        7 => osd::exception_invalid_disposition,
        8 => osd::exception_noncontinuable_exception,
        9 => osd::exception_priv_instruction,
        10 => osd::exception_stack_overflow,
        11 => osd::exception_status_no_memory,
        12 => osd::signal,
        13 => osd::sigbus,
        14 => osd::sighup,
        15 => osd::sigill,
        16 => osd::sigint,
        17 => osd::sigkill,
        18 => osd::sigquit,
        19 => osd::sigsegv,
        20 => osd::sigsys,
        _ => osd::osd_osd_error,
    };
    let failure: Failure = raise(&message);
    format!("{}{}", failure.exception_type(), escape(failure.message()))
}

fn enum_counts() -> String {
    [
        osd::SysType::ALL.len(),
        SingleProtection::ALL.len(),
        osd::FromWhere::ALL.len(),
        osd::KindFile::ALL.len(),
        osd::LockType::ALL.len(),
        osd::OpenMode::ALL.len(),
        osd::OemType::ALL.len(),
        osd::LoadMode::ALL.len(),
        osd::WhoAmI::ALL.len(),
        osd::SignalMode::ALL.len(),
    ]
    .map(|count| count.to_string())
    .join(" ")
}
