# Martensite native point alpha

The Vulkan 1.3 backend distinguishes plain `Aspect_TOM_POINT` from textured
marker sprites. OCCT `OpenGl_PrimitiveArray::Render` binds the Opaque marker shader
for all material alpha modes. Plain points enable in-place source-alpha blending
only for `Graphic3dAlphaMode::MaskBlend`; Opaque, BlendAuto, Blend and Mask retain
unblended color. Sprite draws always enable source-alpha blending. Both preserve
opaque-pass submission order and their layer's existing depth writes.

Plain points discard alpha at or below 0.1, including zero-alpha points. Surviving
fragments write depth even when blended. Native marker programs do not apply the
material's separate alpha-mask cutoff. Unlit sprites replace aspect/vertex alpha
with texture coverage. Alpha-only sprites use aspect or vertex RGB; colored
sprites use their own RGB unless vertex colors override it. Their byte RGB texels
are decoded from sRGB before rendering, matching native color-map texture upload.

`PointStyle::with_alpha_blending(false)` supports immutable direct point draws
without assigning surface transparency ordering/depth policy. Retained extraction
selects the native policy automatically. Plain point pipeline variants remain
independent of clipping and depth clamp, including camera persistence. Existing
surface, line and text alpha rules remain separate.

Twenty-one paired fixtures exercise every plain-point and sprite alpha mode,
zero opacity, the fixed 0.1 cutoff, vertex alpha/color, near/far occlusion,
clipping, colored BALL sprites and fractional-coverage RGBA user sprites.
Eighty-four device-bound native RGBA/depth buffers are committed as Rust replay
assets. Each was generated twice with byte-identical output from pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Rust runtime tests replay these buffers
without building or loading OCCT. Separate analytic checks cover disabled depth
writes, clipping, draw ordering and live retained alpha mutations.

The six-view strict matrix passes 126/126 native AMD cases and 123/126 lavapipe
cases. The three software failures are plain-point alpha exactly 0.1 under
perspective: native interpolation crosses the fixed shader discard threshold.
An exploratory fractional-coverage custom RGB sprite also exposes software sRGB
decode rounding differences of one byte in 12 pixels per view. Its original
strict receipts, source and raw pairs remain committed separately. The final
alpha-focused user sprite uses RGB endpoints; colored BALL fixtures still check
sRGB decoding. Neither gate was relaxed and no backend precision approximation
was introduced. Complete software sprite color parity remains open.

The release benchmark adds `point-alpha-updates` with live aspect opacity/mode
changes, retained extraction, GPU submission and color/depth readback. Both
37-workload runs retain 100 samples after 20 warmups. Median/p95 are
0.3588/0.4551 ms on lavapipe and 0.2982/0.4121 ms on AMD. The ten historical
lavapipe workload budgets pass at unchanged 1.25 median/1.5 p95 limits. Native
historical probe identity does not match, so its regression comparison is
unverified. No new alpha baseline is accepted. The shared host had unrelated
compiler work before timings; process snapshots show only deliberately paused
team compilers during this 10.5-second timing interval.

[Runtime evidence](../porting/runtime-evidence/point-alpha-2026-10-11/README.md)
contains raw captures, selected view images, source hashes and complete timings.

The implementation covers built-in unlit, unbounded, non-indexed native marker
arrays. Existing normal-bearing annotation material restrictions, custom shaders,
indexed/bounded native sprite draw-path differences and multisample alpha-to-
coverage remain separate obligations. This slice claims no upstream API completion
credit or complete cross-device renderer parity.

Native OCCT verification runs only locally during porting. Windows/macOS checks
are deferred. Exact-cutoff software perspective discrepancies retain the strict
RGB and depth gates; no pixel-row, endpoint or tolerance adjustments are included.

```sh
cargo test -p martensite-vulkan --all-targets --locked
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --case-family point-alpha --profiles lavapipe host \
  --host-icd /usr/share/vulkan/icd.d/radeon_icd.json \
  --samples 100 --warmup 20 --output target/vulkan-point-alpha/receipt.json
```
