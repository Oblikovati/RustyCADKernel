//! BinTools behaviour: the scalar encodings and stream primitives, the geometry and
//! location sets, the grouped shape set, the shape-reference writer and reader, the
//! file variants, and the preserved native quirks. The package has no upstream
//! GoogleTest suite; byte-exact parity with the native kernel is the job of the br_bin*
//! requests of the numeric oracle (tools/verify_tkbrep_numeric.py).
mod fakes;
mod files;
mod references;
mod sets;
mod shape_set;
mod streams;
#[path = "../support/mod.rs"]
mod support;
