//! OSD timing probe operations (twin of `tkernel_osd_timing_probe.cxx`).
use super::{bit, escape, integer, token};
use rustycad_tkernel::{
    osd::{Chronometer, Osd, SignalMode, Timer},
    standard::Failure,
};
use std::str::SplitWhitespace;

/// The output of a timing operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_timer_reset" => timer_reset(real(input)),
        "osd_chrono" => chronometer(integer(input) != 0),
        "osd_real_to_cstring" => format!("1 {}", escape(&Osd::real_to_c_string(real(input)))),
        "osd_cstring_to_real" => cstring_to_real(&token(input)),
        "osd_signal_stack" => signal_stack(integer(input) as i32),
        "osd_signal" | "osd_thread_signal" | "osd_floating" => signal(op, input),
        "osd_ctrl_break" => control_break(),
        "osd_sleep" => {
            Osd::sec_sleep(integer(input) as i32);
            Osd::milli_sec_sleep(integer(input) as i32);
            "slept".to_string()
        }
        _ => return None,
    })
}

fn real(input: &mut SplitWhitespace<'_>) -> f64 {
    let hex: &str = input.next().expect("hex bits");
    f64::from_bits(u64::from_str_radix(hex, 16).expect("hex bits"))
}

fn bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn timer_reset(elapsed: f64) -> String {
    let mut timer: Timer = Timer::new(false);
    timer.reset_to(elapsed);
    let (seconds, minutes, hours, cpu) = timer.show_values();
    format!(
        "{} {} {} {minutes} {hours} {} {}",
        bits(timer.elapsed_time()),
        bit(timer.is_started()),
        bits(seconds),
        bits(cpu),
        escape(&timer.show_to())
    )
}

fn failure_text(result: Result<(), Failure>) -> String {
    match result {
        Ok(()) => "ok".to_string(),
        Err(failure) => format!("{}{}", failure.exception_type(), escape(failure.message())),
    }
}

fn chronometer(thread_only: bool) -> String {
    let mut chrono: Chronometer = Chronometer::new(thread_only);
    let (user, sys) = chrono.show_cpu();
    let mut out: String = format!(
        "{} {} {} {} {} {} {} {}",
        bit(chrono.is_started()),
        bit(chrono.is_this_thread_only()),
        escape(&chrono.show_to()),
        bits(user),
        bits(sys),
        bits(chrono.show_user()),
        bits(chrono.user_time_cpu()),
        bits(chrono.system_time_cpu())
    );
    chrono
        .set_this_thread_only(!thread_only)
        .expect("stopped chronometer");
    out += &format!(" {}", bit(chrono.is_this_thread_only()));
    chrono.start();
    out += &format!(" {}", bit(chrono.is_started()));
    out += &format!(
        " {}",
        failure_text(chrono.set_this_thread_only(thread_only))
    );
    chrono.stop();
    out += &format!(
        " {} {}",
        bit(chrono.is_started()),
        bit(chrono.is_this_thread_only())
    );
    chrono.reset();
    out + &format!(" {}", escape(&chrono.show_to()))
}

fn cstring_to_real(hex: &str) -> String {
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[2 * index..2 * index + 2], 16).expect("hex byte"))
        .collect();
    let text: String = String::from_utf8(bytes).expect("ASCII corpus text");
    let (full, value) = Osd::c_string_to_real(&text);
    format!("{} {}", bit(full), bits(value))
}

fn signal_stack(length: i32) -> String {
    let before: i32 = Osd::signal_stack_trace_length();
    Osd::set_signal_stack_trace_length(length);
    let set: i32 = Osd::signal_stack_trace_length();
    Osd::set_signal_stack_trace_length(0);
    format!("{before} {set} {}", Osd::signal_stack_trace_length())
}

fn signal(op: &str, input: &mut SplitWhitespace<'_>) -> String {
    let mode: i64 = if op == "osd_floating" {
        0
    } else {
        integer(input)
    };
    let floating: bool = integer(input) != 0;
    let mode: SignalMode = SignalMode::from_raw(mode as i32).expect("signal mode");
    match op {
        "osd_signal" => Osd::set_signal(mode, floating),
        "osd_thread_signal" => Osd::set_thread_local_signal(mode, floating),
        _ => Osd::set_floating_signal(floating),
    }
    let catching: bool = Osd::to_catch_floating_signals();
    Osd::set_floating_signal(false);
    format!(
        " {} {} {}",
        bit(Osd::to_catch_floating_signals()),
        Osd::signal_mode() as i32,
        bit(catching)
    )
}

fn control_break() -> String {
    let mut calls: Vec<String> = Vec::new();
    for call in 0..3 {
        if call == 1 {
            Osd::set_signal(SignalMode::Set, false);
            raise_interrupt();
        }
        calls.push(failure_text(Osd::control_break()));
    }
    Osd::set_signal(SignalMode::Unset, false);
    let unset: i32 = Osd::signal_mode() as i32;
    Osd::set_signal(SignalMode::AsIs, false);
    format!("{} {unset} {}", calls.join(" "), Osd::signal_mode() as i32)
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn raise_interrupt() {
    // SAFETY: raise has no memory preconditions; the installed SIGINT handler only stores
    // an atomic flag.
    unsafe { libc::raise(libc::SIGINT) };
}

// The oracle profile is Linux-only; other hosts only need the probe to compile.
#[cfg(not(unix))]
fn raise_interrupt() {
    panic!("SIGINT probe requests run on the Linux oracle profile only");
}
