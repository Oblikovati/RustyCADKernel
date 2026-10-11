# Martensite navigation motion

Martensite implements the complete pinned `AIS_ViewController::handleNavigationKeys`
operation with actual retained `V3dView` cameras and scene geometry. Call
`controller.handle_navigation_keys(&context, &view)?` from the rendering thread.
The context is unused upstream; the view is dereferenced only when jumping or
applying nonempty, unblocked camera motion. Failures preserve earlier key fetch,
clock, thrust and next-frame changes.

The implementation preserves absolute walking units, largest-axis scene-relative
orbit/flight speed, void-scene fallback, flight run acceleration, pressure-scaled
translation, non-normalized projected forward, and pressure-independent orbit
rotation. Pitch recomputes the camera side after yaw. Orthographic forward motion
calls the actual integrated zoom operation before translation, including animation
stop failures. Active XR selects the real base camera and synchronizes base to posed
at the native point; zoom's separate posed-to-base synchronization remains intact.
Ordinary perspective walking and rotation leave camera animation running.

The local native oracle produces 8,640 full-state snapshots: 2,880 cases with held,
released and consumed key frames. It uses real displayed AIS point clouds,
retained native scene layers, actual cameras, a named deterministic headset I/O
session, and stopped event timers. Another 21 snapshots prove rejected animation
child stop and retry across orthographic/perspective, active XR and policy guards.
Rust replay loads only committed numeric assets and does not require OCCT.

Each 244-column row contains case/frame, six walk parts (value/pressure/duration),
seven walk queries, invalidation/animation flags, current/base/posed cameras (21
fields each with optional-presence flags), next-frame/thrust/times/modifiers,
held/free state for all 15 relevant keys, and all 57 UI plus 57 GL request fields.
Camera fields include center/eye/up/direction, scale/distance/stereo focus/type,
aspect/projection/depth range/FOV. Request fields reuse the complete pinned transfer
schema and are compared exactly. Camera arithmetic uses a tight relative tolerance.

Run the Rust replay and toolkit suite with:

```sh
cargo test -p rustycad-tkv3d --all-targets --locked
```

To regenerate locally, reuse the existing pinned OCCT libraries. The shared source
below renames only the previous oracle's unused entry point; all helper definitions
remain byte-for-byte unchanged. It is temporary build input, not a new dependency.

```sh
mkdir -p /tmp/martensite-navigation-oracle
python3 -c 'from pathlib import Path; p=Path("tools/oracle/martensite_controller_camera_primitives.cxx"); Path("/tmp/martensite-navigation-oracle/martensite_camera_primitives_oracle_shared.cxx").write_text(p.read_text().replace("int main(", "int martensite_unused_camera_primitives_main("))'
c++ -std=c++17 -O0 -Wno-return-type -I/tmp/martensite-navigation-oracle -Itools/oracle -I../RustyCADKernel/target/occt-all-full/include/opencascade tools/oracle/martensite_navigation_motion.cxx -L../RustyCADKernel/target/occt-all-full/lin64/gcc/libi -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-all-full/lin64/gcc/libi -lTKV3d -lTKService -lTKMath -lTKernel -o /tmp/martensite-navigation-oracle/probe
/tmp/martensite-navigation-oracle/probe > /tmp/martensite-navigation-oracle/navigation.txt
/tmp/martensite-navigation-oracle/probe failure > /tmp/martensite-navigation-oracle/failure.txt
```

The scoped ledger claims only the visible navigation method. The controller and
constructor remain partial. Orbit gesture handling, navigation hotkey expansion,
animated orientation and full frame orchestration remain separate slices.
Windows/macOS testing remains deferred.
