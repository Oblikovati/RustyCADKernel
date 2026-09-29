//! The GeomLib probe; the counterpart of tools/oracle/tkgeombase_probe_geomlib.cxx (the
//! static members, statics.rs) and tkgeombase_probe_geomlib_classes.cxx (classes.rs).
mod classes;
mod statics;
pub use classes::*;
pub use statics::*;
