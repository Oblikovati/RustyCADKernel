//! Native ReSize bucket order and allocation trace for every integral key width.
#[path = "packed_map_native_rehash/oracle.rs"]
mod oracle;
#[path = "packed_map_native_rehash/trace.rs"]
mod trace;
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { oracle::compare::<$ty>(stringify!($ty)); })+ };
}
widths!(no_scratch_i8: i8, no_scratch_u8: u8, no_scratch_i16: i16, no_scratch_u16: u16, no_scratch_i32: i32, no_scratch_u32: u32, no_scratch_i64: i64, no_scratch_u64: u64, no_scratch_usize: usize, no_scratch_isize: isize, no_scratch_bool: bool);
