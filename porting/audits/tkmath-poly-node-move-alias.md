# Poly node array move-alias behavior

`Poly_ArrayOfNodes::Move` and `Poly_ArrayOfUVNodes::Move` have a defined observable behavior that the Rust wrappers do not preserve. The native base `NCollection_AliasedArray::Move` copies stride, byte size, and data pointer into the destination, then clears only the source's `myDeletable` flag. The source keeps its length and remains a non-owning alias of the moved bytes. Writes through either array are visible through the other while both objects are alive.

The Rust `NodeArray::move_from` transfers its `AliasedArray` and takes the source length to zero. The focused diagnostic test records that current source behavior and is intentionally not a native-parity test. Mapping rows for both `Move` methods are therefore partial. Move assignment syntax and move constructors remain Rust ownership operations rather than callable API rows.

## Native reproduction

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Compiler: `/usr/bin/c++`, GCC 13.3.0. Both native classes are tested with owned double-precision storage. Output is compared byte-for-byte with `tools/oracle/tkmath_poly_node_move_alias.expected`.

The isolated Rust branch starts from `main` at `06e1bea86048db2d4a44ed98c1a69c19f01fc28a`. Its diagnostic tests pass by asserting the current source-empty behavior; they do not assert native equivalence. `cargo test --locked -p rustycad-tkmath --test poly_node_move_alias_gap` passed 2 tests. Strict targeted Clippy and `cargo fmt --all -- --check` passed.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
/usr/bin/c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
  tools/oracle/tkmath_poly_node_move_alias.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$occt/lin64/gcc/libi" \
  -lTKMath -lTKernel -o /tmp/tkmath_poly_node_move_alias
/tmp/tkmath_poly_node_move_alias
```

```text
2 2 0 1
4 5 6
7 8 9
2 2 0 1
4 5
7 8
```

The first row for each type is source size, target size, source deletability, and target deletability. The next rows show target writes read through the source, then source writes read through the target.

## Safe Rust design limit

Keeping those two aliases live would require shared interior storage across both wrappers. `NodeArray` also wraps an external mutable slice, where duplicating mutable access would violate Rust's aliasing rules. The current `AliasedArray` owns or exclusively borrows its storage and exposes mutable access through `&mut self`; it cannot retain the native source view after ownership transfer. No unsafe alias was introduced. A future safe shared-storage redesign would need to make read/write access fallible when another borrow is active and explicitly preserve the external-slice constructor's exclusivity.

## Source fingerprints

| Input | SHA-256 |
| --- | --- |
| OCCT `NCollection_AliasedArray.hxx` | `b3dfbe5bee9a06c2a1e4f953132faa3c32280d0e22720f749bdba228dc1466ac` |
| OCCT `Poly_ArrayOfNodes.hxx` | `99a034ab77ee99b378e676dc435aadb85f852daaa60a47efa257dda8bab1852d` |
| OCCT `Poly_ArrayOfUVNodes.hxx` | `d261be3da5834202d68df741a2b7e75b0288f804d36378f3cc4cd55914413748` |
| Native probe | `a00db46a21c0d53c78f75a32dbbbe0cb6a533be3a305794d5ddf6d65892ea1b8` |
| Native expected output | `2d0e3e005c273610275c507bf206785297d01444b4e09a9ab6547d8920a925ed` |
| Rust `NodeArray` | `d6fa6ef1c06ffef44be8cf58c40e1e149732c94f17d256e549dbfe1165c507bd` |
| Diagnostic Rust test | `320794474701fe9015c24e2023c39b1a475c47db4cef6b214219c776c1d2c993` |
