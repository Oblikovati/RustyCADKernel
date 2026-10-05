# Matrix scalar operator paths

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` uses element
`*=` and `/=` for Mat3/Mat4 scalar multiplication and division. Value-returning
operations copy the matrix and invoke the same assignment path. There is no
floating-point reciprocal specialization here.

The probe supplies a trivially copyable tagged element with assignment operators
and no binary multiplication/division operators. Each of the twelve output rows
checks every element. Rust tests replay those values, cover i64 matrices, and use
elements without the broader MatrixScalar trait. Generic scalar operators replace
the previous f32/f64-only implementations.

Compile from the worktree with GCC 13.3:

```sh
g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade tools/oracle/tkernel_matrix_scalar_paths.cxx -o /tmp/tkernel-matrix-scalar-paths
/tmp/tkernel-matrix-scalar-paths > /tmp/tkernel-matrix-scalar-paths.actual
diff -u tools/oracle/tkernel_matrix_scalar_paths.expected /tmp/tkernel-matrix-scalar-paths.actual
```

The broader MatrixScalar trait now includes MulAssign and DivAssign, as required
by native matrix algorithms that call these operations. No external dependency is
added. The fixture covers defined, nonthrowing arithmetic; C++ signed overflow,
division by zero, noexcept termination, and non-Copy element lifecycle semantics
are not established by this receipt. Supplemental cases add no legacy declarations.
