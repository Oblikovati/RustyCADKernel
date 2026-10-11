# Martensite retained camera rotation

Martensite implements the complete pinned `AIS_ViewController::handleOrbitRotation`
and `handleViewRotation` operations. For example,
`controller.handle_orbit_rotation(&view, &pivot, true)?` applies locked-Z orbit
rotation; `controller.handle_view_rotation(&view, yaw, pitch, roll, true)?` applies
first-person rotation with restart on increments. Both operate on real `V3dView`
cameras, windows and retained controller state.

The shared state preserves the orbit pivot, initial up/direction/eye/center and
current roll across calls and failures. Orbit start captures state and invalidates
before any animation stop. View rotation updates restart flags, cursor targets,
start vectors and roll before animation stop. Locked orbit and view rotation
preserve native pitch limits, angular thresholds and transform multiplication
order. Free orbit restores the initial camera before computing its axes and keeps
native repeated angle wrapping. View rotation removes old roll before applying new
roll. Orbit selects the active XR base camera and synchronizes base to posed;
view rotation acts on the actual view camera and has no final XR synchronization.

OCCT revision `3d097a0` does not initialize `myCurrentRollAngle` in its constructor.
Martensite chooses deterministic zero. Oracle cases explicitly seed prior roll
`0.21` and valid retained starts; they do not rely on undefined native memory.
The controller and constructor remain partial and unclaimed by this slice.

The CPU oracle has 7,008 committed 202-column snapshots: 6,784 ordinary states,
64 rejected-child-stop/retry states, 80 mixed orbit→view→orbit states, and 32
states with degenerate retained axes, plus 48 passive matching-roll states proving
empty/start-only request guards. The latter preserve actual native partial
camera/roll changes and demonstrate restart recovery. All five projections,
start/rotate flags, gates, locked/free orbit, active XR, large wrapped angles,
pitch limits, tiny angle thresholds and roll increments are covered. Native
headset I/O is a named deterministic session; retained cameras and synchronization
are real. Rust replay uses the assets without OCCT or a GPU.

Rows contain case/frame, failure/invalidation/animation flags, current/base/posed
cameras, the 16 retained rotation values, next-frame state, and every UI/GL request
field. Optional camera slots are zero padded. Roll, flags, queue coordinates and
queues are compared exactly; camera/axis arithmetic has a tight tolerance. Full
seeding and SHA-256 provenance are in
`porting/runtime-evidence/controller-camera-rotation-2026-10-12`.

Run tests with `cargo test -p rustycad-tkv3d --all-targets --locked`.
For local native regeneration, reuse the already built pinned OCCT libraries:

```sh
mkdir -p /tmp/martensite-rotation-oracle
python3 -c 'from pathlib import Path; p=Path("tools/oracle/martensite_controller_camera_primitives.cxx"); Path("/tmp/martensite-rotation-oracle/martensite_camera_primitives_oracle_shared.cxx").write_text(p.read_text().replace("int main(", "int martensite_unused_camera_primitives_main(")); p=Path("tools/oracle/martensite_navigation_motion.cxx"); Path("/tmp/martensite-rotation-oracle/martensite_navigation_motion_oracle_shared.cxx").write_text(p.read_text().replace("int main(", "int martensite_unused_navigation_motion_main("))'
c++ -std=c++17 -O0 -Wno-return-type -I/tmp/martensite-rotation-oracle -Itools/oracle -I../RustyCADKernel/target/occt-all-full/include/opencascade tools/oracle/martensite_camera_rotation.cxx -L../RustyCADKernel/target/occt-all-full/lin64/gcc/libi -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-all-full/lin64/gcc/libi -lTKV3d -lTKService -lTKMath -lTKernel -o /tmp/martensite-rotation-oracle/probe
/tmp/martensite-rotation-oracle/probe > /tmp/martensite-rotation-oracle/main.txt
/tmp/martensite-rotation-oracle/probe failure > /tmp/martensite-rotation-oracle/failure.txt
/tmp/martensite-rotation-oracle/probe mixed > /tmp/martensite-rotation-oracle/mixed.txt
/tmp/martensite-rotation-oracle/probe degenerate > /tmp/martensite-rotation-oracle/degenerate.txt
```

The temporary shared inputs rename only the older oracle entry points. Existing
helper definitions remain unchanged and are pinned by their source hashes.
`handleCameraActions`, `contextLazyMoveTo`, navigation hotkeys, animated orientation
and frame orchestration remain separate work. Windows/macOS testing is deferred.
