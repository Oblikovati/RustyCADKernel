//! AppDef behaviour: the error contracts and the preserved native quirks of the
//! containers, the line tool, LinearCriteria and Variational. The package has no
//! upstream GoogleTest suite; bit-exact parity with the native kernel is the numeric
//! oracle's job (tools/verify_tkgeombase_numeric.py), these tests pin the documented
//! behaviour. The process-wide reference cache of the FEmTool criteria is pinned in
//! its own binary (`app_def_reference_cache.rs`), as it depends on earlier calls.
mod containers;
mod criteria;
mod support;
mod variational;

pub use support::{arc_line, constraints, kind};
