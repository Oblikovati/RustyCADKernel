# Generic vector scalar multiplication

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Build this supplemental oracle against the pinned generated headers:

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_vector_generic_scalar.cxx -o /tmp/vector-scalar-oracle
/tmp/vector-scalar-oracle > /tmp/vector-scalar.actual
diff -u tools/oracle/tkernel_vector_generic_scalar.expected /tmp/vector-scalar.actual
cargo test -p rustycad-tkernel --locked
```

The first six output rows use `std::int64_t`; the next six use an element whose
`operator*=` deliberately adds one after multiplication, while `operator*` does
not. OCCT Vec2 scaled copies use `*`; Vec3/Vec4 scaled copies use `*=`. All three
in-place scalar operations use `*=`. Rust regression tests assert the same values.

Generic scalar and componentwise trait implementations coexist. This replaces the
previous primitive-only scalar multiplication implementations and corrects the
inherent in-place operation for compatible custom elements. Scalar division and
vector/vector arithmetic are outside this change.

The source-bound receipt is `porting/audits/vector-generic-scalar-multiply.json`.
It records full package validation and six compiler-observed Rust API selectors.
These supplemental tests add no original legacy-test declarations, and do not
establish equivalence for overflowing arithmetic or arbitrary C++ element types.
