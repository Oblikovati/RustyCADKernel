# ExtendedString copy-assignment return behavior

The pinned C++ copy-assignment operator calls `Copy(const ExtendedString&)` and returns `*this`. The implementation special-cases self-assignment, otherwise copies the code units. Rust's `copy_assign` adapter uses `Clone::clone_from` for the value and returns the mutable receiver so assignment chaining has an equivalent result.

The native probe checks the declared return type, receiver identity, chained assignment, self-assignment, and assignment from an empty value. Rust supplemental tests check returned receiver identity, chaining, empty-source replacement, and `copy_assign_self`'s self-assignment no-op. `copy_assign_self` makes the native self case callable without violating Rust's aliasing rules. Rust's `Vec` storage does not reproduce OCCT's internal allocator/buffer identity behavior.

Run the native probe against the pinned modeling-full TKernel library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_copy_assignment.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_copy_assignment
/tmp/tkernel_extended_string_copy_assignment
```

The expected output and source-bound audit are committed beside this probe. The supplemental Rust target remains separate from the mapped legacy test target.
