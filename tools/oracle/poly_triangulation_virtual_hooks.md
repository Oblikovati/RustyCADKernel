# `Poly_Triangulation` virtual-hook probe

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe derives `ExpandedTriangulation` and overrides protected
`computeBoundingBox`. It checks direct virtual dispatch, a base-qualified call,
`MinMax` cache selection, rotation with an accurate bound, base `Copy()` slicing,
and a `DetachedLoadDeferredData` factory returning a derived triangulation.

The output was generated with the installed OCCT headers/libraries whose header
SHA-256 is `885f46dcacaf7f3260c347f87ff9de1a4929362671229fcf0080bf8e89d8f802`;
the TKMath and TKernel library hashes are recorded here:

* `libTKMath.so.8.0.0`: `5f025e85993b07b460d483a161ca4c6e2a9052009974ad02ab6da1ea2381943a`
* `libTKernel.so.8.0.0`: `78221dc495f8316c2a28e4fd9202920efdd17e056549006646ffd81bd0ddcc87`

Reproduce with:

```sh
c++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/poly_triangulation_virtual_hooks.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKMath -lTKernel -o /tmp/poly-triangulation-virtual-hooks
/tmp/poly-triangulation-virtual-hooks
```

Rust uses an injected `TriangulationHooks` strategy because this port does not
reproduce C++ inheritance or RTTI. The `DeferredTriangulation` trait name stays
as a public alias. A custom detached factory must return a `Triangulation`
configured with the desired hooks; `Copy()` intentionally returns a base object,
matching the native copy constructor. This is functional override dispatch, not
ABI/layout parity for arbitrary C++ subclasses.
