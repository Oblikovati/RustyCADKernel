# Martensite lit-line runtime evidence

Normal-bearing segments and polylines now use retained Gouraud/Phong material,
light and normal snapshots. Facet and normal-free line geometry use the native
unlit fallback. Normal-bearing lit lines preserve solid coverage when a pattern
is requested, matching the pinned OCCT shader implementation.

The initial probe incorrectly applied the unlit stipple mask to lit lines. Native
patterned/solid captures were byte-identical on both devices, while the initial
Vulkan patterned capture differed. The initial probe receipt preserves that
failure. The solid-policy, repeat and final receipts retain the correction and
all strict raster failures. Earlier diagnostic receipts precede source refactoring;
only the final receipt's source fingerprints are verified against the integrated
renderer sources. Its revision field names the parent integration commit; hashes
identify the tested worktree changes.

The final matrix completes 60 strict captures per device. Lavapipe passes 42;
AMD passes 56. Remaining failures concern endpoint coverage and small color
interpolation differences. No pixel relocation or threshold relaxation is used.
Both profiles pass 20 existing runtime checks and 21 analytic lighting scenes,
including six line scenes, with zero Vulkan validation errors.

All 34 release workloads run with 20 warmup frames and 100 samples per device.
The final repeat passes every existing same-device/driver median and p95 budget
(1.25 and 1.5 ratios). Raw distributions remain in the receipts and the comparison
is in `performance-repeat.json`. Strict captures still fail, so these timings
are not an accepted overall renderer baseline.

CPU all-target tests, all-feature Clippy with warnings denied, bounded formatting
and all 21 Vulkan tooling test modules pass. The Rust tests preserve live normals,
Gouraud/Phong selection, facet fallback and the public segment lighting contract.
The final shader check recompiles every Vulkan 1.3 variant and requires exact
agreement with the checked-in SPIR-V bytes.

`captures.tar.gz` contains final native/Vulkan RGBA and depth buffers, PNG views
and difference masks. Reproduce the local matrix with the command in
[the implementation notes](../../../docs/vulkan-lit-lines.md). The harness
intentionally exits with failure while strict raster comparisons remain open.
