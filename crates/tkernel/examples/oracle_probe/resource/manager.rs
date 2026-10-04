//! Resource_Manager probe operations over real files, with the environment and the console
//! injected (the native probe sets process variables and captures `std::cout`).
use super::{ext_string, failure, hex, token, unhex, units};
use rustycad_tkernel::{
    resource::{
        FormatType, NativeResourceIo, ResourceConsole, ResourceEnvironment, ResourceHost,
        ResourceManager, ResourceUnicode,
    },
    tcollection::{AsciiString, ExtendedString},
};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    str::SplitWhitespace,
    sync::{Arc, Mutex},
};

/// The variables the native probe exports with `setenv`.
#[derive(Default)]
struct ProbeEnvironment(Mutex<BTreeMap<String, PathBuf>>);
impl ProbeEnvironment {
    fn set(&self, name: &str, value: &str) {
        self.0
            .lock()
            .unwrap()
            .insert(name.into(), PathBuf::from(value));
    }
    fn clear(&self, name: &str) {
        self.0.lock().unwrap().remove(name);
    }
}
impl ResourceEnvironment for ProbeEnvironment {
    fn directory(&self, name: &str) -> Option<PathBuf> {
        self.0.lock().unwrap().get(name).cloned()
    }
}
/// The `std::cout` capture of the native probe.
#[derive(Default)]
struct ProbeConsole(Mutex<Vec<u8>>);
impl ResourceConsole for ProbeConsole {
    fn write(&self, bytes: &[u8]) {
        self.0.lock().unwrap().extend_from_slice(bytes);
    }
}
struct ProbeHost {
    environment: Arc<ProbeEnvironment>,
    console: Arc<ProbeConsole>,
}
impl ProbeHost {
    fn new() -> Self {
        Self {
            environment: Arc::default(),
            console: Arc::default(),
        }
    }
    fn host(&self) -> ResourceHost {
        ResourceHost::new(
            Arc::new(NativeResourceIo),
            self.environment.clone(),
            self.console.clone(),
        )
    }
    fn console(&self) -> String {
        hex(&self.console.0.lock().unwrap())
    }
}
fn write_file(directory: &str, name: &str, content: &str) {
    fs::create_dir_all(directory).expect("probe directory");
    fs::write(format!("{directory}/{name}"), unhex(content)).expect("probe file");
}
fn reset(directory: &str) {
    let _ = fs::remove_dir_all(directory);
}
/// Sorted entries of one map with their Integer/Real readings (native `res_entries`).
fn entries(manager: &mut ResourceManager, reference: bool) -> String {
    let mut sorted: Vec<(AsciiString, AsciiString)> = manager
        .get_map(reference)
        .items()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    sorted.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut text = String::new();
    for (key, value) in sorted {
        let name = String::from_utf8_lossy(key.as_bytes()).into_owned();
        let integer = manager
            .integer(&name)
            .map_or_else(|e| e.exception_type().to_string(), |v| v.to_string());
        let real = manager.real(&name).map_or_else(
            |e| e.exception_type().to_string(),
            |v| format!("{:016x}", v.to_bits()),
        );
        text += &format!(
            " {}={}:{integer}:{real}",
            hex(key.as_bytes()),
            hex(value.as_bytes())
        );
    }
    text
}
fn report(manager: &mut ResourceManager, probe: &ProbeHost) -> String {
    let console = probe.console();
    format!(
        "init={} console={console} ref{} user{}",
        u8::from(manager.is_initialized()),
        entries(manager, true),
        entries(manager, false)
    )
}
fn report_result(
    result: Result<ResourceManager, rustycad_tkernel::standard::Failure>,
    probe: &ProbeHost,
) -> String {
    match result {
        Ok(mut manager) => report(&mut manager, probe),
        Err(error) => format!("{}console={}", failure(&error), probe.console()),
    }
}
pub fn load(input: &mut SplitWhitespace<'_>) -> String {
    let verbose = token(input) == "1";
    let debug = token(input) == "1";
    let directory = token(input);
    reset(directory);
    write_file(directory, "Probe", token(input));
    let probe = ProbeHost::new();
    probe.environment.set("CSF_ProbeDefaults", directory);
    if debug {
        probe.environment.set("ResourceDebug", "1");
    }
    report_result(ResourceManager::new("Probe", verbose, probe.host()), &probe)
}
pub fn directories(input: &mut SplitWhitespace<'_>) -> String {
    let verbose = token(input) == "1";
    let directory = token(input);
    let defaults = String::from_utf8(unhex(token(input))).expect("utf-8 directory");
    let user = String::from_utf8(unhex(token(input))).expect("utf-8 directory");
    let (defaults_file, user_file) = (token(input), token(input));
    reset(directory);
    write_file(&format!("{directory}/defaults"), "Probe", defaults_file);
    write_file(&format!("{directory}/user"), "Probe", user_file);
    let probe = ProbeHost::new();
    let result =
        ResourceManager::with_directories("Probe", &defaults, &user, verbose, probe.host());
    report_result(result, &probe)
}
fn assign(manager: &mut ResourceManager, assignment: &str) {
    let (kind, rest) = assignment.split_at(2);
    let (name, value) = rest.split_once(':').expect("name:value");
    let name = String::from_utf8(unhex(name)).expect("utf-8 name");
    match kind {
        "i:" => manager.set_resource_integer(&name, value.parse().expect("integer")),
        "r:" => {
            let bits = u64::from_str_radix(value, 16).expect("real bits");
            manager.set_resource_real(&name, f64::from_bits(bits));
        }
        _ => manager.set_resource(&name, &unhex(value)),
    }
}
pub fn save(input: &mut SplitWhitespace<'_>) -> String {
    let verbose = token(input) == "1";
    let directory = token(input);
    reset(directory);
    let probe = ProbeHost::new();
    let mut manager = ResourceManager::new("Probe", verbose, probe.host()).expect("no files");
    for assignment in input.by_ref() {
        assign(&mut manager, assignment);
    }
    let user = format!("{directory}/nested/user");
    probe.environment.set("CSF_ProbeUserDefaults", &user);
    let saved = manager.save();
    let content = fs::read(format!("{user}/Probe")).unwrap_or_default();
    probe.environment.clear("CSF_ProbeUserDefaults");
    format!(
        "saved={} file={} {}",
        u8::from(saved),
        hex(&content),
        report(&mut manager, &probe)
    )
}
pub fn path(input: &mut SplitWhitespace<'_>) -> String {
    let user = token(input) == "1";
    let directory = String::from_utf8(unhex(token(input))).expect("utf-8 directory");
    let probe = ProbeHost::new();
    if !directory.is_empty() {
        let variable = if user {
            "CSF_ProbeUserDefaults"
        } else {
            "CSF_ProbeDefaults"
        };
        probe.environment.set(variable, &directory);
    }
    let path = ResourceManager::resource_path_with("Probe", user, probe.environment.as_ref());
    hex(path.as_bytes())
}
/// The `const char16_t*` view of a cached value: units up to the first zero.
fn c_units(text: &ExtendedString) -> String {
    units(&ExtendedString::from_utf16(text.as_utf16()))
}
pub fn ext(input: &mut SplitWhitespace<'_>) -> String {
    let format = FormatType::from_raw(token(input).parse().expect("format")).expect("format");
    ResourceUnicode::set_format(format);
    let bytes = unhex(token(input));
    let value = ext_string(token(input));
    let mut manager = ResourceManager::empty(ProbeHost::new().host());
    manager.set_resource("k", &bytes);
    let mut text = match manager.ext_value("k") {
        Ok(cached) => c_units(cached) + " ",
        Err(error) => failure(&error),
    };
    text += &match manager.set_resource_ext("e", value.as_utf16()) {
        Ok(()) => format!("e={} ", hex(manager.value("e").expect("stored").as_bytes())),
        Err(error) => failure(&error),
    };
    let cached = manager.ext_value("e").expect("cached before conversion");
    format!("{text}cached {}", c_units(cached))
}
pub fn charset(input: &mut SplitWhitespace<'_>) -> String {
    let directory = token(input);
    reset(directory);
    write_file(directory, "CharSet", token(input));
    let probe = ProbeHost::new();
    probe.environment.set("CSF_CharSetDefaults", directory);
    ResourceUnicode::read_format_with(&probe.host()).expect("CharSet readable");
    let format = ResourceUnicode::get_format().expect("format already read");
    format!("format={}", format as i32)
}
