//! Locates the system Tcl/Tk 8.6 libraries with the `pkg-config` program and links them.
//!
//! No bindgen and no pkg-config crate: the declarations are hand-written in `src/sys`, and
//! this script only asks `pkg-config` where the libraries and headers are. Debian/Ubuntu
//! ship `tcl8.6.pc` but `tk8.6.pc` requires a module named `tcl`, which does not exist
//! there; the script writes a one-line `tcl.pc` shim that forwards to `tcl8.6`, so the
//! unmodified `tk8.6.pc` resolves (this is why the `tcl-sys` crate fails on this system).
//! The include directory is exported as `RUSTYCAD_TCL_INCLUDE` for the header checks.
use std::path::{Path, PathBuf};
use std::process::Command;
#[path = "build/pkg_config_path.rs"]
mod pkg_config_path;

/// The pkg-config module that describes the Tcl library.
const TCL_MODULE: &str = "tcl8.6";
/// The pkg-config module that describes the Tk library.
const TK_MODULE: &str = "tk8.6";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    let target = std::env::var("TARGET").unwrap_or_default();
    let host = std::env::var("HOST").unwrap_or_default();
    if target != host {
        // A cross check (clippy for Windows/macOS) never links; the host's pkg-config
        // would describe the wrong platform, so the libraries are not searched.
        println!("cargo:warning=rustycad-tcl: cross target {target}: Tcl/Tk not linked");
        println!("cargo:rustc-env=RUSTYCAD_TCL_INCLUDE=");
        return;
    }
    let shim_dir = write_tcl_shim();
    let include = link_module(TCL_MODULE, None);
    link_module(TK_MODULE, Some(&shim_dir));
    println!("cargo:rustc-env=RUSTYCAD_TCL_INCLUDE={include}");
    println!("cargo:include={include}");
}

/// Writes `tcl.pc` (a forwarder to `tcl8.6`) into OUT_DIR and returns its directory.
fn write_tcl_shim() -> PathBuf {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let shim_dir = out_dir.join("pkgconfig-shim");
    std::fs::create_dir_all(&shim_dir).expect("create the pkg-config shim directory");
    let version = pkg_config(&["--modversion", TCL_MODULE], None);
    let text = format!(
        "Name: tcl\nDescription: forwarder to {TCL_MODULE}\nVersion: {}\nRequires: {TCL_MODULE}\n",
        version.trim()
    );
    std::fs::write(shim_dir.join("tcl.pc"), text).expect("write the tcl.pc shim");
    shim_dir
}

/// Emits the link search paths and libraries of `module` (stub libraries excluded: they
/// serve loadable extensions, an embedding application links the real library) and
/// returns its first include directory.
fn link_module(module: &str, extra_path: Option<&Path>) -> String {
    let search = pkg_config(&["--libs-only-L", module], extra_path);
    for directory in search
        .split_whitespace()
        .filter_map(|f| f.strip_prefix("-L"))
    {
        println!("cargo:rustc-link-search=native={directory}");
    }
    let libraries = pkg_config(&["--libs-only-l", module], extra_path);
    for library in libraries
        .split_whitespace()
        .filter_map(|f| f.strip_prefix("-l"))
    {
        if !library.contains("stub") {
            println!("cargo:rustc-link-lib=dylib={library}");
        }
    }
    let cflags = pkg_config(&["--cflags-only-I", module], extra_path);
    let include = cflags.split_whitespace().find_map(|f| f.strip_prefix("-I"));
    include.unwrap_or("/usr/include").to_string()
}

/// Runs `pkg-config` and returns its standard output; a missing program or module stops
/// the build with a message that names what to install.
fn pkg_config(arguments: &[&str], extra_path: Option<&Path>) -> String {
    let mut command = Command::new("pkg-config");
    command.args(arguments);
    if let Some(directory) = extra_path {
        let current = std::env::var_os("PKG_CONFIG_PATH").unwrap_or_default();
        let joined = pkg_config_path::prepend(directory, &current).unwrap_or_else(|error| {
            panic!("pkg-config path {directory:?}: expected native path list: {error}")
        });
        command.env("PKG_CONFIG_PATH", joined);
    }
    let output = command.output().unwrap_or_else(|error| {
        panic!("rustycad-tcl: cannot run pkg-config ({error}); install pkg-config")
    });
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "rustycad-tcl: `pkg-config {}` failed: {stderr}; install the Tcl/Tk 8.6 \
             development packages (tcl8.6-dev, tk8.6-dev)",
            arguments.join(" ")
        );
    }
    String::from_utf8(output.stdout).expect("pkg-config prints UTF-8")
}
