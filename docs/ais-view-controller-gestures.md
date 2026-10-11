# Martensite touch gesture synthesis

`AisViewController::flush_gestures` implements the complete native protected
`AIS_ViewController::flushGestures` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It observes contact-count transitions
and synthesizes render requests from one or two contacts. Other counts only
update the gesture transition flag. This method leaves the UI queue untouched.

One contact starts the native orbit or view rotation, invalidates the actual
`V3dView`, then compares Manhattan movement with the shared touch threshold.
Precise contacts use `gp::Resolution()`. Ground walking uses mouse acceleration;
orbit and flight use orbit acceleration. Successful movement queues rotation
and drag movement and advances the contact origin. A requested start is consumed
even if subsequent movement stays below tolerance.

Two contacts compute start/end centers and separation. Native Z rotation uses
`atan(numerator / denominator)` only above `Precision::Confusion()`; antiparallel
and perpendicular contacts therefore retain native zero-angle behavior. Pinch
zoom appends a request at the truncated start center. Pan truncates the center
deviation and reverses its Y component. Any accepted rotation, zoom or pan
advances both origins. Starting pan invalidates the view only after its movement
threshold passes. General rotation/pan/zoom policy gates are intentionally not
reapplied in this native stage; the touch Z gate remains active.

The Rust signature returns `Result<(), Failure>`. A missing view is rejected
when native execution would dereference it to invalidate a rotation/pan start.
Already completed request mutations remain visible; later motion and contact
origin advancement stop. A null view remains usable when no invalidation is
needed, matching the method's actual dependency on the view.

Run `cargo test -p rustycad-tkv3d --all-targets --locked` locally. Committed assets
contain 1,116 byte-identical native snapshots across 372 cases and three frames,
including 0–4 contacts, all navigation modes, precise-device combinations,
negative/zero/infinite/NaN scale, retained starts, strict movement thresholds,
Z-angle denominator cases and minimum positive double tolerance. A real native
`V3d_View` calls a named recording CPU driver; Rust uses actual
`V3dView`/`Graphic3dRetainedView` invalidation. Native OCCT is not needed to run
Rust tests. Additional regressions verify UI/request preservation, contact-count
changes and failure ordering for rotation and pinch starts. Windows/macOS
execution remains deferred. Public frame dispatch, camera/render application
and XR remain separate open work; the controller class/constructor stay partial.
