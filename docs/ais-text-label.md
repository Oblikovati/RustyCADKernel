# Martensite AIS text labels

`AisTextLabel` ports pinned OCCT `AIS_TextLabel`, including owned UTF-16 text,
font/style setters, orientation/anchor/flipping state and optional formatter
identity. Setters preserve native notification rules: color and transparency
synchronize retained aspects, while other label setters do not schedule redisplay.
Material assignment has no effect. The class accepts display/selection mode zero.

Presentation computation installs native rotate/zoom persistence, retains explicit
2D persistence for nonzoomable screen text, and centers oriented flipping using
font measurements. The optional formatter belongs to the submitted text; native
label measurements use the standard formatter independently. Bounds and sensitive
rectangles use native world-Z text rotation and both alignment axes. Failed font
creation falls back to point selection; as in OCCT, an oriented flipping bracket
is not closed when measurement fails. A missing context produces a descriptive
Rust failure instead of the native null dereference.

A paired native probe checks 228 numeric values exactly across all 12 alignment
combinations with multiline/tab text, a non-XY orientation and a nonzero angle.
Focused tests cover defaults/setters, opacity, material no-op, selection snapshots,
font failure and retained presentation ordering. The renderer fixture family
`text-label` includes screen, zoomable, oriented, flipped, angled and redisplayed
annotations across the existing size/projection/translation controls. A release
`ais-text-label-updates` workload includes font measurement, presentation/selection
rebuilding and complete Vulkan redraw/readback.

```sh
cargo test -p rustycad-tkv3d --test toolkit ais_text_label:: --locked
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full \
  --output target/text-label.json --case-family text-label
```

Receipts under `porting/runtime-evidence/ais-text-label-2026-10-11/` record actual
capture and timing outcomes. Font raster parity and renderer display modes remain
subject to the existing strict gates; storing native label aspects does not prove
that every legacy text rendering mode is supported by the Vulkan backend.

The verified captures pass 16/36 lavapipe cases and 6/36 native AMD cases. Strict
color/coverage differences remain in zoomed, fractional, rotated and host glyph
rasterization; depth errors at changed coverage are retained too. Both passes
record the same outcomes with unchanged RGBA and 1e-6 depth gates. All 26
workloads are compared independently: lavapipe passes; native AMD retains a
window-present p95 excess of about 0.003 ms. Label updates pass both timing
budgets (median about 0.497 ms software and 0.387 ms host). A failed functional
profile is not accepted as an overall baseline. Renderer completion stays false.
