# Critical application blockers

User priority decision: 2026-10-13; native GUI dependency added 2026-10-14. These dependencies take precedence over
broader visualization and exchange expansion in the [porting plan](porting-plan.md).
Retain active work and all existing renderer regression gates. The full OCCT
scope, local-only integration rule, and separate release authorization remain.

Evidence base: app `192db4f99`, consuming Martensite v0.2.5 at
`8118c7e1fae4d28bb5bb5656cd8f86445dadef10`; kernel `bb1e00115`.
The application epic is
[Oblikovati#3563](https://github.com/Oblikovati/Oblikovati/issues/3563).
Creating owned sketch entities is supported, but is not proof that general
sketch profiles can drive valid named solids. A fixed diagnostic prism is not
the R3 editable-part exit gate.

## 1. Prepare tested release of accepted application prerequisites

Owners: [release #3](https://github.com/Oblikovati/RustyCADKernel/issues/3) and
[spline #9](https://github.com/Oblikovati/RustyCADKernel/issues/9).

The owned control/fit spline constructor (`bab95916c`), derivative-handle
semantics (`4ebbba28e`), and exact distance/projection comparisons (`bb1e00115`)
are integrated locally but absent from v0.2.5. The app cannot consume them by
changing a local source path. Spline conversion blocks complete sketch curve
queries; exact comparisons are prerequisites for endpoint and incidence ordering.

Next action: run the owner package and extracted-source acceptance workflow
for a candidate containing these commits and retain the exact source revision,
manifest and checksums. Publish only under the existing release authorization
policy. Then the app can pin that tested release and run its complete local gate.
Do not wait for unrelated renderer feature expansion to prepare this candidate.

Acceptance: preserve the [spline receipts](sketch-spline-construction.md) and
[comparison receipts](sketch-edit-comparisons.md), pass owner release gates,
and verify application adoption through typed errors, source isolation,
recompute, persistence and native consumption. A release tag alone is not
application acceptance. Exact comparisons do not certify constructed roots.

## 2. Complete one general analytic sketch-region-to-solid path

Owner: [region #10](https://github.com/Oblikovati/RustyCADKernel/issues/10).

The owning-kernel boundary and deletion gate are recorded in
[ADR 0014](adr/0014-owned-analytic-sketch-regions.md).

This is the critical functional blocker for general parameter-driven sketch
profiles and extrusion. The cardinality diagnostic and exact numeric primitives
do not establish complete regions, source identity, or operation validity.

Next action: drive one representative solved sketch, including a single closed
analytic curve, through the general interference/arrangement pipeline, exact or
filtered classification, region construction, prism generation, and ordered
validation. Reuse mapped builders; do not add a separate circle implementation
or promote the app's sampled diagnostic classifier into production acceptance.

Acceptance: satisfy every requirement in issue #10, including per-face OCCT
checks, exact/degenerate classification cases, source history and isolation,
measured error and explicit incomplete results. Return a valid named solid with
keys that round-trip and survive the edit that consumed them. Release the tested
contract, then verify app parameter edits, save/reopen, undo/redo and presentation.
Delete the diagnostic construction path when its replacement gate passes.

## 3. Release native GUI composition on the existing view

Consumer: application epic #3563, R1V/R4 native shell. Rechecked against app
`3bbcd8933` and released Martensite v0.2.5 (`8118c7e1`). The app already produces
owned Dear ImGui packets and lowers them to `RenderLayer`, but cannot attach that
layer to `Graphic3dRenderView`. Its public `capture`/`try_redraw` path still builds
submissions through private `view_scene::frame_for_view`; native presentation
refuses an injected `RenderBackend`. Native status text and slot outlines do not
supply this composition contract.

This blocks the visible command controls, sketch selection controls, and placement
field widgets. It therefore takes precedence over trim/extension expansion and
broader renderer coverage. Keep the existing scene and rendering regression gates.

Next action: expose a kernel-owned attach/replace/clear operation for an owned GUI
layer on an existing native view, with expected target extent. Preserve the model
scene, clear color, layer ordering, float colors, UVs, intersected clip chains and
alpha blending. Keep device, swapchain, capture and resource retirement in the
existing renderer. UI must not affect Fit bounds or model selection identity.
A resize must require a fresh packet; retained font images must outlive GPU use.
Do not introduce a second application renderer or production offscreen-copy path.

Acceptance: native model-plus-UI pixels and GPU capture must agree after draw,
replacement, removal, resize and close. Cover invalid extent and failed replacement
without stale capture or lost model content, ordered alpha blending, all four clip
edges, atlas lifetime and validation on software Vulkan and the native GPU. Release
the tested contract, then run the application's full gate with native GUI commands.
The consumer requirements are recorded in application
`architecture/rust-port/implementation-imgui-packets.md`.

## 4. Certify the incidences consumed by trim and extension

Owners: application epic #3563 and the shared exact-classification dependencies
of region #10. This work shares the region intersector; it is not a second engine.

The released intersection adapter returns native completion, points, overlaps,
parameters and measured point residuals. Those values alone do not establish
root completeness or geometric root-error bounds. The exact comparisons added
in `bb1e00115` order supplied coordinates; they do not resolve that gap.

Next action: establish complete bounded and support-line incidence results for
the admitted curve representations, including spline construction from item 1.
Certify root selection and construction error before selecting an edit target.
Keep tangencies, overlaps, periodic trims, source identity and deterministic order
explicit. Named incomplete results must remain failures at the edit boundary.

Acceptance: independent analytic/native fixtures, near-degenerate generated
cases, complete candidate ordering, and explicit measured error. The app then
uses its atomic endpoint ownership operation and validates the final solved edit,
with undo/reload and native tests. A seed point can move during recompute; a
pre-solve hit is not proof of the final endpoint's incidence.

## Resuming the broader queue

Reduce a blocker's priority only when its consuming application test passes
against a tested release, or new evidence proves that the consumer no longer
requires it. Source declarations, local commits, and focused unit tests do not
clear an application blocker. Once these critical paths are unblocked, resume
the visualization and Vulkan sequence ahead of general exchange expansion.
