# CSLib Class2d assignment return values

Pinned native source is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`CSLib_Class2d.hxx` declares copy and move assignment as returning
`CSLib_Class2d&`; the implementation deep-copies the source through its copy
constructor for copy assignment, while move assignment transfers the polygon
and a ready grid and disables the source. Both return `*this`.

The supplemental Rust API exposes `Class2d::assign_returning` and
`Class2d::move_from_returning`. Their `&mut Self` return identifies the
destination. Safe Rust borrowing does not express native self-copy or
self-move assignment, so these mappings remain partial for those alias cases.

## Native reproduction

The probe constructs a 32-vertex polygon, runs 64 classifications to complete
the lazy grid, then checks copy/move assignment return identity and source and
destination classifications. It prints `1 1 -1 1 1 0`.

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/include/opencascade \
  tools/oracle/cslib_class2d_assignment_returns.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi \
  -lTKMath -lTKernel \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi \
  -o /tmp/cslib_class2d_assignment_returns
/tmp/cslib_class2d_assignment_returns > /tmp/cslib_class2d_assignment_returns.out
diff -u tools/oracle/cslib_class2d_assignment_returns.expected \
  /tmp/cslib_class2d_assignment_returns.out
```

The probe uses `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` and links
`libTKMath.so.8.1.0` with SHA256
`10e1965fdd50486bf85d905493d84234878b828a0e2d06381ce6fedab7b40243`.
The checked-in native probe has SHA256
`e5f66701feabdcc376b0a80a713448d37277fa104ddeaad9697c0a48a35a666a`; its
captured output has SHA256
`e3c6ad3050c16a7d6f6e267987d88817392159259bb6b8463ec80c5030e3ea38`.
Native source hashes are `990f1b31b97b5d797c6ee6a767428a6f72dfd1f158ae4a03b2600194f71d5302`
for the header and `5dd2a866a9e5c6f7670b5382966dcefcea7d1f2dbdecabc38025edbbf955265f`
for the implementation. The linked `libTKMath.so.8.1.0` is from the pinned
foundation build.

## Rust coverage

```sh
cargo test --locked -p rustycad-tkmath --test class2d_assignment_returns \
  --test cslib_behavior
cargo test --locked -p rustycad-tkmath --test cslib
cargo clippy --locked -p rustycad-tkmath --test class2d_assignment_returns \
  --test cslib_behavior --test cslib -- -D warnings
cargo test --locked --doc -p rustycad-tkmath
```

With pinned Rust 1.98.1, incremental compilation and debug symbols disabled,
the runs pass 2 supplemental assignment tests, 12 behavior tests, 36 original
`cslib` integration tests, Clippy with warnings denied, and 35 doc tests.
Original legacy test names and counts are unchanged.
