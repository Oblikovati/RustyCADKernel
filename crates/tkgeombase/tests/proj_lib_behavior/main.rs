//! ProjLib behaviour: the contracts of the projectors and the approximations and the
//! preserved native quirks (docs/tkgeombase-progress.md, Preserved native behaviour).
//! Bit-exact parity with the native kernel is the numeric oracle's job
//! (tools/verify_tkgeombase_numeric.py); these tests pin the documented behaviour.
mod approximations;
mod degenerate_ranges;
mod elementary;
mod support;
