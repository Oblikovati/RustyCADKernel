//! Behavior of the FSD drivers and Base64 over in-memory and failing file systems. The
//! native byte formats are compared with OCCT by the `stor_*` probe requests.
#[path = "fsd_api_behavior/base64.rs"]
mod base64;
#[path = "fsd_api_behavior/binary.rs"]
mod binary;
#[path = "fsd_api_behavior/fakes.rs"]
mod fakes;
#[path = "fsd_api_behavior/text.rs"]
mod text;
