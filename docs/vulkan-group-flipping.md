# Martensite ordered group flipping

`Graphic3dDrawGroup::set_flipping_options` retains native push/pop commands rather
than a final enabled flag. It uses the ported `Graphic3dFlipper` with float
reference axes and model-view orientation, matching `OpenGl_Flipper` at pinned
OCCT `3d097a0`. Enabled commands save the current matrix even when no rotation is
needed; nested disabled commands restore the exact saved model and clip matrix.
The group's selection flipper and structure flipping flag persist after disable.

Each geometry command receives its current model transform for primitive
positions, world-space clipping, normals/material lighting and oriented text.
Ordinary commands retain the previous precision path when no flip occurs.
Native capping applies `OpenGl_RenderFilter_FillModeOnly`, which rejects flipper
commands. Its masks therefore keep the structure transform and bypass both the
group transform and its flips; ordinary surfaces still use their command model.

The paired capture family contains front/back/upside-down cameras, a translated
reference origin, nested brackets, group zoom persistence, oriented text,
Phong-facet surfaces and a transformed closed cube with capping. Every mode uses
the existing six viewport/projection/translation controls on lavapipe and native
AMD. `group-flipping-updates` adds complete release redraw/readback timings with
alternating live camera orientation, alongside all existing performance workloads.
Run with the appropriate native build path for the checkout:

```sh
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full \
  --output target/flipping.json --case-family group-flipping
```


The first captures exposed fixture/behavior errors: the native lit fixture
needed a matching view mode and `SynchronizeAspects` after its cached model changed.
Replaying flipping during capping contradicted the native fill-only filter. They also isolated an existing blended-glyph alpha defect:
RGB matched, but Vulkan wrote blended alpha while native's default
`buffersOpaqueAlpha` disables alpha writes. Blended glyph and sprite pipelines
now retain the cleared opaque alpha; a regression test records the failing state.
This does not establish renderer-wide parity, transparent-buffer support or
completion of the broader camera, clipping, lighting and text capability families.

Validation receipts are retained under
`porting/runtime-evidence/group-flipping-2026-10-11/`. The original strict failures
remain evidence; acceptance uses unchanged RGBA and depth comparison gates.

The verified functional matrix retains four lavapipe text failures (14 pixels per
case differ by one RGB unit) and three native AMD perspective-cap depth failures
(maximum delta about 5.19e-6 against the unchanged 1e-6 gate). The remaining
50/54 software and 51/54 host cases pass. These are partial text/raster evidence,
not exceptions that turn a failed profile into a passed one. All 25 workload
budgets are compared independently with exact device/driver and configuration
matching. The final source comparison passes all host budgets but exceeds 16
lavapipe median/p95 budgets across 13 workloads; both raw distributions and
failures are retained. The preceding source comparison passed both devices.
Concurrent workspace builds were active during the final measurement; no
causality or performance acceptance is inferred from that observation. This
does not accept a baseline whose functional profile failed.
