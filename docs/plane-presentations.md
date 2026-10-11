# Martensite plane presentation builders

Martensite ports `StdPrs_Plane`, `DsgPrs_XYZPlanePresentation` and
`DsgPrs_ShadedPlanePresentation` from OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`. These are prerequisites for
`AIS_Plane`; this change does not claim that interactive object is ported.

`StdPrsPlane::add` retains the current group before testing the surface kind.
Planes produce the native rectangular edge order, incrementally spaced isolines,
and normal arrows. Center shafts precede their arrowhead; corner arrowheads
precede the combined shaft array. The arrow aspect is applied even with both
arrow flags disabled. `StdPrsPlane::matches` uses inclusive distance to the infinite
plane, independently of display size and the drawer.

The datum builders submit the native four-vertex closed polyline or polygon.
The shaded builder retains the edge-to-shading aspect transition. Vulkan lowers
convex polygons to triangle fans per bound, retaining indexed input, winding,
bound colors and the closing degenerate triangle. Positive subpixel line widths
retain their requested aspect value and rasterize at the native one-pixel minimum,
including the default half-pixel plane isoline aspect. General concave or nonplanar
polygon parity is not established by the datum triangle fixture.

Nonpositive isoline capacity retains OCCT's empty submission behavior. Inputs
whose floating-point-to-integer conversion or capacity arithmetic would be
undefined in C++ return a descriptive Rust failure instead. No native result is
claimed for that undefined input domain.

Validation uses nine focused builder tests, two retained polygon tests and
236 exact native differential requests. The native corpus covers all sixteen
visibility combinations, translated tilted planes, negative extents,
nonplanes, sparse isolines, both datum builders and distance matching. Driver
callbacks that mutate aspect settings retain native geometry snapshots: plane
extents precede the edge callbacks, while arrow dimensions precede the arrow
aspect callback after edges and isolines have been submitted.

Rendering verification includes six paired plane modes on lavapipe and the host
GPU, with color/depth captures and the `plane-presentation-updates` performance
workload. The final fixture passes all 36 strict cases on lavapipe and 19/36 on
the native AMD GPU. All 72 RGBA captures match exactly; the remaining 17 native
GPU failures are depth precision deltas above the unchanged 1e-6 threshold.
Both closed datum builders pass every case on both profiles. Measured results
and integration receipts are recorded in
`porting/runtime-evidence/plane-presentations-2026-10-11`. The two final runs reproduce the same functional results. The plane-update
workload passes both unchanged median/p95 budgets on both profiles, with repeat
medians around 0.524 ms on lavapipe and 0.411 ms on the host. One existing
lavapipe workload and three existing host workloads exceed p95 limits;
those signed failures are retained, and no overall performance baseline is
accepted.

Strict gates remain unchanged; full legacy OpenGL replacement
proof remains incomplete.

```sh
cargo test -p rustycad-tkv3d --test plane_presentations --locked
cargo test -p martensite-vulkan --test polygon_primitives --locked
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full --output target/planes.json \
  --captures target/plane-captures --case-family plane-presentations \
  --profiles lavapipe host
```
