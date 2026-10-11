# Martensite translucent surface lighting

Retained Graphic3d triangles now combine PhongFacet, Gouraud and Phong lighting
with ordinary unordered transparency in Vulkan 1.3. Front material opacity and,
when distinction is enabled, back material opacity select the automatic blend
pass. Explicit Blend keeps even opaque material coefficients in that pass;
explicit Opaque forces opaque lighting coefficients without changing the source
aspect. Live material changes rebuild the next snapshot and can move geometry
between opaque and transparent passes.

The source policy is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, particularly
`OpenGl_Context::CheckIsTransparent`, `SetShadingMaterial` and the native
Phong-family shader alpha multiplication. Ordinary surfaces accept finite diffuse
opacity in `[0,1]` for both material faces. Lit lines, markers and caps retain their
explicit opaque-material restriction. Alpha masks, OIT and translucent cap or
annotation rendering remain work to port.

Opaque geometry renders before transparent surfaces until a layer depth clear.
Transparent surfaces retain submission order, test depth and suppress depth
writes. Lighting multiplies enabled vertex alpha by material opacity. The color
attachment blends in linear space and preserves opaque framebuffer alpha.
Low-level callers can override the selected ordinary surface pass after attaching
lighting with `draw.with_lighting(state)?.with_surface_blending(false)?`; retained
Graphic3d extraction derives this policy from the source aspect instead.

Six CPU opacity regressions cover both material faces, invalid coefficients,
explicit pass overrides and rejected nodal/cap paths. Retained tests cover live
opaque/translucent/opaque transitions, snapshot independence, distinguished back
opacity and explicit Blend/Opaque policies. Twenty-one new analytic GPU scenes
check all three algorithms, zero and fractional opacity, vertex alpha, back faces,
opaque-pass overrides, clipping, foreground ordering and depth preservation.
These run alongside the previous 21 lighting scenes in CI. The blended green
encoding of linear 0.5 accepts 187 or 188 because device sRGB conversion rounds
differently; all other asserted channels and depths remain exact. Native capture
comparisons retain their exact RGBA gate.

The local `lit-transparency` family compares 13 native OCCT modes across all six
size/projection/translation configurations. It includes distinguished back faces,
vertex colors, clipping, explicit modes and live material/light/opacity/pass
updates. Native oracle checks remain local during porting. Strict capture gates
retain their existing thresholds; differences stay recorded as open evidence.

The `lit-surface-opacity-updates` release performance workload alternates opaque
and translucent material states while updating a normal and light. It measures
V3d scene extraction, submission, waiting and color/depth readback with 20 warmup
frames and 100 samples on both lavapipe and the native GPU. All 35 workloads run
with validation disabled; functional checks keep validation enabled.

```sh
cargo test -p martensite-vulkan --all-targets --locked
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --case-family lit-transparency --profiles lavapipe host \
  --host-icd /usr/share/vulkan/icd.d/radeon_icd.json \
  --samples 100 --warmup 20 --output target/vulkan-lit-transparency/receipt.json
```

Receipts, raw captures, comparison masks and timings are retained in
[the integration evidence](../porting/runtime-evidence/lit-transparency-2026-10-11/README.md).
This advances renderer replacement behavior; it does not establish complete
legacy rendering parity or invent OCCT API-completion credit.
