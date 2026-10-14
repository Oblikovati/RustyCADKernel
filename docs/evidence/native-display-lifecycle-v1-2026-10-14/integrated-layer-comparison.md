# Depth/stencil MSAA resolve validation comparison (2026-10-14)

## Finding and scope

No renderer correction is warranted by the observed failures. The same main test
binary fails with the system validation layer, while the pinned validation layer
passes the main full AMD suite. This matches a specific upstream correction to
validation's implicit depth/stencil resolve ordering. Candidate acceptance still
requires its own run with the exact pinned validation environment; its historical
runner did not record the loaded layer, so that provenance must not be invented.

Inspection was read-only (no builds/GPU runs or repository edits by this investigator).
Only this requested temporary note was written. Coordinator owns the executions.

## Controlled results and provenance

Main revision: `221eebd8a5c527806beab5e806e3a70a2be6e679`.
Main toolkit binary SHA256:
`da606b4c482d6cffe294e331e10a78c97f7ce1c09309bf5b2cb37fbc0bef0a80`.

| Layer | Package / manifest API | Library SHA256 | Main result |
| --- | --- | --- | --- |
| System | `vulkan-validationlayers 1.3.275.0-1` / `1.3.275` | `1d486283e4ceb342ef7931f97782f03b95a9fad379da638537ea24527780a287` | Both named cases fail on AMD and lavapipe: 10 and 6 hazards respectively |
| Explicit pinned extraction | `1.4.313.0~rc2-1lunarg24.04-1` / `1.4.313` | `89e618526d6d1532096dbbd62ee1dc710dda647061cd3b7b66b542dc7dbce104` | Coordinator's full AMD and lavapipe runs: 766 passed, zero failed, five existing ignored per driver |

System library: `/usr/lib/x86_64-linux-gnu/libVkLayer_khronos_validation.so`.
Pinned library: `ROOT/target/vulkan-validation/extracted/usr/lib/x86_64-linux-gnu/libVkLayer_khronos_validation.so`.
Pinned manifest: `ROOT/target/vulkan-validation/extracted/usr/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json`.
Package versions were read with `dpkg-query -W` and `dpkg-deb -f` respectively;
SHA256 values were computed from the actual library files.

The coordinator's pinned AMD environment explicitly sets `VK_LAYER_PATH` to that
manifest directory, `LD_LIBRARY_PATH` to that library directory, and both
`VK_DRIVER_FILES` and `VK_ICD_FILENAMES` to `/usr/share/vulkan/icd.d/radeon_icd.json`.
Runtime source enables `SYNCHRONIZATION_VALIDATION` in `runtime.rs:190-196`.

Comparative executions and exit/log hashes are recorded in
`target/integrated-native-display-v1/layer-comparison/receipt.json`.
Each system-layer case exits 101. The two cases are:

- `actual_colored_glyph_and_live_cap_inputs_replay_native_buffers_without_occt` (10 hazards).
- `actual_same_view_sampling_and_persistent_peeling_fallback_replay_native_assets` (6 hazards).

The candidate at inspected HEAD `df93fc40d44cc37c92cf352f10ea17fea7a7c5c6`
has matching failures in `../Martensite-unlit-side-color/target/unlit-side-color/full-{host,lavapipe}.log`.
Its `run-gpu-regression.py` and `run-lavapipe-regression.py` inherit `VK_LAYER_PATH`,
`LD_LIBRARY_PATH` and validation settings. Their verdict JSON records ICD only.
The investigator's current environment has those layer variables unset, but that
is not proof of the candidate processes' historical environment or loaded binary.

## Primary specification and upstream correction

The specification places fixed-function resolves at color-attachment-output,
including depth/stencil resolves. It guarantees the final resolve follows writes
to the multisample attachment in that rendering scope. Thus the reported color-read
access name is expected, and these same-scope writes do not themselves establish a
missing application barrier. [Vulkan resolve operation semantics](https://docs.vulkan.org/spec/latest/chapters/renderpass.html#renderpass-resolve-operations).

Upstream commit [`51895dc412e343fb0df056f644f270ce8ea350fe`](https://github.com/KhronosGroup/Vulkan-ValidationLayers/commit/51895dc412e343fb0df056f644f270ce8ea350fe),
committed 2024-02-10, fixes ordered accesses for depth/stencil resolve. It separates
color ordering from raster ordering for depth/stencil resolves in both validation
and access recording. This is the precise code distinction observed between:

- [v1.3.275 header, line 233](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/v1.3.275/layers/sync/sync_commandbuffer.h#L233): one color ordering rule; [resolve validation, line 220](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/v1.3.275/layers/sync/sync_commandbuffer.cpp#L220).
- [v1.4.313 header, lines 172–175](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/v1.4.313/layers/sync/sync_commandbuffer.h#L172): separate depth/stencil raster ordering; [attachment-specific selection, line 234](https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/v1.4.313/layers/sync/sync_commandbuffer.cpp#L234).

The installed pinned package is an rc2 build, so the upstream tag comparison alone
is not binary provenance. The actual library hashes and controlled results above
supply the local binary evidence.

## Martensite source and attachment identities

Main and candidate files below are byte-identical (SHA256, relative to
`crates/vulkan/src/backend/`):

| File | SHA256 |
| --- | --- |
| `record.rs` | `1170607be43f6601c41d5b751d4147e1a62a5357c2718550e1fd2a66f793e23d` |
| `render_target_record.rs` | `8e9d81dad48cd6ae5edaaa5fe6f70dc824198a351476b566a6d25c2d3a88cb36` |
| `multisample_targets.rs` | `64d16357b132f2372ff3d729502040585fc6bbe6d9ca3f3ffb38824dced3fdfe` |
| `image.rs` | `f09dcd6a3817e115cfa30ee8083fe964ed29c47513cd239312af865250d8cc6f` |
| `capping_record.rs` | `77689fcb5b08c784bb5605c88544fe0ca52c1e688684ea839b74faedbe6f1f86` |
| `frame_raster_plan.rs` | `45c6cf53211d2e2426ebc58a7cd5963de276dd63b4ae2c3ca9b73cd02e465da4` |
| `oit_record.rs` | `d80ad81c96d985c2fab384c7d0b74d2f2ad64ffaf4851a70fc1d33a4ed98ae4b` |

Source audit (not an executed identity trace): `frame.rs:106-115` allocates separate
multisample and single-sample targets; `frame.rs:153-156` selects the multisample
source. `image.rs:68,80-81,106-111` independently creates/binds each image and its
memory. `render_target_record.rs:54-63` pairs the source depth view with the separate
single-sample output, using SAMPLE_ZERO only with MSAA. `image.rs:179-191` chooses
D32_SFLOAT or D32_SFLOAT_S8_UINT and matching aspect masks. Both depth and stencil
intentionally use the same combined image view when stencil is present
(`render_target_record.rs:20-28,82-87`). No source/resolve alias was found in this
allocation/wiring path; runtime identities were not instrumented in this inquiry.

The reported prior writes are same-rendering load clears (`BeginRendering`,
sequence 2) and stencil clears (`ClearAttachments`, later sequences). They are
reads of `pDepthAttachment/pStencilAttachment.imageView`, not writes to
`resolveImageView` or subsequent transfer reads. `record.rs:358` and `416` end
rendering; its between-scope dependency at `360-375` already includes color and
depth/stencil reads/writes at ALL_GRAPHICS. Its post-render transfer transition is
at `66-76`. Adding barriers after EndRendering would not address the reported
intra-scope read, and adding a speculative in-rendering barrier is unjustified.

## Remaining discriminating check

Rerun the candidate's same two tests/full suite with the explicit pinned paths and
capture actual layer/loader library identity and relevant environment. Preserve
both old failures. If a pinned-layer failure remains, reduce it independently to
clear-only 4x depth and combined depth/stencil resolves (plus stencil-clear and
multi-scope variants), check exact clear-depth results and zero validation errors,
and log actual source/resolve image/view/memory identities before considering a
production correction. Do not suppress sync validation, alter baselines, remove
MSAA or add a barrier merely to silence the old-layer report.

The comparative raw logs and receipt are committed inside `integrated-raw.tar.gz`,
under `integrated-native-display-v1/layer-comparison/`.
