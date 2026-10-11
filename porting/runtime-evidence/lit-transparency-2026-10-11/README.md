# Martensite translucent lighting evidence

Ordinary retained triangles now render translucent front and distinguished back
materials with PhongFacet, Gouraud and Phong. Explicit Blend/Opaque policies,
live material opacity, automatic pass transitions, clipping, vertex alpha and
opaque-before-transparent depth behavior are covered by Rust regressions and GPU
assertions. Lit nodal geometry and caps retain their opaque-material guard.

The final receipt binds the tested source fingerprints to the integrated feature.
Its revision field identifies the parent checkout commit; the fingerprints identify
the worktree changes. Earlier receipts retain the first native GPU analytic failure
and the probe before function decomposition. The first assertion expected one
sRGB encoding of linear green 0.5. Lavapipe encodes 187 and AMD encodes 188;
that analytic channel now permits both. Other asserted channels and depths remain
exact. Native RGBA/depth capture thresholds are unchanged.

Both profiles pass 20 existing runtime checks and 42 analytic lighting scenes,
covering 172,032 color/depth pixels per device, with zero Vulkan validation errors.
The CPU all-target suite passes 240 tests. All-feature Clippy with warnings denied,
formatting and all 65 Vulkan tooling tests pass.

| Device | Exact captures | Open strict failures |
| --- | ---: | ---: |
| lavapipe, llvmpipe LLVM 20.1.2 | 76/78 | 2 |
| AMD Radeon 8060S, RADV STRIX_HALO | 35/78 | 43 |

The two lavapipe failures each change one color pixel. AMD includes 40 cases with
one to three changed color pixels and three orthographic facet cases with 186,
316 or 418 changed color pixels. Their precise causes are not established by this
receipt. Every failed case has zero depth difference. These are retained as open
rendering parity evidence without further pixel tuning or threshold relaxation.

All 35 release performance workloads run with 20 warmup frames and 100 samples
on each device. The new `lit-surface-opacity-updates` workload alternates material
opacity while updating a normal/light through V3d redraw and full GPU readback.
The final matrix has median/p95 timings of 0.446/0.510 ms on lavapipe and
0.340/0.493 ms on AMD for this workload. All 34 existing workloads pass the previous
integrated lit-line receipt's same-device/driver median 1.25 and p95 1.5 budgets,
as recorded in `performance-existing.json`.

The independent timing repeat passes all AMD workloads but exceeds the lavapipe
median budget for AIS plane wire and shaded updates. An earlier consecutive
comparison also exceeded window resize and a marginal single-triangle p95 budget.
All distributions and failures are preserved. This does not establish a sustained
regression or prove its absence. The overall renderer baseline remains unaccepted
while strict capture and timing stability gates remain open.

`captures.tar.gz` contains final native/Vulkan raw RGBA/depth buffers, PNG views
and separate comparison masks. Native verification remains local during porting;
CI runs Rust functional/analytic tests and release performance workloads. Reproduce
with the command in [the implementation notes](../../../docs/vulkan-lit-transparency.md).
The local harness deliberately exits with failure while exact parity remains open.
