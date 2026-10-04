# Direct polynomial protected `Solve` state reuse

The probe derives from `math_DirectPolynomialRoots` to expose all four protected overloads. It records observable root flags, counts, and IEEE-754 bits after repeated calls. The fixture uses only defined calls: it does not inspect uninitialized native count storage or access roots while the infinite-solutions guard is set.

Pinned source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

- Header SHA-256: `357604688545f0a238ef78607e5d5614a8dab2ec25150a5373c9e4a8f3da2bfc`
- Implementation SHA-256: `cbc9650d3062a2ee76d7ad28f43731affe693cec7baf6686c4dc9f81e1ab72c5`
- Probe SHA-256: see repository revision for this file.
- Native executable SHA-256: recorded by the caller on rebuild; installation is the pinned OCCT build.

Rebuild and replay on the Linux OCCT install used for this fixture:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkmath_direct_roots_solve_reuse.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKMath -lTKernel -o /tmp/tkmath-direct-roots-solve-reuse
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib:/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/tkmath-direct-roots-solve-reuse
```

The native methods do not reset `myDone` or `myInfiniteStatus`. Degree reduction calls a lower-degree overload on the same object. A failed cubic/quartic solve sets `myDone=false` and leaves earlier count/root slots intact. Linear identity solving sets the infinite flag, and later finite solves do not clear it. The Rust `solve_linear`, `solve_quadratic`, `solve_cubic`, and `solve_quartic` methods preserve these state transitions.

This proves the four protected overloads for the exercised finite, no-real-root, overflow, and infinite-flag transitions. It does not make Rust methods protected: Rust exposes them as public stateful methods so downstream code can reproduce derived-class calls. Exceptional native states whose count was never initialized are deliberately excluded from parity assertions.
