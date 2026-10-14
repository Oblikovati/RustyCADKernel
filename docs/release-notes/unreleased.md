# Unreleased

Next compatible patch target: **0.2.8**. Workspace version changes and publication
remain with the release coordinator.

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
Unreported native destruction, shared-display reinitialization, the original
intermittent `BadWindow` and fresh performance acceptance remain open.

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
