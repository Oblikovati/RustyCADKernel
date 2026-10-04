# TKG3d `GeomGridEval_Curve` protected initialization

The derived probe exposes the two protected overloads without changing OCCT. Its replay records each evaluator kind and one evaluated point. The failure case confirms that native `GetType` throws before replacing the prior evaluator.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

- Header SHA-256: `279d9f6ef2c25b11ae3197db302ff2d6bceae3a950218d59f8e499619ec5d8e0`
- Implementation SHA-256: `06230b2774f40018e34812ee3042b130a82923a5a6b1cb571fd25697bb78a011`
- Probe and library hashes, full output, and limitations: `porting/audits/tkg3d-grid-eval-curve-init.json`.

Build and replay against the pinned Linux OCCT installation:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkg3d_grid_eval_curve_reinitialization.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkg3d-grid-eval-curve-reinitialization
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib:/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/tkg3d-grid-eval-curve-reinitialization
```

The cases cover line → circle → trimmed-circle replacement, null clearing, the concrete `GeomAdaptor_Curve` path ignoring its range, a borrowed `Adaptor3d_IsoCurve` fallback, and a `GetType` exception preserving the prior fallback. Rust's lifetime ties the fallback adaptor to the evaluator. Rust does not reproduce C++ subclass layout or raw-reference ABI; these protected-hook mappings remain partial.
