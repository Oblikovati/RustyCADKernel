# Elementary surface conversion storage

This slice exposes a safe Rust construction path for the protected storage protocol
of `Convert_ElementarySurfaceToBSplineSurface`. It lets Rust callers implement new
conversions without C++ subclass ABI or layout compatibility.

## Verified behavior

The derived-class native probe initializes every array cell, changes active counts,
and calls `Finalize`. It verifies 2D pole/weight positions, knots and multiplicities,
output dimensions, idempotence, and the native mutation order when a negative U-knot
count fails. Both implementations trim poles and weights before failing at the U-knot
array. Both expose zero-capacity construction. One malformed input differs: native
accepts a negative pole capacity and reports `NbRows() == -1`; Rust rejects that
invalid bound with `Standard_OutOfMemory`. The negative-count diagnostic also differs:
Rust reports the invalid bounds, while the pinned native allocator reports a failed
`realloc`. These invalid-capacity/error-message cases remain partial.

A derived native fixture rebases all protected arrays without changing their extents.
`Pole(1, 1)` and `UKnot(1)` then raise `Standard_OutOfRange` from the underlying
NCollection array. Rust accessors now perform the same active-count checks first, then
use checked array reads and return `Standard_OutOfRange` with the requested index and
actual stored bounds; they no longer panic. The native and Rust diagnostic strings
are different, but the exception class and ordering match. The safe mutable array
accessors can create these rebased states, so this is part of their tested behavior.

The test covers all six scalar accessors under custom lower bounds, plus a too-short
pole extent and an index that fails the active-count check before touching storage.
It verifies that `Finalize` restores one-based bounds when trimming is required. It also covers array trimming,
retained initialized values, idempotence, zero capacity, and the ordered Finalize
failure point. Rust initializes allocated cells because uninitialized C++ storage has
no defined values to preserve; this is not treated as an observable parity gap.
C++ exceptions are represented by Rust `Failure` results and are not a parity gap.
Custom OCCT memory-manager diagnostic text is not covered.

Rust evidence is `crates/tkmath/tests/convert_elementary_surface_storage.rs`.
Native output is captured in `tools/oracle/tkmath_elementary_surface_finalize.expected`.

## Callable crosswalk records

The constructor and `Finalize()` mappings are partial for the invalid-capacity and
allocator-diagnostic differences described above. Protected state fields are mapped
to public typed getters/setters or immutable/mutable array accessors, which provide
read/write behavior without exposing C++ layout. These field mappings apply to valid
capacities and initialized values. Their compiler IDs are retained in the source
inventory; the callable IDs for constructor and `Finalize()` are listed below.

| Native declaration ID | Native member | Rust counterpart |
| --- | --- | --- |
| `7ad17411e0bbe391a2f184bb21d9c9818b8ee5b7f17e182db1222e27cd829173` | protected six-argument constructor | `ElementarySurfaceToBSplineSurface::with_capacity` |
| `da07de0f77704c07687378f79e76954b88b8648029fb42534ee1fa7be95749e6` | protected `Finalize()` | `ElementarySurfaceToBSplineSurface::finalize` |

## Reproduction

```sh
occt=target/occt-tkernel-full
c++ -std=c++17 -O2 -Wno-deprecated-declarations -I"$occt/include/opencascade" \
  tools/oracle/tkmath_elementary_surface_finalize.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$occt/lin64/gcc/libi" \
  -lTKMath -lTKernel -o /tmp/tkmath_elementary_surface_finalize
/tmp/tkmath_elementary_surface_finalize
```

## Pinned source hashes

| File | SHA-256 |
| --- | --- |
| OCCT `Convert_ElementarySurfaceToBSplineSurface.hxx` | `cbc1a0515a106bae9212de10ba32cbb18c4d93071855e28d6387ae37270f7cef` |
| OCCT `Convert_ElementarySurfaceToBSplineSurface.cxx` | `7cd6847b7aa64c6c97f1748359efe8fc878dd2d37c8194e018e42328e5dc2571` |
| Native probe | `9f43da7f2e52a9a0bea386dacca49fa950b48cfe3e68f53c7715641c3b8705a0` |
