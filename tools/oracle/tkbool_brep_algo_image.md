# `BRepAlgo_Image` core behavior

The probe records generator chains, `LastImage`, compaction, and OCCT's many-sources to
one-image behavior: each source keeps its direct image link while the image-to-generator
map retains the last generator bound.
The final row records the source's process-wide fallback list, which accumulates one
unbound shape on every `Image()` call.
It targets the repository's pinned OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

From the RustyCADKernel root, compile and compare with:

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbool_brep_algo_image.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKBool -lTKBO -lTKPrim -lTKBRep -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkbool-brep-algo-image
/tmp/tkbool-brep-algo-image | diff -u tools/oracle/tkbool_brep_algo_image.expected -
```
