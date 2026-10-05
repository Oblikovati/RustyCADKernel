# math_VectorBase compiler-selector mapping probe

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Pinned `math_VectorBase.hxx` SHA-256:
`d68820c34a6cb97d0eeec2639b9c1948c9e0934ddc9cba5d88b22256c0f68369`.
Pinned `math_VectorBase.lxx` SHA-256:
`d14b14d7cab257576b442bc9612e2bb3c0781c2efa0ff0dcb0cb129a808ad46c`.
This probe invokes the exact template methods whose selectors were corrected
in the parity ledger. It exercises differing vector bounds, copy initialization
and assignment, in-place and returned arithmetic, range assignment, two-input
sum/difference, all four matrix/vector products, and dot product.

Reproduce against the pinned local OCCT build:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_vectorbase_unclaimed.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath-vectorbase-unclaimed
/tmp/tkmath-vectorbase-unclaimed > /tmp/tkmath-vectorbase-unclaimed.out
cmp /tmp/tkmath-vectorbase-unclaimed.out tools/oracle/tkmath_vectorbase_unclaimed.expected
```

Compiler used for the recorded fixture: `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1)`.
The Rust regression checks the same operand values and checks output lower and
upper bounds for each native output vector. The selector audit records the
unique compiler declaration ID and reachable Rust source hash for each of the
14 corrected ledger rows. This evidence is bounded to the exercised double
template instantiation and does not claim complete `math_VectorBase` behavior.
