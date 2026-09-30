//! Behaviour of the Quantity members completed beyond the upstream GoogleTest suites.
//! Bit-exact agreement with OCCT is checked by the `qc_*`/`qrgba_*` differential probe.
#[path = "quantity_api_behavior/colors.rs"]
mod colors;
#[path = "quantity_api_behavior/encodings.rs"]
mod encodings;
#[path = "quantity_api_behavior/rgba_and_time.rs"]
mod rgba_and_time;
