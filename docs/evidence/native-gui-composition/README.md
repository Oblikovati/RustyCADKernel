# Existing-view native GUI composition evidence

This is a Martensite extension, based on checked local main `63ad971f1`.
`frozen-source-sha256.json` binds the Vulkan sources, shared test helpers, shaders,
SPIR-V, manifests and lockfile actually used for these checks. There are no new
OCCT mapping or class-completion claims, and the original controller Frame/Pick/XR
work remains deferred in its preserved branch.

`functional-results.json` records all-target Vulkan results on lavapipe and AMD:
673 passed, zero failed, five existing ignored tests, 73 successful groups each.
The identical shared GUI assertions execute display independently in the ordinary
toolkit and on a real mapped X11 window in the explicit native example. The latter
also checks a genuine application-owned neutral window reporting stale dimensions,
actual swapchain rejection, fresh packet retry and native pixel equality. Both
native logs are successful and every validation-error assertion passes.

The `*-frozen.rgba` and `*-frozen.window.rgba` files are row-major 64x64 RGBA8;
`*-frozen.depth` is 64x64 little-endian f32 depth. Each actual X11 color buffer is
byte-identical to its GPU readback. Software and host colors and depths also match.
The tests independently verify straight linear alpha, asymmetric font UVs, all
four clipping edges, model transparency ordering, model depth, replacement,
resize, selection/bounds isolation and font ownership/retirement. New analytic
sRGB expectations allow one quantization byte; old fixtures/tolerances are unchanged.

The retained model alpha regressions preserve the original unit red and a genuine
GPU red from deliberately disabling only GUI alpha isolation. The latter produced
blue rather than the expected translucent green-over-blue result. The owned native
model structure remains alive across all frames. Restored tests pass, including a
GUI marker that would otherwise reset the model threshold, clear and two subsequent
model frames. `alpha-disabled-source-hashes.json` records the two source versions.

Clippy with warnings denied, documentation with warnings denied, workspace format,
owned included-test format and canonical test discovery pass. The global included
formatter reports two unchanged TKBO import layouts; `inherited-format-source.json`
proves that unrelated file is byte-identical to the branch base. The first native
extent invocation's missing-trait compilation error is retained separately from
the successful corrected invocation, rather than treated as a rendering result.

Driver/ICD and pinned validation binaries are bound by `driver-validation-sha256.json`.
The software run uses SSE2/128-bit lavapipe, device index zero, and native runs use
RADV STRIX_HALO, device index zero. Both mapped runs use `DISPLAY=:1`. All functional
commands use the pinned validation wrapper, two Cargo build jobs, no incremental
compilation and zero dev/test debug information. See the public usage and commands
in `docs/native-gui-composition.md`.

The coordinator-approved frozen timing window completed on both devices using
existing executables, seven native-view workloads, 20 warmup frames and 100 samples
per workload. All 1,400 samples are finite and positive. Four pinned-validation
native bookends passed; their actual-window/capture buffers remain byte-identical
across phases and devices. Seven hash bookends verified all 827 inputs, both
executables and six driver/validation files unchanged. The sampled process monitor
observed no competing compiler, test or benchmark workload; ordinary desktop GPU
owners are retained in the readiness/final receipts. Whole-process peak RSS was
107,380 KiB on lavapipe and 66,880 KiB on AMD. Raw samples and exact commands are
retained; this is no speedup, baseline acceptance or GPU-memory claim.

`run_frozen_performance.py` is the exact one-off observation script retained for
reproducibility, not a new production tool or test selection interface. The source
revision bound to the runs is `9c9f10dfa08d7b6286a0e2422e26d67ddd527784`.
`performance-freeze.json` records the earlier pre-GO freeze proposal; the completed
results, commands and bookends supersede its pending-GO field.
