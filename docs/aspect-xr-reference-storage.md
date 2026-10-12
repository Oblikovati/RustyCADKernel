# Martensite XR session reference storage

OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` returns references from
`Aspect_XRSession::HeadPose`, `TrackedPoses` and `GenericAction`. Martensite's new
`head_pose_ref()`, `tracked_poses_ref()` and `generic_action_ref()` name the actual
protected fields. Existing copied convenience getters remain explicitly copied
values; they are not the native reference bindings.

Tracked poses and each role's action array now use actual signed-bound `Array1`
storage. Array-object aliases observe current bounds and resize. Retained element
aliases instead capture a physical offset and address. A valid same-length rebase
preserves that physical slot, even when a fresh logical lookup names another
slot. Observing an action acquires a storage lease without an intrusive action
reference, and releases the field guard before an overridable session call.

All four virtual action-input paths and the vibration members accept
`AspectXrActionRef`: for caller storage use
`session.get_digital_action_data(&AspectXrActionRef::borrowed(&action))?`; for a
session field use `session.generic_action_ref(role, kind)?`. The real OpenVR
implementation and existing controller test consumer use this interface. The
public protected `tracked_poses` and `role_actions` fields now expose actual
`Array1` values rather than vectors. These are intentional Rust interface changes.

Thirteen native rows prove signed, nonzero and empty array bounds, live head and
array identity, and actual action bounds failures. Four additional rows prove
same-length action rebasing, physical slot replacement and exact native intrusive
counts. Each oracle was rebuilt and run twice; both full-precision outputs are
byte-identical. The fixtures, compressed second runs and source/header/executable
hashes are checked in. Rust-only tests replay those assets and cover transform,
action, array and caller-owned references. Existing backend tests preserve typed
action validation and virtual forwarding.

Actual element reallocation makes retained native element references dangling.
Those native undefined operations are excluded from replay; Rust adapters raise a
typed `Failure` on changed storage instead of retargeting silently. Zero-length
ranges remain defined array-object operations. No GPU or hardware headset is used.
The XR session class and full controller frame methods remain partial; this slice
restores only the precise getter and protected-storage contracts listed in its
scoped claim fragment.
