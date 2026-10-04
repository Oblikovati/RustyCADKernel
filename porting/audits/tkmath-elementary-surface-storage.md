# Elementary surface conversion storage

This slice exposes a safe Rust construction path for the protected storage protocol
of `Convert_ElementarySurfaceToBSplineSurface`. It is an extension API for Rust
callers implementing new conversions, not an implementation of C++ subclass ABI or
inheritance. The mutable array views retain Rust bounds and ownership.

## Verified behavior

The native derived-class probe initializes every array cell, changes the protected
active pole/knot counts, and calls `Finalize`. It checks 2D pole/weight positions,
the retained knots/multiplicities, and output dimensions. Calling `Finalize` a
second time leaves the state unchanged. A negative U-knot count fails after poles
and weights have already been trimmed and before U/V knot arrays change. Native
reports `Standard_OutOfMemory`; Rust reports the same exception class and preserves
the same earlier trims. Rust's message describes the invalid `[1, -1]` bounds,
while this pinned native allocator reports its failed `realloc` message. Custom
memory-manager diagnostics are not covered.

Focused Rust evidence is `crates/tkmath/tests/convert_elementary_surface_storage.rs`.
It writes the complete capacities, then asserts final counts, degrees, periodicity,
retained array values, idempotence, and state at the ordered failure point. The
native output is captured in `tools/oracle/tkmath_elementary_surface_finalize.expected`.

## Callable crosswalk records

The compiler inventory IDs below are exact candidates from the pinned
`occt-declarations.jsonl.gz`. The protected constructor and `Finalize` are exposed
as the safe Rust `with_capacity` and `finalize` entrypoints. Protected fields are
represented with typed mutable accessors and state setters for Rust subclass-like
construction, but no field mapping is claimed because Rust exposes methods rather
than layout-compatible protected fields.

| Native declaration ID | Native member | Rust counterpart |
| --- | --- | --- |
| `7ad17411e0bbe391a2f184bb21d9c9818b8ee5b7f17e182db1222e27cd829173` | protected six-argument constructor | `ElementarySurfaceToBSplineSurface::with_capacity` |
| `da07de0f77704c07687378f79e76954b88b8648029fb42534ee1fa7be95749e6` | protected `Finalize()` | `ElementarySurfaceToBSplineSurface::finalize` |

These two mappings are partial: the Rust constructor is fallible and uses owned
initialized arrays, and the negative-count Finalize diagnostic differs from the
pinned native allocator message. The tested values, array dimensions, trimming
order, mutation-before-failure, and exception class match. This does not establish
general subclass ABI or field-reference lifetime parity.

## Reproduction

```sh
occt=target/occt-tkernel-full
c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
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
| Native probe | `18e269dc52d6dfb0cedaa96de82170f41f3f4d930bf8cddd47f6080e69c43300` |
