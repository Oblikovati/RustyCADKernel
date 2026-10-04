# Declared but undefined `BRepOffset_MakeOffset::GetAnalyse`

The pinned OCCT source declares this public method as exported, but does not
define it anywhere under `src/ModelingAlgorithms/TKOffset`. The pinned native
`libTKOffset.so` also has no dynamic symbol for it. A program that calls the
declaration therefore fails to link; Rust must retain this as an explicit
`upstream_unimplemented` inventory obligation with `rust_item: null`.

Evidence is pinned to OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`:

- `BRepOffset_MakeOffset.hxx` SHA-256:
  `73df05be4d5fb4de9d482ef03832009bfe348201daa23c851e00ffa7090ada18`
- The only source match is the declaration at line 92. `rg -n GetAnalyse
  OCCT/src/ModelingAlgorithms/TKOffset` returns no definition.
- The pinned native library SHA-256 is
  `60feb372f24d5d83e63c0b835385eeb57dfa03d5ae88b26602927f2b5295f2c0`;
  `nm -D -C libTKOffset.so | grep 'BRepOffset_MakeOffset::GetAnalyse'`
  returns no symbol.
- `tkoffset_get_analyse_link_probe.cxx` includes and calls the declared
  method. Against the pinned modeling build, the linker reports:
  `undefined reference to BRepOffset_MakeOffset::GetAnalyse() const`.

Reproduce the link check with the pinned build paths:

```sh
g++ -std=c++17 \
  -I"$NATIVE_BUILD/include/opencascade" \
  -I"$OCCT_SOURCE/src/FoundationClasses/TKernel/NCollection" \
  tools/oracle/tkoffset_get_analyse_link_probe.cxx \
  -L"$NATIVE_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$NATIVE_BUILD/lin64/gcc/libi" \
  -lTKOffset -lTKFeat -lTKFillet -lTKBool -lTKBO -lTKPrim \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/get-analyse-probe
```

Set `NATIVE_BUILD` to the pinned `target/occt-modeling-full` directory and
`OCCT_SOURCE` to the checkout at the revision above. A successful link would
invalidate this record and should trigger a fresh behavior probe before any
Rust mapping is claimed.
