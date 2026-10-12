# Martensite camera animation subclass protocol

`AisAnimationCameraSource: AisAnimationSource` represents a real derived camera
animation. Its embedded `AisAnimationCameraBase` owns the inherited timeline and
an `Arc<SharedState<AisAnimationCameraFields>>` containing the retained target,
start and end camera handles. The concrete `AisAnimationCamera` uses this same
base. Native RTTI projections preserve camera and animation ancestry using the same
concrete descriptors for Source and concrete handles. Pointer-based `IsKind` is
verified against native `STANDARD_TYPE`, alongside registered interface casts.
The former concrete `.fields` member now lives at
`animation.camera_animation_base().fields`; this avoids duplicating base storage
and preserves valid derived/base layout ordering.

A derived implementation returns its actual base from `camera_animation_base`
and the same timeline from `animation_base`. Its `update_own` delegates to
`ais_animation_camera_update(self, progress)` when inheriting the native camera
atom. Overrides of `Start`, `Update`, `UpdateTimer` and the own-frame hook remain
real `AisAnimationSource` virtual dispatch.

`AisAnimationCameraMembers::target_view_ref`, `camera_start_ref` and
`camera_end_ref` return live field aliases. They observe setter replacements and
null assignments without retaining an animation target or a lock. Explicit
`clone_handle` and copied convenience getters acquire ownership. `with_object`
uses a storage observer with unchanged native reference counts and releases the
field lock before invoking callbacks. Camera interpolation rereads the target
slot after `SetCamera` callbacks, and restores immediate update only on success.

`WeakHandleSlotRef` is a read-only live observer of independently owned slots.
`WeakHandleSlotAssignment` is a separate Rust callback capability for actual slot
assignment; it does not change the native const-reference contract. Both keep
owner storage and targets weak. Expired owners produce null observations or a
specific assignment failure; native dangling references are outside valid use.

This is a dependency checkpoint. Controller frame/XR integration remains in its
unfinished worktree; no API completion or whole-class restoration is credited
here. Cube migration can retain `Handle<dyn AisAnimationCameraSource>` and use
the nonvirtual members without copying the native source object.

The seven-row native fixture at
`crates/tkv3d/tests/fixtures/animation-camera-source-native.txt` verifies derived
timer dispatch, failure/retry, RTTI, endpoint alias identity and native ownership.
The provenance JSON records OCCT source hashes, revision, compiler, binary and
two equal raw-run hashes. Values use 17 digits without canonicalization.

```bash
OCCT_NATIVE=../RustyCADKernel/target/occt-all-full
g++ -std=c++17 -O0 -I"$OCCT_NATIVE/include/opencascade" \
  tools/oracle/martensite_animation_camera_source.cxx \
  -L"$OCCT_NATIVE/lin64/gcc/libi" -Wl,-rpath,"$(realpath "$OCCT_NATIVE/lin64/gcc/libi")" \
  -lTKV3d -lTKG3d -lTKService -lTKMath -lTKernel -o /tmp/martensite-camera-protocol
/tmp/martensite-camera-protocol
cargo test -p rustycad-tkv3d --test toolkit ais_animation_camera
```
