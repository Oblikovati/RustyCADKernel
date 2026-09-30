//! GeomTools behaviour: the text layouts, the reader failure contract, the sets, the
//! undefined-type handler and the preserved native quirks. The package has no upstream
//! GoogleTest suite; byte-exact parity with the native kernel is the numeric oracle's
//! job (tools/verify_tkgeombase_numeric.py), these tests pin the documented behaviour.
mod fakes;
mod handler;
mod reads;
mod sets;
mod texts;
