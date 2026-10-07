//! The session commands of `Draw_BasicCommands.cxx`: `batch`, `spy`, `wait`, `dlog`,
//! `decho`, `dbreak`, `dtracelevel`, `ddebugtraces`, `dputs`.
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::message::{ConsoleColor, Gravity, Message, PrinterOStream};
use rustycad_tkernel::osd::Osd;
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;

/// `batch`: 1 in batch mode.
pub(super) fn ifbatch(
    session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    di.append(if session.is_batch() { "1" } else { "0" })?;
    Ok(0)
}

/// `spy [file]`: records every console command in `file`; without a file stops.
pub(super) fn spy(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    *session.spy_file.borrow_mut() = None;
    let Some(file_name) = a.get(1) else {
        return Ok(0);
    };
    match std::fs::File::create(file_name) {
        Ok(file) => *session.spy_file.borrow_mut() = Some(file),
        Err(_) => {
            di.append(format!("Cannot open {file_name} for writing\n"))?;
            return Ok(1);
        }
    }
    Ok(0)
}

/// Seconds since the epoch (`time(nullptr)`).
fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// `wait [seconds=10]`: returns once the wall clock passed `now + seconds` (the C++ loop
/// spins on `time()`; this one sleeps between checks).
pub(super) fn wait(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let seconds = a.get(1).map_or(10, |text| session.atoi(text));
    let deadline = now_seconds() + i64::from(seconds);
    while now_seconds() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    Ok(0)
}

/// `dlog on|off|reset|get|status|add text`: the command log.
pub(super) fn dlog(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let (option, extra) = (a.get(1).copied(), a.get(2).copied());
    match (a.len(), option, extra) {
        (2, Some("on"), _) => di.set_do_log(true),
        (2, Some("off"), _) => di.set_do_log(false),
        (2, Some("reset"), _) => di.reset_log(),
        (2, Some("get"), _) => drop(di.append(di.get_log())?),
        (3, Some("add"), Some(text)) => {
            di.add_log(text);
            di.add_log("\n");
        }
        (2, Some("status"), _) => drop(di.append(if di.get_do_log() { "on" } else { "off" })?),
        (2 | 3, _, _) => {
            let _ = write!(Message::send_fail(), "Unrecognized option(s): {}", a[1]);
            return Ok(1);
        }
        _ => {
            let name = a[0];
            let _ = write!(
                Message::send_fail(),
                "Enable or disable logging: {name} {{on|off}}\nReset log: {name} reset\nGet log content: {name} get"
            );
            return Ok(1);
        }
    }
    Ok(0)
}

/// `decho on|off`: echo of commands and results.
pub(super) fn decho(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 2 {
        let _ = write!(
            Message::send_fail(),
            "Enable or disable echoing: {} {{on|off}}",
            a[0]
        );
        return Ok(1);
    }
    match a[1] {
        "on" => di.set_do_echo(true),
        "off" => di.set_do_echo(false),
        option => {
            let _ = write!(Message::send_fail(), "Unrecognized option: {option}");
            return Ok(1);
        }
    }
    Ok(0)
}

/// `dbreak`: a Tcl error when the user pressed Control-Break.
pub(super) fn dbreak(
    _session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    match Osd::control_break() {
        Ok(()) => Ok(0),
        Err(failure) if failure.exception_type() == "OSD_Exception_CTRL_BREAK" => {
            di.append("User pressed Control-Break")?;
            Ok(1)
        }
        Err(failure) => Err(failure),
    }
}

/// The trace level named by `text` (`trace`, `info`, `warn`/`warning`, `alarm`, `fail`).
fn gravity_named(text: &str) -> Option<Gravity> {
    match text.to_ascii_lowercase().as_str() {
        "trace" => Some(Gravity::Trace),
        "info" => Some(Gravity::Info),
        "warn" | "warning" => Some(Gravity::Warning),
        "alarm" => Some(Gravity::Alarm),
        "fail" => Some(Gravity::Fail),
        _ => None,
    }
}

/// The name `dtracelevel` prints for a level.
fn gravity_name(gravity: Gravity) -> &'static str {
    match gravity {
        Gravity::Trace => "trace",
        Gravity::Info => "info",
        Gravity::Warning => "warn",
        Gravity::Alarm => "alarm",
        Gravity::Fail => "fail",
    }
}

/// `dtracelevel [level]`: sets the trace level of every printer of the default
/// messenger, or prints the level of the first printer and of the others that differ.
pub(super) fn dtracelevel(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() > 2 {
        let _ = write!(
            Message::send_fail(),
            "Error: wrong number of arguments! See usage:"
        );
        di.print_help(a[0])?;
        return Ok(1);
    }
    let level = match a.get(1) {
        None => None,
        Some(text) => match gravity_named(text) {
            Some(level) => Some(level),
            None => {
                let _ = write!(Message::send_fail(), "Error: unknown gravity '{text}'");
                return Ok(1);
            }
        },
    };
    let printers = Message::default_messenger().printers().clone();
    if printers.is_empty() {
        let _ = write!(
            Message::send_fail(),
            "Error: no printers registered in default Messenger"
        );
        return Ok(0);
    }
    let Some(level) = level else {
        let first = printers[0].trace_level();
        for (index, printer) in printers.iter().enumerate() {
            if index > 0 && printer.trace_level() == first {
                continue;
            }
            di.append(gravity_name(printer.trace_level()))?;
        }
        return Ok(0);
    };
    printers
        .iter()
        .for_each(|printer| printer.set_trace_level(level));
    Ok(0)
}

/// `ddebugtraces [count]`: the stack trace length of new exceptions.
pub(super) fn ddebugtraces(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    match a.len() {
        0 | 1 => {
            di.append(Failure::default_stack_trace_length())?;
            Ok(0)
        }
        2 => {
            Failure::set_default_stack_trace_length(session.atoi(a[1]));
            Ok(0)
        }
        _ => {
            di.append("Syntax error: wrong number of arguments")?;
            Ok(1)
        }
    }
}

/// The console color named by a `dputs` option.
fn console_color(option: &str) -> Option<ConsoleColor> {
    match option {
        "-black" => Some(ConsoleColor::Black),
        "-white" => Some(ConsoleColor::White),
        "-red" => Some(ConsoleColor::Red),
        "-blue" => Some(ConsoleColor::Blue),
        "-green" => Some(ConsoleColor::Green),
        "-yellow" => Some(ConsoleColor::Yellow),
        "-cyan" => Some(ConsoleColor::Cyan),
        "-magenta" => Some(ConsoleColor::Magenta),
        _ => None,
    }
}

/// How `dputs` prints its text.
#[derive(Default)]
struct PutsStyle {
    to_error: bool,
    no_newline: bool,
    intense: bool,
    color: Option<ConsoleColor>,
}

/// `dputs [options] text`: prints the last argument to the console, optionally colored.
pub(super) fn dputs(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let mut style = PutsStyle::default();
    for (index, argument) in a.iter().enumerate().skip(1) {
        let option = argument.to_ascii_lowercase();
        match option.as_str() {
            "-nonewline" => style.no_newline = true,
            "stdcout" => style.to_error = false,
            "stdcerr" => style.to_error = true,
            "-intense" => style.intense = true,
            _ if console_color(&option).is_some() => style.color = console_color(&option),
            _ if index + 1 == a.len() => {
                put_text(session, di, &style, argument);
                return Ok(0);
            }
            _ => {
                let _ = write!(Message::send_fail(), "Syntax error at '{option}'");
                return Ok(1);
            }
        }
    }
    let _ = write!(
        Message::send_fail(),
        "Syntax error: wrong number of arguments"
    );
    Ok(1)
}

/// Writes the `dputs` text with its colors and newline.
fn put_text(session: &DrawSession, di: &DrawInterpretor, style: &PutsStyle, text: &str) {
    let colored = di.to_colorize() && (style.intense || style.color.is_some());
    let write = |text: &str| {
        if style.to_error {
            session.console().write_err(text)
        } else {
            session.console().write_out(text)
        }
    };
    if colored {
        write(&color_sequence(
            style.color.unwrap_or(ConsoleColor::Default),
            style.intense,
        ));
    }
    write(text);
    if colored {
        write(&color_sequence(ConsoleColor::Default, false));
    }
    if !style.no_newline {
        write("\n");
    }
}

/// The escape sequence of `Message_PrinterOStream::SetConsoleTextColor`.
pub(super) fn color_sequence(color: ConsoleColor, intense: bool) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    PrinterOStream::set_console_text_color(Some(&mut bytes), color, intense);
    String::from_utf8_lossy(&bytes).into_owned()
}
