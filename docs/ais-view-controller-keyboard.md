# Martensite AIS keyboard navigation

Martensite now ports 16 more captured `AIS_ViewController` declarations at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: keyboard forwarding, navigation
fetch, focus/reset, native empty window callbacks, previous/current event times,
next-frame requests and inherited key/event-time exports. The controller owns
the existing `AspectWindowInputListenerBase`; all keyboard accessors share its
cache. `with_event_timer` injects the clock used by both event timing and click
bookkeeping. [Click/touch dispatch and animation ownership](ais-view-controller-pointer.md)
are now ported, as are mouse-button/position and 3D-mouse dispatch.
[The complete window-listener trait](ais-view-controller-window-listener.md)
routes these paths. Camera/render dispatch and XR remain unported.
The controller record and constructor remain partial in the inventory.

Navigation fetch advances the previous/current event-time interval. Held or
just-released keys define a component, even when its resulting value is zero.
A released key contributes once, then becomes free. Repeated presses retain the
initial press time while updating pressure. Jump explicitly releases its key;
the next fetch consumes that released state, so it can also report jumping once
before becoming free. Jump takes precedence over crouching. Crouching suppresses
running, and the run/crouch ratio scales horizontal translation only.

Forward/backward values add. Side, vertical, yaw, pitch and roll values overwrite
in native check order when both directions are held. Translation stores pressure
without multiplying displacement; rotations multiply by pressure. Component
duration retains the full held/released duration while progress uses
`abs(std::min(frame_interval, key_duration))`. The ordered comparison preserves
the native NaN behavior rather than using Rust's NaN-ignoring `f64::min`.
Negative, zero and nonfinite ratios remain unclamped. Pitch inversion changes
the two pitch signs without changing other components.

Nonzero thrust first marks thrust active and requests another frame. Later fetches
add speed times frame interval to forward displacement. Zero or NaN stops active
thrust without clearing an existing next-frame request; applications acknowledge
that request with `set_ask_next_frame(false)`. A resumed thrust starts without a
thrust displacement again. Held navigation keys still contribute on its first
frame. The threshold is the native `gp::Resolution()` (smallest positive normal
double), so even a nonzero subnormal float speed is active.

Focus activation preserves input. Focus loss clears keys and mouse buttons/flags,
resets click/gesture bookkeeping, stops the click timer without zeroing elapsed
time, and requests drag abort in the UI buffer. It preserves other pending
requests, selection points, zoom actions, the rendering queue and navigation
bookkeeping. Expose/configure/input/close callbacks have the upstream empty bodies;
they preserve queued gestures until explicit flushing.

```rust
use rustycad_tkv3d::ais::AisViewController;
use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic};
let mut controller = AisViewController::new()?;
let key = AspectVKeyBasic::NavForward as AspectVKey;
controller.key_down(key, controller.event_time(), 0.5)?;
let movement = controller.fetch_navigation_keys(0.5, 2.0)?;
assert!(movement.is_defined());
# Ok::<(), rustycad_tkernel::standard::Failure>(())
```

Sixteen new Rust regressions replay 206 committed frozen-time native snapshots,
including all axes, opposite keys, pressure/duration, held/released key lifecycles,
stance combinations, thrust transitions, signed zeros and NaN/infinities. Named
injected clocks also verify click-timer stop and safe error forwarding. No test
sleeps, native compiler, OCCT headers or OCCT runtime are needed in CI. Run
`cargo test -p rustycad-tkv3d --all-targets --locked`.

The local-only generator is `tools/oracle/martensite_view_controller_keys.cxx`.
It uses the pinned headers and TKV3d/TKService/TKMath/TKernel libraries, like the
[event-queue generator](ais-view-controller.md). Native event timers are reset to
stopped, exact values, preserving the full native fetch path without patching it.
Fixture, generator, header and implementation hashes are recorded beside the asset.
