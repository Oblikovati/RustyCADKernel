# AIS detection and dynamic highlighting

The context ports both public `MoveTo` overloads and their shared protected
`moveTo` body from OCCT `3d097a0`. Pixel detection uses the camera, window and
selector traversal; axis detection uses the selector's native axis volume.
A view from another viewer raises ProgramError before picking or clearing state.

Detection records every filter-accepted rank. With OnlyTopmost, rejecting the
first hit prevents automatic detection of a lower hit but retains accepted ranks
for cycling. Iteration through detected owners has its own cursor, independent
of the highlighted owner. Repeated detection of the same owner skips highlighting
unless the virtual owner forces it. Selected-owner and automatic-highlight flags
follow the native branches, including clearing an old owner when replacing it
while automatic highlighting is disabled.

Dynamic highlighting selects a custom object drawer, or the appropriate context
drawer slot. `AisHighlightStyle::read` rereads that original slot after callbacks;
it does not select a different slot when object attributes change. AcceptDisplayMode
can replace that drawer without a context or object lock spanning the callback.
Mode selection preserves style, explicit mode, object mode and default mode order.
Slow-style detection intentionally uses the last picked owner's decomposition
flag, as the pinned native implementation does.

Immediate presentation begin/highlight/end calls preserve partial state when a
callback fails. There is no automatic end-draw guard on failure. Clearing and
cycling retain native cursor/reset/callback ordering. Redraw chooses computed
update, full redraw for an invalidated view, or immediate redraw otherwise.

Named native and Rust fixtures compare callback traces, retained ranks, cursors,
owner/object identities, immediate mode, view state and exceptions. Public pick
cases use actual sensitive geometry and orthographic/perspective cameras.
The fixture explicitly marks supplied selections as computed and supplies their
object bounds so the production selection manager registers the real geometry.

The full context remains partial: display/erase/remove, selection actions and
additional attribute APIs are still open. Shape-specific detection accessors
await the concrete AIS shape dependency. This port does not claim those APIs.
