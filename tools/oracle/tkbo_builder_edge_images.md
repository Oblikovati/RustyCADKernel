# BOPAlgo_Builder edge image stage

Oracle source is pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exposes protected `FillImagesEdges` over a prepared BOPDS with ordinary,
referenced-empty, unreferenced, and common-block edge records. It verifies that
referenced edges bind an Images entry even when their pave-block list is empty,
while an edge without a reference is omitted. Common pave blocks resolve through
`RealPaveBlock`, append each original to the real image's Origins list, and bind
same-domain image edges to the representative.

Replay from the configured OCCT modeling build:

```sh
g++ -std=c++17 -O0 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbo_builder_edge_images.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_builder_edge_images
/tmp/tkbo_builder_edge_images
```

The stage has no cancellation/progress API yet. This is a Builder preparation
slice, not complete Boolean result construction, and does not map a Fuse test.
