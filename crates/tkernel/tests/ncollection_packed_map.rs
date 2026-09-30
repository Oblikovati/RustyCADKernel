//! Every native typed expansion has a distinct Rust runtime identity.
#![allow(
    non_snake_case,
    unused_mut,
    unused_parens,
    clippy::assertions_on_constants,
    clippy::unnecessary_cast
)]
use rustycad_tkernel::ncollection::{PackedMap, packed_map_algo};
mod cpp_int {
    use super::*;
    type TypeParam = i32;
    include!("ncollection_packed_map/typed_0.rs");
    include!("ncollection_packed_map/typed_1.rs");
    include!("ncollection_packed_map/typed_2.rs");
    include!("ncollection_packed_map/typed_3.rs");
    include!("ncollection_packed_map/typed_4.rs");
}
mod cpp_uint {
    use super::*;
    type TypeParam = u32;
    include!("ncollection_packed_map/typed_0.rs");
    include!("ncollection_packed_map/typed_1.rs");
    include!("ncollection_packed_map/typed_2.rs");
    include!("ncollection_packed_map/typed_3.rs");
    include!("ncollection_packed_map/typed_4.rs");
}
mod cpp_long {
    use super::*;
    type TypeParam = std::ffi::c_long;
    include!("ncollection_packed_map/typed_0.rs");
    include!("ncollection_packed_map/typed_1.rs");
    include!("ncollection_packed_map/typed_2.rs");
    include!("ncollection_packed_map/typed_3.rs");
    include!("ncollection_packed_map/typed_4.rs");
}
mod cpp_int64 {
    use super::*;
    type TypeParam = i64;
    include!("ncollection_packed_map/typed_0.rs");
    include!("ncollection_packed_map/typed_1.rs");
    include!("ncollection_packed_map/typed_2.rs");
    include!("ncollection_packed_map/typed_3.rs");
    include!("ncollection_packed_map/typed_4.rs");
}
mod cpp_size {
    use super::*;
    type TypeParam = usize;
    include!("ncollection_packed_map/typed_0.rs");
    include!("ncollection_packed_map/typed_1.rs");
    include!("ncollection_packed_map/typed_2.rs");
    include!("ncollection_packed_map/typed_3.rs");
    include!("ncollection_packed_map/typed_4.rs");
}
mod signed_int {
    use super::*;
    type TypeParam = i32;
    include!("ncollection_packed_map/signed.rs");
}
mod signed_long {
    use super::*;
    type TypeParam = std::ffi::c_long;
    include!("ncollection_packed_map/signed.rs");
}
mod signed_int64 {
    use super::*;
    type TypeParam = i64;
    include!("ncollection_packed_map/signed.rs");
}
include!("ncollection_packed_map/specific.rs");
