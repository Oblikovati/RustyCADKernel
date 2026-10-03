# Angular surface splitting oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
This repository-owned fixture is separate from legacy test declarations.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_split_surface_angle.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKG3d -lTKMath -lTKernel \
  -o /tmp/rustycad-surface-angle-oracle
/tmp/rustycad-surface-angle-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_split_surface_angle
```

Rows record the name, status bits, split-sequence identity retention, and U split
values. Rust compares the recorded values by binary64 bits. The fixture covers
all five accepted rotational dynamic types, one/multiple angular intervals,
small/equal/wide trimmed bounds, offsets, nested wrappers, unsupported planes,
and a negative angular limit whose segment-count conversion is defined.

Native Compute unwraps exactly one wrapper; a trim of an offset is not recognized
as a rotational surface. Compute does not set DONE1 for multiple intervals; Build
supplies that status later. Existing status is retained unless the one-segment
DONE2 branch overwrites it.

## Undefined native conversions

Native `int((uLength - Precision::Angular()) / maxAngle) + 1` is undefined when
the truncated quotient cannot fit in `int`, is nonfinite, or the addition overflows.
Rust returns `Standard_OutOfRange` for those values and tests this response for
zero, NaN, and tiny positive angular limits. Those tests document a defined Rust
boundary; they do not establish parity for undefined C++ executions.
