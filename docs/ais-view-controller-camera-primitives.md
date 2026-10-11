# Martensite immediate camera primitives

Martensite implements the complete native `AIS_ViewController::handlePanning`,
`handleZRotate`, `handleZoom` and `handleZFocusScroll` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. They apply actual queued requests to
the actual `V3dView` camera, leaving every UI/render buffer field retained for
later frame dispatch. For example, `controller.handle_panning(&view)?` applies
render-buffer panning; `controller.handle_zoom(&view, &scroll, Some(&hit))?`
keeps a supplied world hit under the cursor.

Panning uses ordinary V3d pan for orthographic cameras or an undefined anchor.
Perspective anchored pan projects the retained world anchor depth into camera
coordinates and uses viewport width for both pixel axes, as upstream does.
`set_panning_anchor_point` retains all coordinates without validation. The
constructor uses native `Precision::Infinite()` in X; `has_panning_anchor_point`
tests only X with native infinity semantics, including its NaN behavior.

Z rotation uses native start/progress pixels and threshold 0.4. Zoom preserves
the minimum camera distance before scaling, continuous floating-point deltas,
V3d's shared Confusion/1e12 dimension limits, and the perspective world-hit
correction's `height - y - 1` pixel convention. Center zoom and cursor zoom keep
their distinct native paths. Stereo focus uses strict `(0.2, 2.0)` bounds and
preserves absolute/relative focus type. A zero scroll delta follows the negative
focus branch. Focus changes do not abort camera animation or synchronize XR.

Pan/rotation request guards and policy gates run before animation cancellation.
Pan, Z rotation and zoom propagate failures from the real animation before any
camera mutation. A rejected child stop leaves the already-stopped parent with
its view attached, matching native partial failure order. Successful camera
mutations invalidate the actual retained view, then synchronize posed XR changes
back to the base camera. A missing native window dereference becomes a typed
`Failure`; already completed earlier mutations remain observable.

Run `cargo test -p rustycad-tkv3d --all-targets --locked` locally. Committed assets
contain 4,302 native snapshots: 4,140 across 1,380 ordinary cases and three frames,
150 extreme-scale zoom snapshots, and 12 child-stop failure/retry snapshots. Each
166-column snapshot compares invalidation, animation state/view attachment,
anchor state, 21 current/base camera values and all 114 UI/render buffer fields.
Queue fields compare exactly. Cases include all five camera projections,
request/policy gates, center/cursor/world-hit zoom, minimum distances 1/250,
relative/absolute focus, strict focus boundaries and actual XR synchronization.
XR fixtures use a closed real OpenVR session with a seeded head translation;
no hardware calls or GPU submissions are needed. Native generation reuses the
pinned transfer oracle's complete buffer schema and runs locally twice with
identical bytes; Rust replay never loads OCCT.

Orbit/view rotation, navigation keys, animated orientation keys and general
controller frame dispatch remain separate work. The controller class and
constructor remain partial. Windows/macOS testing is deferred.
