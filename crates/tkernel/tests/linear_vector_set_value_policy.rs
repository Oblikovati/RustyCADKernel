//! SetValue publishes successful default extension before assigning the requested slot.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "linear_vector_set_value_policy/fixture.rs"]
mod fixture;
#[path = "linear_vector_set_value_policy/item.rs"]
mod item;
#[path = "linear_vector_set_value_policy/recovery.rs"]
mod recovery;
use fixture::{SetSpec, set_case};
use item::{SetItem, SetTrace};

macro_rules! native_set {
    ($test:ident,$row:literal,$cap:expr,$count:expr,$index:expr,$moving:expr,$reloc:expr,$fail:expr,$assign:expr) => {
        #[test]
        fn $test() {
            set_case(SetSpec {
                name: $row,
                capacity: $cap,
                count: $count,
                index: $index,
                moving: $moving,
                relocating: $reloc,
                fail: $fail,
                assignment_fail: $assign,
            });
        }
    };
}
native_set!(
    copy_existing,
    "copy-existing",
    4,
    2,
    0,
    false,
    false,
    0,
    false
);
native_set!(
    move_existing,
    "move-existing",
    4,
    2,
    1,
    true,
    false,
    0,
    false
);
native_set!(copy_grow, "copy-grow", 2, 2, 4, false, true, 0, false);
native_set!(move_grow, "move-grow", 2, 2, 4, true, true, 0, false);
native_set!(
    copy_fallback,
    "copy-fallback",
    2,
    2,
    2,
    false,
    false,
    0,
    false
);
native_set!(
    move_fallback,
    "move-fallback",
    2,
    2,
    2,
    true,
    false,
    0,
    false
);
native_set!(copy_kept, "copy-kept", 6, 2, 4, false, false, 0, false);
native_set!(move_kept, "move-kept", 6, 2, 4, true, false, 0, false);
native_set!(empty_copy, "empty-copy", 0, 0, 0, false, false, 0, false);
native_set!(empty_move, "empty-move", 0, 0, 2, true, false, 0, false);
native_set!(
    copy_fail_existing,
    "copy-fail-existing",
    4,
    2,
    1,
    false,
    false,
    0,
    true
);
native_set!(
    move_fail_existing,
    "move-fail-existing",
    4,
    2,
    0,
    true,
    false,
    0,
    true
);
native_set!(
    copy_fail_grow,
    "copy-fail-grow",
    2,
    2,
    3,
    false,
    true,
    0,
    true
);
native_set!(
    move_fail_grow,
    "move-fail-grow",
    2,
    2,
    3,
    true,
    false,
    0,
    true
);
native_set!(
    copy_fail_kept,
    "copy-fail-kept",
    6,
    2,
    4,
    false,
    false,
    0,
    true
);
native_set!(
    move_fail_kept,
    "move-fail-kept",
    6,
    2,
    4,
    true,
    false,
    0,
    true
);
native_set!(first_copy, "first-copy", 2, 2, 4, false, true, 1, false);
native_set!(first_move, "first-move", 6, 2, 4, true, false, 1, false);
native_set!(late_copy, "late-copy", 2, 2, 4, false, true, 3, false);
native_set!(late_move, "late-move", 6, 2, 4, true, false, 3, false);
