//! OSD process probe operations (OSD_Process, OSD_Host, OSD_Environment, OSD_SharedLibrary,
//! OSD_MemInfo); twin of `tkernel_osd_process_probe.cxx`.
use super::{bit, escape, integer, token};
use rustycad_tkernel::{
    osd::{
        CadPath, Environment, Host, LoadMode, MemCounter, MemInfo, OsdErrorAccess, Process,
        SharedLibrary,
    },
    standard::Failure,
    tcollection::AsciiString,
};
use std::str::SplitWhitespace;

/// The output of a process operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_env" => environment_sequence(input),
        "osd_env_name" => environment_names(input),
        "osd_host" => host_report(),
        "osd_process" => process_names(),
        "osd_chdir" => change_directory(input),
        "osd_shlib" => shared_library(input),
        "osd_meminfo" => mem_info(input),
        _ => return None,
    })
}

fn text(value: &AsciiString) -> String {
    escape(&String::from_utf8_lossy(value.as_bytes()))
}

fn failure_text(failure: &Failure) -> String {
    format!("{}{}", failure.exception_type(), escape(failure.message()))
}

/// The native `osd_process_error`: Failed, Perror outcome, Error, then Reset.
fn error_state(object: &mut impl OsdErrorAccess) -> String {
    let failed: u8 = bit(object.failed());
    let report: String = match object.perror() {
        Ok(()) => "ok".to_string(),
        Err(failure) => failure_text(&failure),
    };
    let out: String = format!(" {failed} {report} {}", object.error());
    object.reset();
    out
}

fn environment_sequence(input: &mut SplitWhitespace<'_>) -> String {
    let (name, value, value2) = (token(input), token(input), token(input));
    let mut env: Environment = match Environment::with_value(&name, &value) {
        Ok(env) => env,
        Err(failure) => return failure_text(&failure),
    };
    env.build();
    let mut out: String = format!("{} {}", text(&env.value()), bit(env.failed()));
    out += &match env.set_value(&value2) {
        Ok(()) => " set".to_string(),
        Err(failure) => format!(" {}", failure_text(&failure)),
    };
    env.build();
    out += &format!(" {}", text(&env.value()));
    env.remove();
    out += &format!(" {} {}", text(&env.value()), bit(env.failed()));
    match Environment::named(&name) {
        Ok(mut fresh) => out + &format!(" {}", text(&fresh.value())),
        Err(failure) => failure_text(&failure),
    }
}

fn environment_names(input: &mut SplitWhitespace<'_>) -> String {
    let (name, renamed) = (token(input), token(input));
    let mut env: Environment = match Environment::named(&name) {
        Ok(env) => env,
        Err(failure) => return format!(" {}", failure_text(&failure)),
    };
    let mut out: String = format!("{} {}", text(&env.name()), text(&env.value()));
    if let Err(failure) = env.set_name(&renamed) {
        return out + &format!(" {}", failure_text(&failure));
    }
    out += &format!(" {} {}", text(&env.name()), text(&env.value()));
    out
}

fn host_report() -> String {
    let mut host: Host = Host::new();
    let out: String = format!(
        "{} {} {} {} {} {}",
        text(&host.system_version()),
        host.system_id() as i32,
        text(&host.host_name()),
        host.machine_type() as i32,
        host.available_memory(),
        text(&host.internet_address())
    );
    out + &error_state(&mut host)
}

fn process_names() -> String {
    let mut process: Process = Process::new();
    let terminal: AsciiString = process.terminal_type();
    let date_ok: bool = process
        .system_date()
        .is_ok_and(|date| date.values().2 >= 2020);
    let out: String = format!(
        "{} {} {} {} {}",
        text(&terminal),
        text(&process.user_name()),
        bit(process.is_super_user()),
        bit(process.process_id() > 0),
        bit(date_ok)
    );
    out + &error_state(&mut process)
}

fn change_directory(input: &mut SplitWhitespace<'_>) -> String {
    let target: String = token(input);
    let mut process: Process = Process::new();
    let back: CadPath = process.current_directory();
    process.set_current_directory(&CadPath::new(target.as_str()));
    let here: AsciiString = process.current_directory().system_name();
    let trek: AsciiString = process.current_directory().trek();
    let mut out: String = format!("{} {}", text(&here), text(&trek));
    out += &error_state(&mut process);
    process.set_current_directory(&back);
    out + &format!(" {}", bit(process.failed()))
}

fn optional_text(value: Option<String>) -> String {
    value.map_or_else(|| "(none)".to_string(), |message| escape(&message))
}

fn shared_library(input: &mut SplitWhitespace<'_>) -> String {
    let name: String = token(input);
    let mode: i64 = integer(input);
    let symbol: String = token(input);
    let mut library: SharedLibrary = SharedLibrary::new();
    if !name.is_empty() {
        library.set_name(Some(&name));
    }
    let _stale: Option<String> = library.dl_error();
    let mut out: String = library.name().map_or("(null)".to_string(), escape);
    let mode: LoadMode = if mode == 0 {
        LoadMode::RtldLazy
    } else {
        LoadMode::RtldNow
    };
    let open: bool = library.dl_open(mode);
    out += &format!(" {} {}", bit(open), optional_text(library.dl_error()));
    if !open {
        return out;
    }
    let found: bool = library.dl_symb(&symbol).is_some();
    out += &format!(" {} {}", bit(found), optional_text(library.dl_error()));
    library.dl_close();
    library.destroy();
    out + &format!(" {}", bit(library.name().is_none()))
}

fn counter_shape(info: &MemInfo, counter: MemCounter) -> String {
    let shape = |unknown: bool| if unknown { 'u' } else { 'k' };
    format!(
        " {}{}{}{}",
        bit(info.is_active(counter)),
        shape(info.value(counter) == usize::MAX),
        shape(info.value_mib(counter) == usize::MAX),
        shape(info.value_precise_mib(counter) == -1.0)
    )
}

fn mem_info(input: &mut SplitWhitespace<'_>) -> String {
    let mask: i64 = integer(input);
    let update: bool = integer(input) != 0;
    let mut info: MemInfo = MemInfo::new(false);
    let mut out: String = text(&info.to_text());
    for (index, counter) in MemCounter::ALL.into_iter().enumerate() {
        info.set_counter_active(counter, (mask >> index) & 1 == 1);
    }
    if update {
        info.update();
    }
    for counter in MemCounter::ALL {
        out += &counter_shape(&info, counter);
    }
    if !update {
        out += &format!(" {}", text(&info.to_text()));
    }
    info.set_active(false);
    out += &format!(
        " {} {}",
        bit(info.is_active(MemCounter::HeapUsage)),
        text(&info.to_text())
    );
    info.clear();
    out
}
