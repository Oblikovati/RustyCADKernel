# Martensite Blend/XOR text local evidence

This bounded renderer feature implements actual Vulkan logic-op XOR glyphs and an
owned explicit native Set2D caller seam. It does not complete the broader legacy
OpenGl_Text class, does not claim retained AddText equivalence for Set2D, and adds
no OCCT API ledger credit. OCCT is a local generation oracle only: committed
CPU traces and framebuffer assets are replayed by Rust without OCCT dependencies.

The pinned upstream, actual native probe sources/libraries/font, ten executable
hashes and committed asset hashes are recorded in `provenance.json`. The optimized
source/executable freeze is `performance-source-manifest.json`. Native GL reports
AMD Radeon 8060S/radeonsi; Vulkan uses RADV STRIX_HALO and lavapipe, enumeration
index zero with explicit ICD selection. All functional Vulkan probes enable
validation. No Windows/macOS validation is claimed.

## Functional results

The final 72-scene matrix ran native, repeated native, lavapipe and AMD serially:
288 commands exited zero. Native raw RGBA/depth/traces repeated byte-for-byte.
`final-captures.json` retains every comparison. The strict gate is unchanged:
exact RGBA and maximum depth error 1e-6. Retained scenes pass 36/39 lavapipe and
34/39 AMD; explicit pixel scenes pass 31/33 on each. The remaining rotated and
transformed-plane failures are recorded, not accepted or threshold-adjusted.

Final Rust-only `blend_native_replay` independently replays all 72 committed
native scenes per device: 67/72 lavapipe and 65/72 AMD, with exit101 and raw logs
preserved for every strict failure in `rust-replay-results.json`. The ordinary
`blend_native_functional` gate checks 13 exact stable native buffers on each
device, including actual colored/alpha/highlight/cancellation/overlap and native
same-element Reset/no-Reset cache behavior; both exit zero. `blend_functional`
checks exact color/depth controls for XOR cancellation, ordinary unlit/lit/atlas
submissions afterward, ordered layers, depth-write override and capping stencil
boundaries; both devices exit zero. The actual new sampled VkImage surface path
is absent from this branch base; its post-merge integration recipe is explicit
in `sampled-surface-integration-recipe.md`, and is not represented as completed.

Final Rust all-target checks pass 329 tests across 49 executables. All-target
Clippy with warnings denied, global fmt, canonical toolkit discovery and the
13 discovery Python regressions pass. Raw checks/build logs are retained.
Existing Shadow/Subtitle/Decal tests, fixtures and strict residuals remain intact.

## Bounded conversion diagnosis

The actual native live-color probe repeats all 256 raw rows byte-for-byte.
Canonical sRGB encoding differs in 24 rows by one byte. Actual Vulkan XOR matches
232/256 lavapipe and 211/256 AMD against the native AMD oracle; every mismatch is
one byte. Ordinary Vulkan sRGB glyph conversion matches 256/256 AMD and 206/256
lavapipe against that same AMD native oracle, isolating attachment/source
quantization. `color-transfer-comparison.json` and raw output preserve all rows.
No oracle-derived production lookup, raster threshold change or baseline
acceptance is used. Investigation stops at this bounded seam diagnosis.

## Performance contract

Optimized measurements use the frozen executable directly: 72 workloads per
device, 20 complete redraw/readback warmups and 100 raw complete-operation samples
per workload, validation false. Functional validation is separate. Source and
binary identities, process/load snapshots, raw samples and terminal statuses are
recorded in `performance-results.json` and both `performance-*.log` files. Both
profiles exited zero with all 72 unique workloads and exactly 100 samples each;
all 298 frozen source hashes and the executable hash stayed unchanged. These
distributions establish no accepted
performance baseline. Shared-host observations bound any isolation claim.

## Reproduction

Native generators are `tools/oracle/martensite_vulkan_decal_capture.cxx` with
family `blend` (31 retained modes), `martensite_vulkan_blend_capture.cxx`
(eight extra modes), `martensite_vulkan_pixel_text_capture.cxx` (33 modes) and
`martensite_vulkan_xor_color_transfer.cxx` (256 live colors). Each is compiled
with C++17 against the pinned local OCCT include/lib directories and linked to
TKOpenGl/TKV3d/TKService/TKMath/TKernel. Captures use DISPLAY=:1 and actual native
OpenGl_Text; probes query current driver/FBO state rather than duplicate policy.
`native-repeat-hashes.json` records all 216 first/repeated raw artifacts.

```sh
cargo test -p martensite-vulkan --all-targets
cargo clippy -p martensite-vulkan --all-targets -- -D warnings
cargo fmt --all -- --check
python3 tools/toolkit_test_suites.py
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json MARTENSITE_VULKAN_DEVICE=0 \
  target/debug/examples/blend_native_functional
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json MARTENSITE_VULKAN_DEVICE=0 \
  target/debug/examples/blend_native_replay retained decal-rotated
```

The last command intentionally returns exit101 for the unchanged strict residual.
Run both `retained` and `pixel` entries from `final-captures.json` to reproduce
the complete diagnostic matrix. `rust-replay-results.json` keeps each command's
family/mode/profile/exit and raw output path.
