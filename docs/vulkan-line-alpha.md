# Martensite native line alpha

The Vulkan 1.3 backend now preserves OCCT's in-place line blending. Unlit
segments and polylines accept finite RGBA vertex or aspect colors. Only
`Graphic3dAlphaMode::MaskBlend` enables `SRC_ALPHA` / `ONE_MINUS_SRC_ALPHA`
blending. Opaque, automatic, explicit Blend and Mask line modes remain unblended,
matching `OpenGl_PrimitiveArray::Render` at upstream `3d097a0`.

This differs from the surface transparency pass. Native non-fill arrays remain
in the opaque pass, preserve submission order and retain their layer's depth
writes. A zero-alpha forced-blended line therefore preserves background color
while writing depth and occluding a later line. Native line programs bind Opaque
shader alpha policy even for MaskBlend: a 0.9 cutoff does not discard a 0.25-alpha
line. Surface deferral and point/text alpha policies remain independent.

`LineStyle::with_alpha_blending(true)` exposes the same immutable submission
policy. Blended line pipelines are created lazily, with independent clipping and
depth-clamp variants. Device line capability and width checks execute before
pipeline selection. Ordinary pipelines and their shader binaries are unchanged.

Eleven paired fixtures cover all five alpha modes, zero opacity, vertex alpha,
near/far occlusion, width, stipple and clipping. Forty-four committed native
RGBA/depth buffers represent a 64×64 orthographic view on lavapipe and native AMD.
Rust GPU checks replay these assets without loading or building OCCT. Analytic
checks separately cover depth writes disabled and forced-blend ordering.

The local six-view capture matrix passes 66/66 strict cases on native AMD and
50/66 on lavapipe. Remaining software endpoint coverage differences stay strict
failures; no pixel/depth thresholds were changed. Fractional fixture Y coordinates
isolate alpha behavior from the already known integer-boundary row ambiguity.
Existing gradient, sloped-depth and persistence precision gaps remain open.

The release benchmark adds `line-alpha-updates`, changing live opacity and alpha
mode through V3d extraction, Vulkan submission and color/depth readback. Final
100-sample/20-warmup median/p95 timings are 0.3711/0.7916 ms on lavapipe and
0.2642/0.3610 ms on AMD. Both complete 36-workload runs retain every raw sample.
The new workload has no accepted regression baseline. The final software run
also misses the historical surface-bound median and window-present p95 budgets;
the initial run passed those ten budgets. Native historical probe identity does
not match, so its historical regression comparison remains unverified.

Verification from combined local `main` also passed both profiles' functional,
analytic, lighting and native-buffer checks. Integrated median/p95 timings are
0.3653/0.4367 ms on lavapipe and 0.2602/0.3710 ms on AMD. The integrated software
run passed all ten historical workload budgets. Its receipt retains the earlier
misses, native identity mismatch and observed shared-host compilation caveat;
the new alpha baseline remains unaccepted.

```sh
cargo test -p martensite-vulkan --all-targets --locked
cargo run -p martensite-vulkan --example offscreen_acceptance --locked
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --case-family line-alpha --profiles lavapipe host \
  --host-icd /usr/share/vulkan/icd.d/radeon_icd.json \
  --samples 100 --warmup 20 --output target/vulkan-line-alpha/receipt.json
```

Native OCCT comparisons run locally during porting. Windows/macOS validation is
deferred. [Runtime evidence](../porting/runtime-evidence/line-alpha-2026-10-11/README.md)
contains captures, hashes, timings and open strict failures. This renderer slice
adds no OCCT API completion credit and does not establish complete line parity.
