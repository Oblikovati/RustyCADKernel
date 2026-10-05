# ExtendedString UTF-16 C-string mutation oracle

The pinned pointer wrappers for `Cat`, `Insert`, `SetValue`, and `Prepend` compute their length with `extStringLen`, so each reads only through the first zero code unit. The explicit-length overloads retain embedded zeros. `Insert` validates its one-based position before its null/length no-op; `SetValue` likewise checks `1..=Length()+1` before null/length. `Prepend` treats null and empty input as no-op.

The fixture uses `A, NUL, B` against `XY`. It records pointer mutation results, empty/null behavior, and native exception classes/messages for invalid index plus null input. Rust regressions compare the new pointer-like bounded-slice adapters with the existing explicit-span APIs.

Build and run against pinned OCCT source `3d097a0328e71b826377d4814ab05ec3c3d23871` and the rebuilt `target/occt-modeling-full` TKernel library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_c_units_mutations.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_c_units_mutations
/tmp/tkernel_extended_string_c_units_mutations
```

The observed output is committed in `tkernel_extended_string_c_units_mutations.expected`. The Rust adapters accept bounded slices and model null with `None`; they cover non-aliased buffers. Native `Prepend` has a self-aliasing branch and raw pointer mutation can alias the receiver. Those aliasing cases are not claimed by these safe APIs, so the three mutating C-string rows remain partial. `Cat` is read-only and maps to the existing bounded `cat_c_units` implementation.
