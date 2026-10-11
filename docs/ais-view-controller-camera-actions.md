# Martensite camera actions and lazy detection

Martensite implements the full pinned OCCT `AIS_ViewController::handleCameraActions`
and protected `contextLazyMoveTo` operations on real Rust views, cameras, AIS
contexts, point presentations and retained request buffers.

```rust,ignore
let mut controller = AisViewController::new()?;
controller.handle_camera_actions(&context, &view, &walk)?;
controller.context_lazy_move_to(&context, &view, &Vec2::new(320, 240))?;
```

The dispatcher retains upstream ordering: orientation and fit precede displayed
object transformation callbacks, gesture cleanup and HLR restoration; pan,
Z rotation, orbit rotation, view rotation and queued zoom/focus then update
actual camera state. Constructor-owned AIS_Point presentations carry the real
pan/orbit anchors. Their context association, transformations, layers and display
status participate in gesture cleanup and view invalidation.

`AisViewController::new` and `with_event_timer` now return `Result<Self, Failure>`.
The former infallible `Default` implementation is removed because creation of
these real owned presentations can fail. Callers propagate the result; tests
assert successful construction. No constructor or full class parity is claimed.
Upstream leaves the initial roll member uninitialized; Martensite deliberately
retains its deterministic zero default, and native replay seeds a defined roll.

Void transformation hooks preserve their existing signatures. Named overrides
may use `rustycad_tkernel::standard::raise_native_failure` to raise a typed
`Failure`; the fallible action and lazy-detection boundaries catch this type and
return it. Unrelated panic payloads resume unchanged. Fallible dependencies
propagate ordinary `Result` failures directly. Successfully applied changes stay
applied on later failure, including orientation, anchor state, animation stop,
cursor cache and already executed scrolls. The original zoom queue clears only
when the entire queue succeeds.

Lazy detection temporarily calls actual AutoZFit, executes actual MoveTo and
restores depth only after success. A wrong-viewer failure leaves the temporary
depth and updated cursor cache, so repeating the same pixel takes the native
cache guard. Changed or forced detected owners invalidate every active view's
immediate layer. Actual legacy and shader grid activation controls echo updates;
the CPU null shader driver faithfully reports no shader echo hit.

Run Rust-only behavioral replay with:

```sh
cargo test -p rustycad-tkv3d --lib view_controller_action --locked
```

Committed native fixtures contain 12,540 snapshots with 243 columns each. They
cover 24 request scenarios, five projections, three navigation modes, four grid
states, XR posed/base cameras, repeated frames, display policies, focus and ray
policies, callback failures, unrelated panics, child animation stop/retry and
wrong-viewer lazy detection failure/retry. Every snapshot includes both active
views, full UI/render requests, detected geometry, actual anchors, all retained
rotation and pan state, callback observations and cameras. OCCT is used only to
generate the pinned local oracle assets; the Rust tests do not load native OCCT.

Provenance and both raw 17-digit native runs are recorded in
`porting/runtime-evidence/controller-camera-actions-2026-10-12/`.
The committed fixtures preserve the first full-precision native run. Native
address-dependent hash traversal can change the last bits of GravityPoint sums;
every row/column is compared across both native runs against the unchanged
numeric comparator, with exact metadata, identities, request and trace fields.
Raw byte equality is not claimed. Lower-precision experiments are recorded only
as diagnostics and do not define the committed assets.

The scoped ledger fragment claims only these two methods; frame orchestration,
hotkey expansion, constructor and controller completion remain outside this slice.
