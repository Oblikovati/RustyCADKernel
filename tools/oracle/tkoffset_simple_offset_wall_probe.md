# BRepOffset_MakeSimpleOffset wall-face oracle

The probe invokes the private `BuildWallFace` member through a test-only C++ private-member
access shim. It does not patch or rebuild the OCCT source. A `BRepTools_TrsfModification`
supplies the offset image edge; this isolates wall construction while retaining the native
wire assembly, planar face attempt, and GeomFill fallback. Cases cover four adjacent planar
walls, coincident edges, a nonplanar half-circle ruled wall, and a closed-circle seam wall.

The fixture records face surface type/bounds/tolerance/location, sampled surface geometry,
validity, vertex-edge-map extent, each wire edge's orientation and 3D/2D ranges, curve types,
3D endpoints, p-curve endpoints, tolerance, SameRange/SameParameter and seam status. The
seam case prints the second p-curve branch separately.

Build and run from the repository root with the pinned OCCT modeling build:

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkoffset_simple_offset_wall_probe.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKOffset -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_simple_offset_wall_probe
/tmp/tkoffset_simple_offset_wall_probe
```
