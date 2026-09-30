//! OCCT's Plugin package: loading a service implementation identified by a GUID.
//!
//! Loading native C++ shared libraries has no meaning for the Rust port, so libraries are
//! a seam ([`PluginLibraries`]) where factories are registered by library file name; the
//! resource lookup, library naming and `Plugin_Failure` messages follow `Plugin.cxx`.
use crate::standard::exceptions::define_standard_exceptions;
mod libraries;
mod loader;
pub use libraries::{PluginFactory, PluginLibraries, StaticPluginLibraries, library_file_name};
pub use loader::PluginLoader;

define_standard_exceptions! {
    PLUGIN_HIERARCHY;
    failure => "Plugin_Failure" : "Standard_Failure";
}
