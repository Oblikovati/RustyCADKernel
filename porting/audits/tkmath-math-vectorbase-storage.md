# TKMath `math_VectorBase` storage crosswalk

This audit covers the storage and construction slice of `math_VectorBase<T>` in
the pinned OCCT checkout at revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
It is not a completeness claim for `math_VectorBase`, `math_Vector`, or TKMath.

| Pinned C++ declaration / behavior | Rust path | Evidence |
| --- | --- | --- |
| `math_VectorBase(size_t)` and `math_VectorBase(size_t, T)` | `VectorBase::with_size`, `VectorBase::with_size_filled` | Rust implementation chooses inline storage through 32 elements and owned heap storage above that. |
| `math_VectorBase(int, int)` and `math_VectorBase(int, int, T)` | `VectorBase::with_bounds`, `VectorBase::filled` | Inclusive bounds and empty `[lower, lower-1]` ranges are represented. |
| `math_VectorBase(const T*, int, int)` | `VectorBase::from_buffer(&mut [T], lower, upper)` | `NCollection_Array1(*theTab, ...)` wraps the pointer by default; it does not copy. Rust requires mutable backing storage to express the defined two-way aliasing safely. |
| `math_VectorBase(gp_XY const&)`, `math_VectorBase(gp_XYZ const&)` | `VectorBase::from_xy`, `VectorBase::from_xyz` | Existing conversion constructors set bounds to 1..2 and 1..3. |
| Copy construction and assignment | `Clone`, `initialized` | Rust clone owns a copy; assignment copies matching-length values while retaining destination bounds. |
| Move construction | `VectorBase::move_construct` | Existing implementation transfers owned storage and copies inline/borrowed storage. |
| Protected `SetLower(int)` | `VectorBase::set_lower` | Newly added subclass-facing capability; the native probe exposes the protected member through a derived class. Rust's method is public because this port has no C++ inheritance boundary. |
| `Resize(int)` | `VectorBase::resize` | Existing inline/heap transitions retain the initialized prefix; borrowed-to-inline follows native's no-copy stack transition, while borrowed-to-heap copies through `Array.Resize`. |

## Native and Rust evidence

`tools/oracle/math_vector_base_copy.cpp` compiles against the pinned native build
with:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-tkernel-full/include/opencascade \
  tools/oracle/math_vector_base_copy.cpp \
  -L target/occt-tkernel-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-tkernel-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/math_vector_base_copy
/tmp/math_vector_base_copy
```

Observed output:

```text
5 7 -9 2.25 -10 22 -9 77 32.25 77
```

The output demonstrates that changing the original pointer-backed array is visible
through both vector views, writing through one view changes the original array, and
`SetLower` shifts the index interval without relocating values. The probe uses a
3-element inline view and a 33-element heap-backed view.

Rust regressions are in `crates/tkmath/tests/vector_behavior.rs`:
`external_buffer_aliases_changes_in_both_directions` covers two-way aliasing, and
`changing_lower_bound_relabels_inline_and_heap_values` checks both storage tiers.
The focused run passed 6 tests; targeted Clippy passed with `-D warnings`.

The pointer constructor's `const T*` spelling allows callers to provide read-only
storage, but mutating the resulting vector would then write through a const-cast
pointer. The Rust API intentionally accepts `&mut [T]`; parity evidence applies to
mutable-backed arrays, where native behavior is defined. No copy-constructor claim
is made for the pointer overload.

## Pinned input hashes

The source hashes below bind this note to the reviewed declarations and bodies.

| File | SHA-256 |
| --- | --- |
| OCCT `math_VectorBase.hxx` | `d68820c34a6cb97d0eeec2639b9c1948c9e0934ddc9cba5d88b22256c0f68369` |
| OCCT `math_VectorBase.lxx` | `d14b14d7cab257576b442bc9612e2bb3c0781c2efa0ff0dcb0cb129a808ad46c` |
| OCCT `NCollection_Array1.hxx` | `5757b7e442ae800da44a8584b9eda60a16624bbbf35efa82b766439c87bed3b6` |
| Rust `vector.rs` | `75540d94db723d1fc3f9c50165fa69be4d522049cda8dc590a33c1f430dc333f` |
| Rust `storage.rs` | `1616375cb64efa51b194601a68a7fb03decadf7a3169ddda3470d0db7ab1f013` |
| Rust `vector_behavior.rs` | `44decdec89a7aa11ebcbc91f98044bd76555ae87c5d7e4658a04133b3afd2754` |
| Native probe | `2727166623237103adef8fc76118d0957abae137597230a48d926956b1754be1` |
