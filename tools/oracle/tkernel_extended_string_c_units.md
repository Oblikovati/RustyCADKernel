# `TCollection_ExtendedString` UTF-16 C-string concatenation

Pinned `TCollection_ExtendedString::extStringLen` scans to the first zero unit
and returns zero for a null pointer. The `operator+=` pointer body does nothing
for null; its array-template body calls the same helper. `operator+` calls the
helper and `Cat(pointer, 0)`, producing an unchanged copy. The `u16string_view`
overload instead appends its explicit span, including embedded zero units.

Rust now exposes bounded `assign_cat_c_units` and `cat_c_units` adapters. `None`
models a null pointer without dereferencing it. The existing `assign_cat` and
`cat` span path remains length preserving, so an explicit slice can contain
embedded zero code units.

## Native fixture

The C++ fixture was compiled against the modeling-full TKernel build from OCCT
revision `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 -O2 \
  -I<modeling-full>/include/opencascade \
  tools/oracle/tkernel_extended_string_c_units.cxx \
  <modeling-full>/lin64/gcc/libi/libTKernel.so.8.1.0 \
  -Wl,-rpath,<modeling-full>/lin64/gcc/libi \
  -o /tmp/tkernel_extended_string_c_units
```

Its output matches `tkernel_extended_string_c_units.expected`, covering array,
pointer, explicit-view, null-pointer, and ordinary terminated-input paths.

## Source hashes

| Input | SHA-256 |
| --- | --- |
| `TCollection_ExtendedString.hxx` | `5a483b0daeb75b79e50cdb43cfe53fa7da87a3f477779d3aaece10e96d36acda` |
| `TCollection_ExtendedString.cxx` | `bdcbe42ede4a95a5b1928ad37cc7388e9d5ce2fb84f32f78e24a62c7681fc22c` |
| `libTKernel.so.8.1.0` | `6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee` |
