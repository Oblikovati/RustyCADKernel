# Unlit side-color V2 evidence

Implementation: `9283411d0`, based on `eb63c6843`; owned V1 fixture pinned first in
`df93fc40d`. Decision and migration task: [ADR 0021](../../adr/0021-unlit-side-colors.md).
Final checked source: `653c731c022eb2c50e37e13b14ff6a494bf8b8e9`.
This is bounded Vulkan kernel evidence, not application acceptance or default migration.

## Executed checks

Linux x86_64, kernel 7.1.5-76070105-generic, Rust 1.99.0. Commands used `--locked`.
All driver creation in the new GPU tests requests Khronos validation. ICDs were
explicitly selected with `VK_DRIVER_FILES`:

| Profile | ICD | Selected implementation |
| --- | --- | --- |
| Native | `/usr/share/vulkan/icd.d/radeon_icd.json` | AMD Radeon 8060S Graphics (RADV STRIX_HALO), vendor 4098, device 5510, driver ID 3 |
| Software | `/usr/share/vulkan/icd.d/lvp_icd.json` | llvmpipe (LLVM 20.1.2, 256 bits), vendor 65541, device 0, driver ID 13 |

Both report driver version 109056006 and Mesa
`26.1.6-1pop0~1787580452~24.04~a5619ea`. Software driver info also reports LLVM
20.1.2. The compressed focused logs retain the exact driver identities.

| Check | Result |
| --- | --- |
| `cargo test -p martensite-vulkan --lib unlit_ --locked` | 18 passed |
| `cargo test -p martensite-vulkan --test toolkit unlit_face_color_baseline --locked` | 1 passed; eight exact V1 outcomes and successful repair |
| `cargo test -p martensite-vulkan --test toolkit unlit_face_color_retained --locked` | 3 passed |
| `cargo clippy -p martensite-vulkan --all-targets --locked -- -D warnings` | Passed |
| `cargo test -p martensite-vulkan --test toolkit unlit_face_color_gpu --locked -- --test-threads=1 --nocapture` | 7 passed on each profile, twice in separate processes |
| `cargo test -p martensite-vulkan --all-targets --locked -- --test-threads=1` with pinned validation | Each profile: 789 passed, 0 failed, 5 existing ignored across 77 executables |
| Historical `cargo test -p martensite-vulkan --lib --test toolkit --locked -- --test-threads=1` with inherited layer paths | Each profile: 293 library tests passed; toolkit 323 passed, 2 failed, 5 ignored; preserved below |
| Current `surface.frag` compilation and `spirv-val --target-env vulkan1.3` | All 192 surface fragment variants match checked-in SPIR-V byte for byte; only 32 untextured unlit outputs changed |

Shader compiler: glslang 11:15.1.0; validator: SPIRV-Tools v2025.1, build date
2026-05-13. The compressed shader provenance lists every output, definition set,
and SHA-256. Surface source SHA-256:
`105e051fe68c15f8dfd2f1ba35373d834223861441c7fe6085ad139862b53078`.
The owned V1 fixture SHA-256 remains
`00c99f1202402cfc0fda6300bdd260fccefa72124555b40d855a8d909714f27b`.
Every pre-existing fixture file is unchanged.

## Independent expectations

The CPU table covers all eight combinations of array presence, distinction and
back-vertex-color permission. It checks selected owned RGBA, visible-side alpha
routing, explicit alpha modes, source mutation, failure/reuse, cap and lighting
replacement, marker/glyph/texture exclusions, polygon-edge isolation, auxiliary
packing and pipeline cache identity. Only the new public policy type has a
compile-time `Send + Sync` assertion; no broader concurrency claim is made.

GPU expectations use opposite winding triangles with primary colors. The front
and back sample pixels must equal the independently selected colors exactly.
Back green alpha 0.25 over blue gives sRGB `[0, 137, 225]`; strict masking retains
alpha 0.25 at cutoff 0.25 and discards it at 0.2501. Existing attachment writes
are RGB-only, so capture alpha remains 255. The initial test incorrectly expected
64 for opaque/masked capture alpha; this expectation was corrected after checking
`backend/pipeline_state.rs`'s existing RGB write mask, without changing production
alpha behavior or any old fixture.

Further checks cover clipping, mirrored side identity, cyan mesh-edge composition
before the mask, weighted OIT, depth peeling, and byte-identical color/depth for
a V1-supported scene rendered through both policy constructors. All asserted
pixel errors are zero; tests do not widen tolerances.

Each final focused process records 20 complete raw color/depth captures as JSON,
as well as FNV-1a digests with an independent known-vector test. Python compared
the ordered raw byte strings between the two processes on each driver: all 20
matched byte for byte (32,768 bytes per capture: RGBA plus native-endian f32 depth).
The compressed focused logs preserve those bytes; `pinned-repeat-verdicts.json.gz`
records comparison results and SHA-256. This does not claim byte equality between
different drivers, or concurrency stress/ThreadSanitizer coverage.

## Validation-layer comparison and historical failures

The original inherited-environment full runs failed these existing tests:

- `oit_transparency::actual_colored_glyph_and_live_cap_inputs_replay_native_buffers_without_occt`
- `oit_transparency::actual_same_view_sampling_and_persistent_peeling_fallback_replay_native_assets`

They reported `SYNC-HAZARD-READ-AFTER-WRITE` from `vkCmdEndRendering` depth/stencil
`VK_RESOLVE_MODE_SAMPLE_ZERO_BIT` resolve. The original failures remain unchanged
in `full-host.log.gz` and `full-lavapipe.log.gz`. Their runner inherited layer search
paths; no historical loader trace was captured, so the loaded binary is not proven.

The coordinator reproduced both failures on both drivers using the same main
binary with system validation 1.3.275, while main's full suites passed with pinned
1.4.313. The comparison identifies the upstream depth/stencil resolve-ordering
correction and records actual layer library hashes. See main commit `1f96497cb`,
[the baseline comparison](../native-display-lifecycle-v1-2026-10-14/integrated-layer-comparison.md)
and its `integrated-raw.tar.gz` and `integrated-main.json` artifacts.

This candidate then passed both full suites and both repeated focused runs per
driver with these explicit settings (`ROOT` is the main checkout at
`/home/vmiguel/git/oblikovati-workspace/RustyCADKernel`):

```text
VK_LAYER_PATH=ROOT/target/vulkan-validation/extracted/usr/share/vulkan/explicit_layer.d
LD_LIBRARY_PATH=ROOT/target/vulkan-validation/extracted/usr/lib/x86_64-linux-gnu
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json
# Software replay instead uses /usr/share/vulkan/icd.d/lvp_icd.json.
```

The pinned manifest reports API 1.4.313; its selected library file SHA-256 is
`89e618526d6d1532096dbbd62ee1dc710dda647061cd3b7b66b542dc7dbce104`.
System 1.3.275 library SHA-256 is
`1d486283e4ceb342ef7931f97782f03b95a9fad379da638537ea24527780a287`.
`pinned-receipts.json` records exact paths, manifest hash, executable hashes and
log hashes. Provenance is explicit search paths and file hashes, not a runtime
loader trace. No synchronization code or validation settings were changed to
suppress failures. The bounded V2 functional and full Vulkan corpus gates pass
under the pinned validation environment; default activation and the recorded
migration/deletion task remain open.

The GPU lease was explicitly released after these runs. An earlier mistaken
CPU-only classification of `--lib` ran existing resource tests before the lease;
that run is not acceptance evidence. The coordinating agent confirmed its GPU
sequence had not begun, and all GPU claims above come from the later leased runs.

No live OCCT run, GL-string test mapping, Windows/macOS check, native-window
application acceptance, isolated performance benchmark, default-policy migration,
GitHub milestone completion, or V1 deletion is claimed. Timing during shared host
load is diagnostic only. The three original ShaderManager unlit/Gouraud/Phong
front-only-color declarations remain unmapped.

## Artifact binding

The original `receipts.json` records the source revision, environment fields, and SHA-256 of
uncompressed logs and manifests. `gpu-source-hashes.json.gz` pins the actual
working snapshot used by the GPU runs. Before the source commit, only the GPU
test's cyan-pixel iterator was named in a local variable to satisfy the 20-line
function limit, and ADR status text was updated. Assertions and production source
were unchanged; final CPU tests and Clippy cover the committed source.

The final `pinned-receipts.json` binds all new compressed logs, manifests and the
exact runner script to checked source `653c731c0`. `pinned-source-hashes.json.gz`
pins every changed Vulkan source/output file; all 77 full-run executable hashes
are recorded after the runs. The final all-target Clippy run passed with warnings
denied. This evidence follow-up changes documentation and archives only.
