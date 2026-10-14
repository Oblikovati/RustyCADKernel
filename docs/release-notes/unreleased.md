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
