# Martensite homogeneous vertex input

Graphic3d Vec4 positions now retain their fourth component through scene extraction,
GPU upload and all four vertex shaders. Vec2 and Vec3 positions supply `w = 1`.
`RenderVertex::new` keeps that default; `RenderVertex::homogeneous` supplies a full
position. Zero and negative finite weights reach the native clipping stage. NaN
and infinite components fail submission validation.

This is a prerequisite for the infinite capping geometry in pinned OCCT
`OpenGl_CappingPlaneResource.cxx`, whose four triangles use a finite center and
axis directions with zero weight. It does not implement the stencil capping pass.
The complete legacy renderer scope remains unchanged.

`Graphic3d_Group.cxx` at `3d097a0328e71b826377d4814ab05ec3c3d23871`
computes bounds from raw XYZ even for Vec4 positions. Martensite follows that
policy; it does not divide retained bounds by the input weight. The vertex shader
multiplies the full homogeneous position before converting OpenGL clip depth to
Vulkan clip depth. Clipping world positions preserve the same weight.

The paired native and Rust fixtures cover triangles, polylines and point sprites,
unit/doubled/unequal weights, directions at infinity, negative weights and user
clip planes. The original negative-weight triangle becomes collinear after
perspective division; it remains in the matrix alongside a nondegenerate negative
control. Each mode runs through six projection, extent and structure-translation
configurations on lavapipe and the native GPU. Raw failures remain failures under
exact RGBA comparison and the existing depth tolerance of `1e-6`.

The `homogeneous-position-updates` workload measures live weight changes through
V3d scene extraction, GPU upload, draw submission and readback in release mode.
It does not replace the existing workload budgets.

Run CPU and fixture checks with:

```sh
cargo test -p martensite-vulkan --all-targets --locked
PYTHONPATH=tools python3 -m unittest discover -s tools/tests -p 'test_vulkan*.py'
```

The full `tools/verify_vulkan.py` harness includes the homogeneous modes and
performance workload; use the source/build/profile options in
[the renderer harness documentation](vulkan-renderer.md#reproduce-the-checks).
Capping, strict raster parity at clip-weight boundaries and section appearance
remain open until their own native evidence passes.

## Runtime evidence

The [full matrix](../porting/runtime-evidence/vulkan-homogeneous-full-2026-10-10.json)
ran 2,664 native/ported comparisons. All 2,448 preceding cases retain byte-identical
native and Martensite color/depth buffers; none of their passing results regressed.
The [final fixture audit](../porting/runtime-evidence/vulkan-homogeneous-final-2026-10-10.json)
rechecked all 216 original homogeneous cases after fixture decomposition and added
36 nondegenerate negative-weight controls. Production renderer and shader source
fingerprints are identical between these captures. Validation reported zero errors.

| Final homogeneous matrix | Lavapipe | AMD Radeon 8060S |
| --- | ---: | ---: |
| Triangles | 37/42 | 42/42 |
| Lines | 24/42 | 39/42 |
| Points | 34/42 | 42/42 |
| **Total strict passes** | **95/126** | **123/126** |

The nondegenerate negative-weight triangles pass on both devices. Software failures
remain for five degenerate negative-weight triangles, eighteen infinite/negative
line cases and eight point cases (including two unit-weight controls). AMD retains
three infinite-line differences. Their cause is not yet fully isolated; they remain
failed evidence, without a tolerance change or completion credit.

All 126 renderer tests, 36 Python harness tests, Clippy with warnings denied and
all sixteen shader freshness checks pass. The pinned source inventory check passes.

Initial optimized runs measured homogeneous weight updates at median/p95
**0.392/0.484 ms** on lavapipe and **0.315/0.573 ms** on AMD (100 samples after
20 warmup frames). This new workload has no accepted baseline yet. The first
software run exceeded existing circle and clipping p95 limits; those failures are
preserved. A fixed
[main/change/change/main comparison](../porting/runtime-evidence/vulkan-homogeneous-performance-2026-10-10.json)
then passed all ten existing budgets in every round on both profiles. No budget
was relaxed and no result was discarded. Nine workloads, including this new one,
still lack accepted baselines in the broader gate.

This integration adds no OCCT API inventory claims and marks no renderer family
complete. Stencil attachments, closed-group odd/even masks, remaining-plane cap
clipping and plane/object section appearance are the next capping work.
