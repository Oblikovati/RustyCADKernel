# Outer-bound wire checks

The C++ fixture links pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It visits six faces of a box,
checks each wire and its reverse, and exercises both Wire and WireAPIMake.
The 24 rows encode `face_index reversed api_make found DONE1`.

The Rust test compares every row exactly. This validates the wrapper's wire
construction and status behavior, not all underlying face classification cases.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_outer_bound
```

Use the native compile command in
[the singular-edge fixture](tkshhealing_wire_singular_checks.md), substitute
`tkshhealing_wire_outer_bound.cxx`, and add `-lTKPrim -lTKTopAlgo` before `-lTKBRep`.
