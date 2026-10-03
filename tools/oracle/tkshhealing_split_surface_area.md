# Area surface splitting oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`. The probe records part
counts, shared U/V sequence identity, and the split parameters after `Compute(false)`.
Each parameter is compared as a binary64 value by the Rust fixture.

```sh
g++ -std=c++17 -I/path/to/occt-install/include/opencascade \
  tools/oracle/tkshhealing_split_surface_area.cxx \
  -L/path/to/occt-install/lib -Wl,-rpath,/path/to/occt-install/lib \
  -lTKShHealing -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/rustycad-split-surface-area
LD_LIBRARY_PATH=/path/to/occt-install/lib:/path/to/occt-build/lin64/gcc/lib \
  /tmp/rustycad-split-surface-area
cargo test -p rustycad-tkshhealing --test shape_upgrade_split_surface_area
```

The cases cover no-op defaults, square aspect ratio, long U and V regions, the
non-square part-count rounding path, fixed U/V counts in both aspect orientations,
automatic square counts from physical isoparametric lengths, and partially invalid
fixed counts retaining the previously selected `NbParts`.

## Undefined native domains

OCCT performs signed integer multiplication for positive fixed U/V counts and casts
floating square counts to `int` without a range check. Overflow, or conversion of a
nonfinite/out-of-range quotient, has undefined C++ behavior. The Rust API reports
`Standard_OutOfRange` for those inputs; this boundary is not counted as native parity.
Square splitting also requires the base `Init(..., area)` overload with positive area,
because natural-bound initialization does not set the native `myArea`, `myUsize`, or
`myVsize` metrics.
