# Martensite controller click and touch dispatch

Martensite now ports 11 more captured `AIS_ViewController` declarations from OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: camera/object animation ownership,
camera animation abort, mutable selection bindings, mouse clicks, and touch
add/update/remove. Controller class and constructor claims remain partial.
Mouse buttons/position, 3D-mouse, full window-listener interface, camera/render
flushing and XR remain open. Animation ownership is ready for redraw dispatch;
this slice does not claim that redraw scheduling is complete.

Each controller owns distinct, nonnull, stopped default animations. Camera
animation duration is 0.5; its target view is null. Setters retain handle identity
and accept null without stopping the old owner. Native default animation names
are retained because callers can inspect them. Running camera abort stops the
animation and detaches its view; stopped and paused animations preserve their
view. Object handles retain polymorphic animation callbacks. Clicking pauses an
active object animation before looking up selection, even for an unbound button.
An error from an overridden pause propagates before selection changes.

The default selection map binds left click and left+Alt to Replace, and left+Shift
and left+Alt+Shift to XOR. Other combinations are unbound. Mutable map edits affect
live dispatch. A bound click queues one picking point; duplicate points remain.
The native base click ignores the double-click flag. Dispatch edits only the UI
queue and preserves the renderer queue.

Touch add suppresses highlighting and queues a new gesture. One finger starts
rotation and optional dragging. Two fingers abort dragging and start panning.
Clear-before dispatches controller removal, including stop/selection bookkeeping.
Updating an unknown id dispatches add; updating a known id retains its origin.
Duplicate add preserves the stored touch while restarting controller click and
rotation origins at the supplied point. Removing an
unknown id returns false without altering state. A transition to one finger
resets its current point to its original point to avoid a pinch discontinuity;
a transition to two fingers restarts panning from the first retained touch.

The last finger stops dragging and optionally requests selection application.
A valid unmoved touch emulates a left click at its original integer pixel. Clearing
all touches aborts clicking. Movement cancels clicking when its maximum absolute
component exceeds the scaled threshold. Tolerance remains unclamped. Comparisons
preserve `NCollection_Vec2`'s exact conditional branches for NaN components.
The double-tap timer reuses the injected clock, accepts elapsed <= interval, and
restarts after expiration; tests do not sleep.

Local pinned native generators reproduce 163 click/animation and 254 touch
snapshots byte-for-byte. Committed assets include header, implementation,
generator and fixture hashes. Rust tests replay all transitions, and cover
handle identity, callback errors, independent buffers and exact timer boundaries.
Native double-click callback flags are checked through timer tests; the base click
method deliberately ignores the flag. Pixel-to-integer comparisons use finite
coordinates within the upstream integer range, where native conversion is defined.

Run `cargo test -p rustycad-tkv3d --all-targets --locked`. CI requires no OCCT
headers, compiler or runtime. [Mouse button and pointer-motion dispatch](ais-view-controller-mouse.md)
builds on these transitions and the existing [keyboard navigation](ais-view-controller-keyboard.md).
[3D mouse routing](ais-view-controller-space-mouse.md) now supplies HID navigation.
Camera/window application remains open.
