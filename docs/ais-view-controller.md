# Martensite AIS view controller

Martensite now ports 66 public `AIS_ViewController` methods at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: input-buffer access, public input
policy getters/setters, selection queue editing, zoom/wheel coalescing, touch
Z-rotation accumulation, navigation policy and previous-highlight position.
The controller type and constructor remain partial in the ledger. Rendering
and camera dispatch and XR are still open porting work. Mouse-button/position,
3D-mouse dispatch and [window-listener integration](ais-view-controller-window-listener.md)
are now ported.
[Animation ownership and click/touch dispatch](ais-view-controller-pointer.md)
now preserve native callbacks and gesture transitions.
[Keyboard navigation](ais-view-controller-keyboard.md) now adds 16 captured
keyboard, focus/reset and event-clock declarations using the shared listener base.

`AisViewController` owns independent UI and rendering buffers. Changing
navigation mode clears four UI rotation request flags, even when choosing the
same mode; it preserves coordinates, Z rotation and the rendering queue.
Selection of one point accumulates picking points and does not set the apply
flag. Sequence selection snapshots ownership, sets the apply flag and chooses
picking/rectangle/polygon for one/two/other lengths. Empty sequences choose the
polygon tool. Rectangle/polygon/single-pick edits retain an existing apply flag.
Polygon append ignores a duplicate last point; replacement changes the last
vertex, with an empty polygon accepting its first vertex in either mode.

Zoom actions merge only with the immediately preceding action at the same
pixel. The sum retains the first action's modifiers, and a merged zero delta
remains queued. Wheel input multiplies the delta by 15. These queue methods do
not check the allow-zoom policy; the later camera dispatch owns that check.
Touch Z rotation checks its allow flag, replaces a stale angle when starting,
then accumulates while the request remains pending. Disabling the flag preserves
pending state. Reset acknowledges the request while retaining its old angle;
the next accepted rotation starts a fresh angle.

```rust
use rustycad_tkv3d::ais::{AisViewController, AisViewInputBufferType};
let mut controller = AisViewController::new()?;
controller.set_allow_touch_z_rotation(true);
assert!(controller.update_z_rotation(0.5));
assert!(!controller.update_z_rotation(-0.25));
assert_eq!(controller.input_buffer(AisViewInputBufferType::Ui).z_rotate.angle, 0.25);
```

The public policy setters preserve native values without clamping. The native
previous-highlight sentinel is `(-1,-1)`. A local-only native generator and its
80 committed snapshots preserve these behaviors. Rust tests replay them without
OCCT headers, libraries or a native build. Run the toolkit tests with
`cargo test -p rustycad-tkv3d --all-targets --locked`. Fixture and generator hashes
are recorded in `crates/tkv3d/tests/fixtures/view-controller-events-provenance.json`.

The local generator requires the pinned native visualization build only during
porting. Regenerate it with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-all-full/include/opencascade \
  tools/oracle/martensite_view_controller_events.cxx \
  -Ltarget/occt-all-full/lin64/gcc/libi -lTKV3d -lTKService -lTKMath -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-all-full/lin64/gcc/libi" \
  -o target/martensite-view-controller-events-oracle
./target/martensite-view-controller-events-oracle > \
  crates/tkv3d/tests/fixtures/view-controller-events-native.txt
```
