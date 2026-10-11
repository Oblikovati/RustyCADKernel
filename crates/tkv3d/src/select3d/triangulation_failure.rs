//! Checked native exceptions cross nonfallible BVH traits as typed panic payloads.
pub(super) use rustycad_tkernel::standard::{
    catch_native_failure as catch_triangulation_failure, raise_native_failure as native_checked,
};
