# Martensite concrete ViewCube capture

The local fixture now displays the actual `AisViewCube` through a Vulkan viewer
and AIS context, with presentation mode 0 and real selection mode 0. It replaces
the earlier presentation-only adapter. Detection at pixel (100,220) finds one of
26 actual part owners; `select_detected(Replace)` consumes that owner click,
leaves global selection empty and starts the camera animation.

Ordinary geometry preserves native Topmost and LEFT_LOWER triedron persistence
with offset (100,100). A 320×320 orthographic camera starts at (10,8,20), observes
the origin with Y-up, scale 20 and z 1..100, and uses a blue background. Nine cases
capture default, unlabeled, lit, lit-unlabeled, hover, animation quarter/half/end
and a live appearance mutation. The hover case explicitly assigns the dynamic
highlight drawer to TopOSD, exposing real cyan part geometry above the ordinary
cube; this is an intentional supported style configuration. Native default
Topmost highlighting is covered by the ordinary cube in this fixture.

The injected wall clock drives the existing camera timer at 0.125, 0.25 and
0.5 seconds. Quarter/half remain active, and the final step stops and detaches the
animation. Appearance changes call real size, rounding, radius, font-height,
color and label setters before normal context redisplay. Native OpenGL,
lavapipe and AMD all produce 4,149 exact cyan pixels; appearance changes 13,311
native pixels and 13,312 pixels on each Vulkan device. Both Vulkan captures
finish with no validation errors. Named recording-device and fixed-font tests
verify submitted geometry, cyan draws, cached selection and camera progress.

```sh
cargo test --locked -p martensite-vulkan --all-targets
cargo build --locked -p martensite-vulkan --example view_cube_capture
DISPLAY=:1 VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 target/debug/examples/view_cube_capture /tmp/view-cube
```

Repeat with `radeon_icd.json` for the AMD host. Each case writes complete
`.rgba`/`.depth` pairs. The native generator uses real `AIS_ViewCube` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`; it is a local oracle only. Committed
buffers and PNG previews permit inspection and comparison without OCCT.

The existing identical-RGBA and maximum depth-difference 1e-6 gates remain strict.
The fixture renders successfully but fails those comparisons. No pixel-row
precision tuning, threshold change or accepted historical baseline accompanies
this work. Initial static captures retain their previous small depth failures.
The initial animation endpoint has a maximum depth difference about 0.79 because
native covers a pixel where Vulkan leaves the clear depth 1.0. Large differences
are confined to narrow silhouette columns, with a one-pixel horizontal boundary
shift at the exact front orientation.

Complete native/ported source, retained start/end and quarter/half/end camera
states match, including Eye/Center/Up/Direction, scale, FOVy/FOV2d, aspect,
projection and clipping planes. Primary scene bounds are void in both systems,
so fitting leaves scale 20; auxiliary bounds agree. AutoZFit is disabled.
Native dumps leave invalidation set, while Vulkan clears rendered invalidation;
this does not change the sampled cameras. The bounded trace identifies coverage
precision rather than an animation or FitAll state discrepancy. Before-failure
buffers, strict failures and diagnostic masks remain committed. GPU parity and
whole-class completion remain unclaimed.

The optimized `view_cube_performance` executable records completed redraw and
readback for seven workloads: default, lit, hover, quarter, half, end and live
appearance. Each has 20 warmups and 100 samples; validation is disabled only for
those timings. Run under the coordinating agent's exclusive timing slot:

```sh
cargo build --locked --release -p martensite-vulkan --example view_cube_performance
DISPLAY=:1 VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 target/release/examples/view_cube_performance
```

[Current evidence](../porting/runtime-evidence/view-cube-widget-appearance-2026-10-12/README.md)
records source/binary hashes, native CPU provenance, compressed raw captures,
PNG previews, camera/bounds logs and strict comparison results. The older
[presentation-only evidence](../porting/runtime-evidence/view-cube-capture-2026-10-11/receipt.json)
remains historical provenance; it does not describe the current concrete widget.

The final exclusive run measured these completed-frame median/p95 milliseconds.
These are new workload distributions, with no accepted historical baseline:

| Workload | lavapipe | AMD host |
| --- | --- | --- |
| Default | 1.730581 / 2.130921 | 1.602418 / 1.941541 |
| Lit | 1.721268 / 2.016660 | 1.532188 / 1.789630 |
| Hover | 1.716960 / 2.242577 | 1.572323 / 1.823402 |
| Quarter | 1.742634 / 2.175664 | 1.559149 / 1.844632 |
| Half | 1.659344 / 1.912578 | 1.584636 / 1.852406 |
| End | 1.616850 / 1.957650 | 1.533822 / 1.772037 |
| Appearance | 1.850558 / 2.241345 | 1.732519 / 2.013294 |

The verified final executable SHA remains
`aea6a3a7f6e2464935dd349bc11bca915087e76ff61bdc5c0e579e29bd75ef87`.
The retained final relink log records the successful TKV3d/Vulkan rebuild after
the ResetStyles subclass-dispatch correction. That generic reset is not invoked
by these workload paths, so the final linked executable retained its earlier
hash. The coordinating agent independently verified the final binary and all 49
source hashes before authorizing timings; no rebuild or source mutation occurred
during the exclusive window.

A later helper extraction enforces the complete 20-line function-span limit,
including signatures. The original captured buffers and aea6 timings above
remain earlier-source evidence. The final helper-refactored source passes 991
combined tests and fresh 76-row binding checks. Final exclusive timings for
binary `c9bdedd5f5d3b1f9e500ccdbec5e8eab762e378a24b9062bad28d26db2d51d8f`
are retained separately under `post-refactor-timing/`; all seven workloads on
each device completed 100 samples after 20 warmups. The coordinating agent's
final-main functional capture run will verify the extracted helpers on both
profiles. Original strict failures and gates remain unchanged.
