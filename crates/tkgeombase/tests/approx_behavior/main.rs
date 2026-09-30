//! Approx behaviour: the error contracts, the defaults, the refinement scripts of the
//! Approx line members and the preserved native quirks. The package has no upstream
//! GoogleTest suite; bit-exact parity with the native kernel is the numeric oracle's
//! job (tools/verify_tkgeombase_numeric.py), these tests pin the documented behaviour.
mod curves;
mod drivers;
mod fakes;
mod sweep;
