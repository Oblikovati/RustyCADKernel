# Martensite rubber-band overlays

Martensite ports the pinned OCCT `AIS_RubberBand` rectangle and polygon overlays.
`AisRubberBand::new`, `with_line` and `with_filling` preserve the three native
constructors, their appearance defaults, lower-left 2D persistence and TopOSD layer.
The default object has an empty interior, white solid unit-width boundary,
Plastified material, Unlit shading and Blend alpha mode.

`AisRubberBandSource` and `AisRubberBandBase` expose the inherited interface and
protected screen-point sequence, cached triangles, cached border and closure flag
for application subclasses. `points()` borrows the live sequence storage; read it
with `points().read()` and release that guard before mutating the object. A retained
storage reference observes later edits. No point or appearance setter implicitly
schedules redisplay. Applications explicitly call the interactive context's
`redisplay` after edits, as in OCCT.

Rectangle coordinates retain their supplied order, including reversed bounds.
Removing the last point from an empty sequence raises `Standard_OutOfRange`.
The filling-color overload enables filling before validating transparency and
assigns color only after transparency succeeds. Selection computation is empty
and leaves a supplied selection unchanged. Unsupported display modes leave the
presentation unchanged; supported computation appends native groups.

Filling uses Martensite's existing frontier-constrained `BRepMesh_Delaun`, retaining
native domain traversal order. Empty/degenerate filling leaves the previous triangle
handle intact. Triangle and border arrays are reused when their native vertex counts
match. Triangle reuse changes positions without replacing existing normals. Border
closure appends the first point and preserves native failure for an empty closed
polygon. Explicit destruction clears points and releases the cached handles before
the interactive base is destroyed.

The exact numeric corpus includes all constructors, invalid widths/transparency,
rectangle reversal, appending/removing points, degeneracy, cached array identity,
filled/open boundary transitions and unsupported modes. Run it with:

```sh
python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full
cargo test -p rustycad-tkv3d --tests --locked
```

The native/Vulkan capture family covers ordinary/open rectangles, opaque/translucent
filling, concave polygons, dashed/wide boundaries and rectangle updates in both
camera projections and three viewport shapes. Blank output and missing filling
fail independently of the strict image/depth comparison. The release performance
workload `ais-rubber-band-updates` measures triangulation, cache changes, interactive
redisplay and GPU readback together. Both lavapipe and native host profiles are required:

```sh
DISPLAY=:1 python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/rubber-band-vulkan.json --captures target/rubber-band-captures \
  --case-family rubber-band --profiles lavapipe host --samples 100 --warmup 20
```

Strict renderer raster/depth parity and an accepted performance baseline remain
separate open gates. Ported API mappings do not claim overall renderer completion.
