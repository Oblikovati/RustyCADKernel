//! `chrono` / `dchrono` (`dchronom` in `Draw_BasicCommands.cxx`): named timers.
use crate::draw::{
    DrawChronometer, DrawInterpretor, DrawSession, downcast_handle, parse_on_off_iterator,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::format_general;
use std::rc::Rc;

/// `std::cout << double` after `Draw_Appli` set the precision to 15.
pub(super) fn cout_real(value: f64) -> String {
    format_general(value, 15)
}

/// `chrono [0|1]` toggles or sets the global timing of commands; `chrono name options`
/// operates the timer variable `name` (created when missing).
pub(super) fn dchronom(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let flag = a
        .get(1)
        .and_then(|text| text.bytes().next())
        .filter(|first| *first == b'0' || *first == b'1');
    if a.len() == 1 || (a.len() == 2 && flag.is_some()) {
        let enabled = flag.map_or(!session.chrono(), |first| first == b'1');
        session.set_chrono(enabled);
        di.append(if enabled {
            "Chronometers activated.\n"
        } else {
            "Chronometers deactivated.\n"
        })?;
        return Ok(0);
    }
    let chronometer = named_chronometer(session, a[1]);
    if a.len() <= 2 {
        chronometer.timer().reset();
        return Ok(0);
    }
    let to_cout = a[0] == "chrono";
    let mut puts = 0;
    let mut index = 2;
    while index < a.len() {
        if !apply_option(session, di, &chronometer, a, &mut index, to_cout, &mut puts)? {
            di.append(format!("Syntax error at '{}'\n", a[index]))?;
            return Ok(1);
        }
        index += 1;
    }
    Ok(0)
}

/// The chronometer stored in `name`, or a new one stored there (not displayed).
fn named_chronometer(session: &DrawSession, name: &str) -> Rc<DrawChronometer> {
    if let Some(existing) = session
        .get(name)
        .and_then(|d| downcast_handle::<DrawChronometer>(&d))
    {
        return existing;
    }
    let chronometer = Rc::new(DrawChronometer::new());
    session.set_drawable(name, Some(chronometer.clone()), false);
    chronometer
}

/// Applies the option at `a[*index]`; false for an unknown option.
fn apply_option(
    session: &DrawSession,
    di: &DrawInterpretor,
    chronometer: &DrawChronometer,
    a: &[&str],
    index: &mut usize,
    to_cout: bool,
    puts: &mut i32,
) -> Result<bool, Failure> {
    let option = a[*index].to_ascii_lowercase();
    let mut timer = chronometer.timer();
    match option.as_str() {
        "-reset" | "reset" => timer.reset(),
        "-restart" | "restart" => timer.restart(),
        "-start" | "-resume" | "start" => timer.start(),
        "-stop" | "-pause" | "stop" => timer.stop(),
        "-show" | "show" if to_cout => session.console().write_out(&timer.show_to()),
        "-show" | "show" => {
            di.append(timer.show_to())?;
        }
        "-counter" | "counter" if *index + 1 < a.len() => {
            *index += 1;
            let cpu = timer.show_values().3;
            if to_cout {
                session.console().write_out(&format!(
                    "COUNTER {}: {}\n",
                    a[*index],
                    cout_real(cpu)
                ));
            } else {
                di.append(format!("COUNTER {}: ", a[*index]))?
                    .append(cpu)?
                    .append("\n")?;
            }
        }
        "-elapsed" => put_value(di, puts, timer.elapsed_time())?,
        "-cpu" | "-usercpu" | "-cpuuser" => put_value(di, puts, timer.user_time_cpu())?,
        "-systemcpu" | "-syscpu" | "-cpusystem" | "-cpusys" => {
            put_value(di, puts, timer.system_time_cpu())?
        }
        "-thread" | "-threadonly" | "-process" => {
            let flag = parse_on_off_iterator(a, index);
            timer.stop();
            timer.reset();
            timer.set_this_thread_only(if option == "-process" { !flag } else { flag })?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Appends one measured value, separated by a space from the previous one.
fn put_value(di: &DrawInterpretor, puts: &mut i32, value: f64) -> Result<(), Failure> {
    *puts += 1;
    if *puts > 1 {
        di.append(" ")?;
    }
    di.append(value)?;
    Ok(())
}
