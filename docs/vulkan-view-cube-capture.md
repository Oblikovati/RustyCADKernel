# Martensite ViewCube presentation capture

This local fixture exercises the existing public ViewCube presentation adapter through an actual Vulkan viewer and AIS context. It is a presentation-only subclass, not a complete `AIS_ViewCube` widget. Selection requests return a typed unsupported error; display explicitly selects mode 0 and selection mode -1. Interaction, owner creation and camera animation are separate porting work.

The fixture preserves native default geometry, labels, Topmost layer, and LEFT_LOWER triedron persistence with a (100,100) pixel offset. A 320×320 orthographic view observes the cube from (10,8,20), with blue background and no initial active lights. The first color/depth capture uses all layers. Clearing only label strings through the live AIS context produces a second capture and must change visible pixels. An explicitly lit variant enables the viewer's native default light preset, restores label strings, and repeats the labels-on/off comparison. The lit variation changes viewer illumination, not cube defaults.

The local native oracle instantiates real `AIS_ViewCube` against OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`, explicitly disables selection activation, reports the actual OpenGL renderer and object layer, and follows the same camera and illumination changes. Committed raw captures support inspection without requiring OCCT. CPU tests use the named recording device and injected fixed glyph source to check real retained geometry, textured label submission, live removal/restoration, and unsupported selection.

Build and capture locally:

```sh
cargo test -p martensite-vulkan --all-targets
cargo build -p martensite-vulkan --example view_cube_capture
DISPLAY=:1 VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 target/debug/examples/view_cube_capture /tmp/view-cube
```

Repeat with `radeon_icd.json` for the native AMD host. The executable writes default, unlabeled, lit, and lit-unlabeled `.rgba`/`.depth` pairs and rejects Vulkan validation errors. The release `view_cube_performance` example measures completed redraw/readback for default and lit presentations, each with 20 warmups and 100 samples; validation is disabled only for those timings.

Strict comparisons retain the existing identical-RGBA and maximum depth difference 1e-6 gates. Initial default captures render successfully but fail strict depth parity: lavapipe has one differing color pixel and maximum depth difference 0.0017647; AMD has twelve differing color pixels and maximum depth difference 0.00244465. Removing labels produces exact color on both devices, while depth differences persist. These are unresolved diagnostics, not accepted cross-device parity. No pixel-row tuning, threshold changes, historical performance baseline acceptance or API-class completion claims accompany this fixture.

Final default and lit functional captures succeed on both devices with no validation errors. Both native and Rust label removal affect exactly 126 pixels in the unlit frame and 298 in the lit frame. The lit frame exposes readable oriented side labels. Strict comparisons still fail: lit frames differ at 46 color pixels on each device; lit unlabeled color is exact on lavapipe and differs at seven pixels on AMD. The depth failures remain, so label visibility and matching pixel counts do not imply complete parity.

The source-bound optimized run records these median/p95 milliseconds (20 warmups, 100 samples):

| Device | Default | Lit |
| --- | --- | --- |
| lavapipe | 1.714772 / 2.043720 | 1.739447 / 2.110504 |
| AMD host | 1.599008 / 1.896077 | 1.470285 / 1.675399 |

[Committed evidence](../porting/runtime-evidence/view-cube-capture-2026-10-11/receipt.json) retains raw native/Rust color and depth buffers, captures and diffs, device identities, source and binary hashes, raw samples and process/load snapshots. Committed native assets can be compared again without OCCT; the evidence README describes the existing comparison helper. The new workloads have no accepted historical baseline.
