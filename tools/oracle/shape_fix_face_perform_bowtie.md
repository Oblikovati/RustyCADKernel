# ShapeFix_Face::Perform bow-tie no-op

The native performer leaves this planar bow-tie face unchanged: `Perform` returns false, no DONE status is set, and the result is the input face. This fixture records that behavior so a Rust-only earlier split shortcut cannot be mistaken for OCCT parity. Direct `FixLoopWire` reconstruction remains covered separately by `shape_fix_face_loop_wire`.

Pinned source: clean OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`. The oracle was compiled against the same CMake build used by `OpenCascadeGTest` (`target/occt-modeling-full`, `OCCT_SOURCE_DIR=/home/vmiguel/git/oblikovati-workspace/OCCT`). Build identity: `OpenCascadeGTest` SHA-256 `1abf9129562e16490ab9098162a9cf2c1aa40d72000328fb5bb9e6cf71cf3693`; `libTKShHealing.so.8.1` SHA-256 `9136732bc7e30f719f53c5e9244fdd39c6a59a19998d897caaf3052cb57692c7`. The pinned native regression `ShapeFix_ShapeTest.HealPrismFromSelfIntersectingFace` passes on this build; the Rust `shape_fix_shape_parity::heal_prism_from_self_intersecting_face` test also passes.

Run from the repository root:

```sh
g++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/shape_fix_face_perform_bowtie.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d \
  -lTKG2d -lTKMath -lTKernel -o /tmp/shape_fix_face_perform_bowtie
LD_LIBRARY_PATH=$PWD/target/occt-modeling-full/lin64/gcc/libi \
  /tmp/shape_fix_face_perform_bowtie
```
