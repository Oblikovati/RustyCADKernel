//! The init files of `Draw_Appli`: `DRAWDEFAULT`, `CSF_OCCTResourcePath`, or a search of
//! the standard installation layouts next to the executable (`searchResources`).
use super::interprete;
use crate::draw::session::DrawSession;
use rustycad_tkernel::osd::{Environment, Process};
use rustycad_tkernel::standard::version_info::{
    occt_version_string_complete, occt_version_string_extended,
};

/// `ReadInitFile`: sources `file` as UTF-8 through the console interpreter.
pub(super) fn read_init_file(session: &DrawSession, file: &str) {
    let file = if cfg!(windows) {
        file.replace('\\', "/")
    } else {
        file.to_string()
    };
    interprete(session, &format!("source -encoding utf-8 \"{file}\""));
}

/// The value of an environment variable through `OSD_Environment` (empty when unset or
/// invalid).
fn environment_value(name: &str) -> String {
    Environment::named(name)
        .map(|mut variable| String::from_utf8_lossy(variable.value().as_bytes()).into_owned())
        .unwrap_or_default()
}

/// Reads `DrawDefault` (or `$DRAWDEFAULT`), defining `CASROOT`, `DRAWHOME`,
/// `CSF_OCCTResourcePath` and `CSF_TestScriptsPath` when the resources are found next to
/// the executable.
pub(super) fn read_default_init_file(session: &DrawSession) {
    let draw_default = environment_value("DRAWDEFAULT");
    if !draw_default.is_empty() {
        read_init_file(session, &draw_default);
        return;
    }
    match draw_home() {
        Some(draw_home) => read_init_file(session, &format!("{draw_home}/DrawDefault")),
        None if cfg!(windows) => read_init_file(session, "ddefault"),
        None => session
            .console()
            .write_out(" the CASROOT variable is mandatory to Run OpenCascade \nNo default file\n"),
    }
}

/// The DrawResources folder from `CSF_OCCTResourcePath` or the executable's layout.
fn draw_home() -> Option<String> {
    let resource_path = environment_value("CSF_OCCTResourcePath");
    if !resource_path.is_empty() {
        return Some(format!("{resource_path}/DrawResources"));
    }
    let found = search_resources("resources", "DrawResources/DrawDefault");
    let draw_home = found.as_ref().map(|(cas_root, resources)| {
        let draw_home = format!("{resources}/DrawResources");
        set_occt_tcl_env("CASROOT", cas_root);
        set_occt_tcl_env("DRAWHOME", &draw_home);
        set_occt_tcl_env("CSF_OCCTResourcePath", resources);
        draw_home
    });
    if environment_value("CSF_TestScriptsPath").is_empty()
        && let Some((_, tests)) = search_resources("tests", "parse.rules")
    {
        set_occt_tcl_env("CSF_TestScriptsPath", &tests);
    }
    draw_home
}

/// `setOcctTclEnv`: defines a variable for OCCT and for Tcl (`env` array), with
/// forward slashes.
fn set_occt_tcl_env(name: &str, path: &str) {
    if path.is_empty() {
        return;
    }
    let path = path.replace('\\', "/");
    if let Ok(mut variable) = Environment::named(name)
        && variable.set_value(&path).is_ok()
    {
        variable.build();
    }
    let _ = rustycad_tcl::put_env(&format!("{name}={path}"));
}

/// The resource layouts relative to an installation prefix, in search order.
fn resource_layouts(resource: &str) -> Vec<String> {
    let (extended, complete) = (
        occt_version_string_extended(),
        occt_version_string_complete(),
    );
    let short = complete
        .rsplit_once('.')
        .map_or(complete, |(major_minor, _)| major_minor);
    let mut layouts: Vec<String> = Vec::new();
    for prefix in ["opencascade", "occt"] {
        for version in [extended, complete, short] {
            layouts.push(format!("/share/{prefix}-{version}/{resource}"));
        }
        layouts.push(format!("/share/{prefix}/{resource}"));
    }
    layouts.push(format!("/{resource}"));
    layouts.extend(["/share/opencascade", "/share/occt", "/share", "/src", ""].map(str::to_string));
    layouts
}

/// `searchResources`: `(CASROOT, resource folder)` of the first layout where
/// `probe_file` exists, looking in the executable folder and up to three levels above.
fn search_resources(resource: &str, probe_file: &str) -> Option<(String, String)> {
    let executable_folder =
        String::from_utf8_lossy(Process::executable_folder().as_bytes()).into_owned();
    for layout in resource_layouts(resource) {
        let probe = format!("{layout}/{probe_file}");
        for up in ["", "../", "../../", "../../../"] {
            if std::path::Path::new(&format!("{executable_folder}{up}{probe}")).exists() {
                let cas_root = format!("{executable_folder}{}", up.trim_end_matches('/'));
                return Some((cas_root.clone(), format!("{cas_root}{layout}")));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_follow_the_upstream_order() {
        let layouts = resource_layouts("resources");
        assert_eq!(
            layouts[0],
            "/share/opencascade-8.1.0.dev1-3d097a0328/resources"
        );
        assert_eq!(layouts[2], "/share/opencascade-8.1/resources");
        assert_eq!(layouts[7], "/share/occt/resources");
        assert_eq!(layouts[8], "/resources");
        assert_eq!(layouts.last().map(String::as_str), Some(""));
        assert_eq!(layouts.len(), 14);
    }
}
