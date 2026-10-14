# Unreleased

Next compatible patch target: **0.2.8**. Workspace version changes and publication
remain with the release coordinator.

## Conditional polynomial extrusion-side integral

`certify_polynomial_side_integral` adds behavior v1 for full-[0,1] polynomial
Bézier/line side integrals under supplied affine embeddings. It returns a
certified scalar interval, absolute area-error bound, explicit policy and work,
or a typed incomplete/cancelled/input failure. Default absolute area error is
1e-5 with at most 4096 subdivisions; policies are never silently widened.
Existing signed boundary/sweep and mapped integration behavior is unchanged.
No call establishes face/body validity or replaces source-bound region checks.
See [ADR0024](../adr/0024-conditional-polynomial-extrusion-side-integral.md).

## Native window lifecycle

Native-window-lifecycle behavior v1 retires an X11 window after its matching
`DestroyNotify` is delivered. Applications should continue delivering native
events to `XwWindow::process_message` before issuing subsequent view operations.
Closed targets expose their original native ID and destruction cause; renderer
operations return typed closed-target failures and refuse cached images. Rebind
to a live window and submit a fresh GUI packet before capturing a new frame.

Explicit rebind always invalidates cached frames and GUI packets, including a
rebind with the same native handle and extent. Submit a fresh packet and redraw
before capture; numeric XID equality cannot establish a shared lifetime. Ordinary
live-rendering output is unchanged. Prior fatal behavior is retained through
revision-bound diagnostic replay; see the [migration decision](../adr/0018-delivered-native-window-destruction.md).
Unreported native destruction, externally initiated display reset, the original
intermittent `BadWindow` and fresh performance acceptance remain open.

## Retained native display connections

Native-display-lifecycle v1 refuses `AspectDisplayConnection::init_checked` while
owned/wrapped XwWindow objects or Vulkan instances retain native consumer leases.
The old connection remains unchanged. Drop the retaining windows and Vulkan
runtime before resetting, or create a separate connection. A removed view alone
does not end Vulkan's connection retention.

Use `init_checked` for typed operation/identity/cause/count diagnostics. Existing
`init` remains a lossy legacy error adapter for one migration milestone only;
[ADR0019](../adr/0019-retained-native-display-connections.md) records its deletion
task and deadline. This does not certify externally owned or independently
aliased raw display lifetimes. Existing self-alias and failed-unleased-reset
hazards, historical intermittent BadWindow, performance and application
acceptance remain open. Versioning and publication remain with the coordinator.

XwWindow native operations and renderer admission use the original leased display,
even when protected display metadata is replaced. Custom AspectWindow implementations
retain the default metadata lookup and must define their own native binding lifetime.
The explicit non-null invalidate_content display override is preserved and leased
through its send callback; same-server XID validity and external raw lifetime remain
caller obligations. Failed admission causes no send and cannot be reported by that
legacy unit-return adapter; it is not a successful invalidation or delivery guarantee.

## Opt-in unlit side colors

`Graphic3dRenderDriver::with_unlit_face_color_policy` and
`with_backend_and_unlit_face_color_policy` accept
`RenderUnlitFaceColorPolicy::SideColorsV2` for distinguished, untextured unlit
triangle RGBA and front-only vertex colors. Existing constructors remain on
`LegacyV1` and preserve the owned rejection contract. Callers opting in should
record `behavior_version() == 2` with replay settings. Back alpha participates in
BlendAuto routing and the existing strict Mask/MaskBlend cutoff. Without a
vertex color array, the front-only flag has no effect; distinction chooses the
back interior RGBA. Textured and fixed-function behavior is unchanged.

Default migration and V1 deletion remain gated by
[ADR 0021](../adr/0021-unlit-side-colors.md); this entry does not change workspace
versions, activate the default, or complete a release/milestone.

## Opt-in cumulative pcurve import

BRepGraph import can select `populate::PCurveImportBehavior::CumulativeOccurrenceV2`
through the additive `*_with_behavior` population entrypoints or
`ShapesViewMut::add_with_options_and_behavior`. This preserves stored pcurves
on located face/wire/edge occurrences, including both directed seam uses.
Existing entrypoints and `Options` layouts keep their V1 behavior.

Selection applies to new definitions only: append reuses existing definitions
without repair. Migrate by rebuilding a fresh graph with V2 and auditing it;
a successful import is not body validation. Polygon/triangulation lookup and
mutable graph ownership are unchanged. Default migration and V1 production
removal remain gated by [ADR0026](../adr/0026-cumulative-pcurve-import.md).
