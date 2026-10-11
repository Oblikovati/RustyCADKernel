# Martensite point clouds

Martensite implements the `AIS_PointCloud` and `AIS_PointCloudOwner` contracts
from pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. The concrete types
embed public base storage, and the source traits preserve application overrides
for geometry and inherited owner callbacks.

```rust
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfPoints;
use rustycad_tkv3d::ais::{AisPointCloud, AisPointCloudSource};
let points = Handle::new(Graphic3dArrayOfPoints::new(2, 0)?);
points.add_vertex_f64(0., 0., 0.)?;
points.add_vertex_f64(2., 0., 0.)?;
let cloud = AisPointCloud::new()?;
cloud.set_points(&points)?;
```

`set_points` aliases its buffer. Bounds are cached from used float-buffer vertices
on each assignment; subsequent buffer edits change geometry without updating the
cached bounds. `set_points_arrays` copies coordinates and optional colors/normals
from independently bounded `HArray1` handles, matching elements by array offset.
Null coordinates or mismatched attribute lengths silently clear geometry and bounds.
Empty coordinate arrays retain a nonnull empty point buffer, as native OCCT does.
Neither overload schedules redisplay or selection recomputation.

Display mode 0 submits the live point array with the drawer's shading aspect.
Mode 2 submits the virtual bounding box through `Prs3dBndBox`; unsupported modes
produce no geometry. The inherited display-mode acceptance hook stays permissive.
Point-cloud constructors initialize point markers, display mode 0, highlight mode
2, and a cyan plus marker for dynamic subset highlighting. Derived clouds embed
`AisPointCloudBase` and reuse `ais_point_cloud_compute` and
`ais_point_cloud_compute_selection`, which call the virtual geometry providers.
Base setters remain callable from virtual overrides, so applications can extend
buffer assignment while delegating the native cache/copy behavior.

Selection modes 0/1 use `Select3DSensitivePrimitiveArray`, sensitivity 8, element
detection, identity initial location and eager BVH construction. Mode 1 enables
an element map and a priority-five forced subset owner; mode 0 uses a priority-zero
whole-object owner. More than 500,000 attribute elements select eight BVH groups;
smaller arrays select one. Mode 2 uses the virtual box without asking for points.
Point modes fall back to the virtual box when the provider has no point attributes.
Selection computation appends entities rather than clearing an existing selection.

Subset owners retain separate shared zero-based selected and detected maps. Each
highlight snapshots matching sensitive entities' detection maps and scalar detected
indices. Native traversal replaces the map for each matching selection; it does
not accumulate every selection. Only subset positions are copied into the highlight
buffer. Explicit style layers win; otherwise dynamic highlighting uses Top and
static highlighting uses the cloud's layer. Unhighlight erases the existing selection
presentation. Clear preserves presentations and maps through the base owner's no-op.
`AisPointCloudOwnerBase`, `AisPointCloudOwnerSource`, and the shared highlight body
support derived owners without duplicating this traversal.

Color/material setters preserve native ownership, linked drawer precedence,
front/back facing policy, transparency restoration and live aspect synchronization.
Per-point colors remain buffer attributes; setting the cloud color does not rewrite
them. Native material naming can become UserDefined when material colors change.

## Verification

Run the CPU tests with `cargo test -p rustycad-tkv3d`. The existing numeric oracle
includes 77 independent point-cloud cases: array offsets and attributes, invalid
and empty arrays, alias mutations and stale bounds, all selection modes, appearance
ownership transitions and actual ray-driven owner highlight snapshots. Native and
Rust finite values, signed zeros, exception types and output tokens compare exactly.
Direct tests also exercise derived geometry providers and derived subset owners.

The renderer harness includes the `point-cloud` family in its full corpus. Sixteen
modes cover ordinary/color/normal buffers, box display, whole-object selection,
selected and detected subsets, mutation, erase/redisplay and color restoration.
Six camera/window/translation configurations run on each of lavapipe and the native
AMD host. Capture role checks reject blank geometry, stale erased geometry or
missing subset overlays. Raw RGBA/depth comparisons keep their strict gates.
The `ais-point-cloud-updates` workload includes rebuilding 1,024 colored points,
recomputing active selection and production redraw/readback, with 20 warmup frames
and 100 samples. It is part of the complete performance workload set.

```sh
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full --case-family point-cloud \
  --profiles lavapipe host --output target/point-cloud-renderer.json \
  --captures target/point-cloud-captures
```

See the [runtime evidence](../porting/runtime-evidence/ais-point-cloud-2026-10-11/README.md)
for final results and retained failures. Renderer parity remains incomplete: strict
point/line raster and depth differences remain visible. These fixtures explicitly
use Unlit shading for normal-bearing buffers. [Gouraud/Phong point-normal lighting](vulkan-lit-markers.md) now reaches Vulkan.
Translucent points, PBR markers and other unsupported point rendering policies
remain explicit renderer errors. No tolerance or scope was relaxed.
