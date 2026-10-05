# ExtendedString UTF-16 C-string Copy oracle

The pointer `Copy` wrapper computes `extStringLen`, so a bounded source is copied only through its first NUL. The explicit pointer-and-length overload preserves embedded zero units. Native Copy with a null or empty pointer clears a nonempty receiver. Pointer assignment uses the same Copy path and returns the receiver by reference.

The fixture compares `A\0B` pointer input with an explicit length of three, tests pointer assignment reference identity, and confirms that null and empty pointer copies clear the receiver. The safe Rust adapter takes `Option<&[u16]>`; it cannot model invalid pointers or native pointer reads beyond the supplied slice.

Build and run against the pinned modeling-full native library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_c_units_copy.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_c_units_copy
/tmp/tkernel_extended_string_c_units_copy
```

Expected native output is committed in `tkernel_extended_string_c_units_copy.expected`. Source, declaration, Rustdoc, native library and test log hashes are bound in `porting/audits/extended-string-c-units-copy.json`.
