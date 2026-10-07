//! `Draw_PloadCommands.cxx`: `pload` and `dtryload`.
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::{Environment, LoadMode, OsdErrorAccess, SharedLibrary};
use rustycad_tkernel::resource::{ResourceHost, ResourceManager};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Write;
use std::path::Path;

/// The value of an environment variable (empty when unset or invalid).
fn environment(name: &str) -> String {
    Environment::named(name)
        .map(|mut v| String::from_utf8_lossy(v.value().as_bytes()).into_owned())
        .unwrap_or_default()
}

/// `findPluginFile`: the directory of the plugin resource file `name` (default
/// `DrawPlugin`): `$CSF_<name>Defaults`, else `$DRAWHOME`, else
/// `$CSF_OCCTResourcePath/DrawResources`, recording the choice in `CSF_<name>Defaults`.
fn find_plugin_file(name: &mut String) -> Option<String> {
    if name.is_empty() {
        name.push_str("DrawPlugin");
    }
    let variable = format!("CSF_{name}Defaults");
    let mut directory = environment(&variable);
    let to_set = directory.is_empty();
    if to_set {
        directory = environment("DRAWHOME");
    }
    if directory.is_empty() {
        let resources = environment("CSF_OCCTResourcePath");
        if resources.is_empty() {
            let _ = write!(
                Message::send_fail(),
                "Failed to load plugin: Neither {variable}, nor CSF_OCCTResourcePath variables have been set"
            );
            return None;
        }
        directory = format!("{resources}/DrawResources");
    }
    let file = format!("{directory}/{name}");
    if !Path::new(&file).exists() {
        let _ = write!(
            Message::send_fail(),
            "Failed to load plugin: File {file} not found"
        );
        return None;
    }
    if to_set && !record_directory(&variable, &directory) {
        let _ = write!(
            Message::send_fail(),
            "Failed to load plugin: Failed to initialize {variable} with {directory}"
        );
        return None;
    }
    Some(directory)
}

/// `OSD_Environment(variable, directory).Build()`: false when it failed.
fn record_directory(variable: &str, directory: &str) -> bool {
    match Environment::with_value(variable, directory) {
        Ok(mut environment) => {
            environment.build();
            !environment.failed()
        }
        Err(_) => false,
    }
}

/// Adds `key` to an ordered set (`NCollection_IndexedMap::Add`).
fn add_unique(set: &mut Vec<String>, key: &str) {
    if !set.iter().any(|existing| existing == key) {
        set.push(key.to_string());
    }
}

/// The value of resource `key`, if defined.
fn resource_value(resources: &ResourceManager, key: &str) -> Option<String> {
    resources
        .find_value(&AsciiString::new(key))
        .map(|value| String::from_utf8_lossy(value.as_bytes()).into_owned())
}

/// `resolveKeys`: replaces groups and aliases by the toolkit keys they lead to. A key
/// whose value names no other key is a toolkit key. The alias set is shared by all the
/// keys of one call, as upstream.
fn resolve_keys(keys: &mut Vec<String>, resources: &ResourceManager) {
    let (mut resolved, mut aliases) = (Vec::new(), Vec::new());
    for key in keys.iter() {
        match resource_value(resources, key) {
            Some(value) => {
                for word in value
                    .split([' ', '\t', ','])
                    .filter(|word| !word.is_empty())
                {
                    if resources.find(word) {
                        add_unique(&mut aliases, word)
                    } else {
                        add_unique(&mut resolved, key)
                    }
                }
            }
            None => {
                let _ = write!(
                    Message::send_fail(),
                    "Pload : Resource = {key} is not found"
                );
            }
        }
        if !aliases.is_empty() {
            resolve_keys(&mut aliases, resources);
        }
        aliases
            .clone()
            .iter()
            .for_each(|alias| add_unique(&mut resolved, alias));
    }
    *keys = resolved;
}

/// `pload [-PluginFile] [keys...]`: loads the plugins of the keys (default `DEFAULT`)
/// and their Tcl scripts.
pub(super) fn pload(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let (mut plugin_file, mut keys) = (String::new(), Vec::new());
    for (index, argument) in a.iter().enumerate().skip(1) {
        let text = AsciiString::new(argument);
        if index == 1 && text.value(1)? == b'-' {
            plugin_file = argument[1..].to_string();
        } else {
            add_unique(&mut keys, argument);
        }
    }
    if keys.is_empty() {
        keys.push("DEFAULT".to_string());
    }
    let Some(directory) = find_plugin_file(&mut plugin_file) else {
        return Ok(1);
    };
    let resources = ResourceManager::with_directories(
        &plugin_file,
        &directory,
        "",
        false,
        ResourceHost::native(),
    )?;
    resolve_keys(&mut keys, &resources);
    let (mut loaded, mut skipped) = (0, 0);
    for key in &keys {
        if load_one(session, di, &resources, (&plugin_file, &directory), key)? {
            loaded += 1;
        } else {
            skipped += 1;
        }
    }
    report_counts(loaded, skipped)
}

/// Loads one toolkit key and its Tcl script; false when skipped (already reported).
fn load_one(
    session: &DrawSession,
    di: &DrawInterpretor,
    resources: &ResourceManager,
    (file, directory): (&str, &str),
    key: &str,
) -> Result<bool, Failure> {
    let Some(value) = resource_value(resources, key) else {
        let _ = write!(
            Message::send_warning(),
            "Pload : Resource = {key} is not found"
        );
        return Ok(false);
    };
    if let Err(failure) = session.load_plugin(key, file, directory, "", false) {
        if failure.exception_type() != "Draw_Failure" {
            return Err(failure);
        }
        let _ = write!(
            Message::send_warning(),
            "Pload : Cannot load plugin {key}: {}",
            failure.message()
        );
        return Ok(false);
    }
    let script_directory = environment("CSF_DrawPluginTclDir");
    let script = format!("{script_directory}/{value}.tcl");
    let default_script = format!("{directory}/{value}.tcl");
    if !script_directory.is_empty() && Path::new(&script).exists() {
        di.eval_file(&script)?;
    } else if !directory.is_empty() && Path::new(&default_script).exists() {
        di.eval_file(&default_script)?;
    }
    Ok(true)
}

/// The summary warnings of `pload`.
fn report_counts(loaded: i32, skipped: i32) -> Result<i32, Failure> {
    if skipped > 0 {
        let _ = write!(
            Message::send_warning(),
            "Pload : Loaded {loaded} plugin(s), skipped {skipped} plugin(s)"
        );
    }
    if loaded == 0 && skipped > 0 {
        let _ = write!(Message::send_fail(), "Pload : No plugins loaded");
        return Ok(1);
    }
    Ok(0)
}

/// `dtryload path`: loads and unloads a shared library, reporting the outcome.
pub(super) fn dtryload(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() != 2 {
        let _ = write!(
            Message::send_fail(),
            "Error: specify path to library to be loaded"
        );
        return Ok(1);
    }
    let mut library = SharedLibrary::named(Some(a[1]));
    if library.dl_open(LoadMode::RtldNow) {
        di.append(format!("Loading {} successful", a[1]))?;
        library.dl_close();
    } else {
        di.append(format!(
            "Loading {} failed: {}",
            a[1],
            library.dl_error().unwrap_or_default()
        ))?;
    }
    Ok(0)
}
