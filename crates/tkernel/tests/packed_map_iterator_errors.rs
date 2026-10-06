//! Exact legacy exception payloads and message-only construction at every iterator state.
#[path = "packed_map_iterator_errors/mod.rs"]
mod contracts;
use contracts::check_width;
macro_rules! width {
    ($name:ident, $ty:ty) => {
        mod $name {
            use super::*;
            #[test]
            fn negative_stack_length() {
                check_width::<$ty>(stringify!($ty), -3);
            }
            #[test]
            fn zero_stack_length() {
                check_width::<$ty>(stringify!($ty), 0);
            }
            // libc backtrace captures real machine frames and is outside Miri's FFI domain.
            #[cfg(not(miri))]
            #[test]
            fn positive_stack_length() {
                check_width::<$ty>(stringify!($ty), 3);
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
