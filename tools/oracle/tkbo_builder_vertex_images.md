# BOPAlgo_Builder vertex image stage

Oracle source is pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exposes protected `FillImagesVertices` on a DS with two direct
same-domain links, then reads the public Images, Origins and ShapesSD maps.
It confirms FillImagesVertices records direct links (0→1 and 1→2), rather
than resolving the chain transitively, and appends reverse origins for each
representative.

Replay from the configured OCCT modeling build:

```sh
g++ -std=c++17 -O0 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbo_builder_vertex_images.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_builder_vertex_images
/tmp/tkbo_builder_vertex_images
```

This ports only the first Builder image-building stage. FillImagesEdges,
container/face/solid image construction, BuildResult, history and Boolean
execution remain incomplete, so it maps no legacy Fuse test declaration.
