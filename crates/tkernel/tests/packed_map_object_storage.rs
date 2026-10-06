//! Raw DEFINE_STANDARD_ALLOC contracts and real object lifetimes within owned byte storage.
#[path = "packed_map_object_storage/mod.rs"]
mod contracts;
macro_rules! width {
    ($module:ident, $ty:ty) => {
        mod $module {
            use super::contracts;
            #[test]
            fn scalar_bytes() {
                contracts::raw::<$ty>(stringify!($ty), false);
            }
            #[test]
            fn array_bytes() {
                contracts::raw::<$ty>(stringify!($ty), true);
            }
            #[test]
            fn placement_operations() {
                contracts::placement::<$ty>(stringify!($ty));
            }
            #[test]
            fn scalar_and_array_lifetimes() {
                contracts::lifetimes::<$ty>(stringify!($ty));
            }
            #[test]
            fn failed_placement_retains_caller_storage() {
                contracts::failed_placement::<$ty>(stringify!($ty));
            }
        }
    };
}
width!(signed_32, i32);
width!(unsigned_32, u32);
width!(signed_64, i64);
width!(unsigned_64, u64);
width!(signed_byte, i8);
width!(boolean, bool);
