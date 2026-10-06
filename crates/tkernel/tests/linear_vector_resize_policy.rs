//! Resize compares direct native states separately from explicit native tail recovery.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "linear_vector_resize_policy/fixture.rs"]
mod fixture;
#[path = "linear_vector_resize_policy/item.rs"]
mod item;
#[path = "linear_vector_resize_policy/recovery.rs"]
mod recovery;
use fixture::{ResizeSpec, resize_case};
use item::{ResizeItem, ResizeTrace};

macro_rules! native_resize {
    ($test:ident,$row:literal,$cap:expr,$count:expr,$size:expr,$fill:expr,$moving:expr,$fail:expr) => {
        #[test]
        fn $test() {
            resize_case(ResizeSpec {
                name: $row,
                capacity: $cap,
                count: $count,
                size: $size,
                fill: $fill,
                moving: $moving,
                fail: $fail,
            });
        }
    };
}
native_resize!(empty, "empty", 0, 0, 0, false, false, 0);
native_resize!(reserved_empty, "reserved-empty", 4, 0, 0, true, false, 0);
native_resize!(same_default, "same-default", 4, 2, 2, false, false, 0);
native_resize!(same_fill, "same-fill", 4, 2, 2, true, false, 0);
native_resize!(kept_default, "kept-default", 6, 2, 4, false, false, 0);
native_resize!(kept_fill, "kept-fill", 6, 2, 4, true, false, 0);
native_resize!(move_default, "move-default", 2, 2, 5, false, true, 0);
native_resize!(move_fill, "move-fill", 2, 2, 5, true, true, 0);
native_resize!(copy_default, "copy-default", 2, 2, 3, false, false, 0);
native_resize!(copy_fill, "copy-fill", 2, 2, 3, true, false, 0);
native_resize!(empty_default, "empty-default", 0, 0, 3, false, false, 0);
native_resize!(empty_fill, "empty-fill", 0, 0, 1, true, false, 0);
native_resize!(shrink_default, "shrink-default", 6, 4, 1, false, false, 0);
native_resize!(shrink_fill, "shrink-fill", 6, 4, 1, true, false, 0);
native_resize!(zero_default, "zero-default", 4, 2, 0, false, false, 0);
native_resize!(zero_fill, "zero-fill", 4, 2, 0, true, false, 0);
native_resize!(first_default, "first-default", 2, 2, 4, false, true, 1);
native_resize!(first_fill, "first-fill", 6, 2, 4, true, false, 1);
native_resize!(late_default, "late-default", 2, 2, 5, false, true, 3);
native_resize!(late_fill, "late-fill", 6, 2, 5, true, false, 3);
