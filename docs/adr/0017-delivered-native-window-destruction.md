# Retire native targets after delivered destruction

Decision: native-window-lifecycle behavior version 1. Migration recorded
2026-10-14 for the next compatible patch, targeted at 0.2.8. This changes the
previously unsupported delivered-destruction outcome; valid live-window behavior
and public method signatures remain unchanged. The coordinator owns workspace
versioning and release; this decision authorizes neither publication nor a claim
that the original intermittent GUI failure is fixed.

The [baseline diagnostic](../evidence/native-window-lifecycle-2026-10-14/README.md)
reproduces a delivered `DestroyNotify` followed by public resize terminating in
Xlib. Behavior v1 matches the event's destroyed window ID, not its event/parent
field. It retires native ownership once and retains the original ID, cause and
behavior version as immutable diagnostic values. Further native operations and
destruction of that object must not send requests for the retired ID.

Closed-target capture, redraw and GUI submission are refused with typed operation,
target, `Closed` failure class and `DestroyNotify` cause. An earlier successful
capture is not a successful result for a closed target. An offscreen target does
not bypass known closure. Existing void callbacks retain their signatures; their
closed state and subsequent renderer failure remain observable. Legacy geometry
accessors return invalid zero extents or unknown resize status for a closed
window, never accepted geometry or a rendered image. Its explicit closure record
distinguishes this state from an uninitialized or virtual target.

Rebinding the existing view releases its presenter, invalidates the preceding
capture and any GUI packet tied to that target, and requires a fresh successful
frame. Removal and repeated delivered destruction are idempotent. Foreign-window
events must not close this target, including events whose parent ID happens to
equal this target.

The earlier fatal behavior remains available for revision-bound diagnostic replay
at `c814d12b3`, using the archived probe sources. This is the recorded migration
from that unversioned behavior; fatal termination is not a selectable production
policy in behavior v1. Live-window expected results are not changed.

Scope excludes unreported external destruction, reinitializing a display while
native objects retain it, and attributing the historical intermittent `BadWindow`.
No global Xlib error handler, mutable global algorithm state, alternate renderer,
redraw retry, sleep, baseline widening or new OCCT oracle is introduced.
