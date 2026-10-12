# Martensite ViewCube camera-animation subclasses

The widget retains `Handle<dyn AisAnimationCameraSource>`, allowing native-style
camera subclasses to override `Start`, `Update`, `StartTimer`, `UpdateTimer`,
`Stop` and the camera atom. Subclasses embed the actual `AisAnimationCameraBase`
and delegate inherited interpolation to `ais_animation_camera_update`; the
[owned camera protocol](ais-animation-camera-source.md) describes that base.
The actual rendering fixture installs `CubeWidgetAnimation`, a real derived
camera atom, before consuming a detected part's click.

```rust,ignore
let derived: Handle<MyCameraAnimation> = make_camera_animation();
cube.set_view_animation(&derived.cast());
let alias = cube.view_animation_ref();
// The borrowed field remains live through replacement, including null.
cube.set_view_animation(&Handle::default());
assert!(alias.is_null());
cube.set_view_animation(&derived.cast());
```

`view_animation()` is an explicit owning snapshot convenience. Native
`ViewAnimation` binds the borrowed live slot; callback access uses a storage
lease and releases its field lock before virtual dispatch. Neither the alias
nor its lease increments the intrusive native reference count. Animation
setters retain their targets normally.

Every native statement rereads the widget slot. Replacing A with B during
`UpdateTimer` makes the following duration comparison and `Stop` use B.
Replacing B with C during `Stop` makes the finish hook and later view detachment
use C. Continuing-frame callbacks run only after successful update/redraw;
completion hooks run before detachment. Public `UpdateAnimation` retains the
original target-view copy for its redraw, as native does.

The protected virtual `view_fit_all` receives genuine `HandleReference` inputs,
covering both the local view handle and the actual protected endpoint slot.
Replacing `myEndState` inside that callback changes its retained argument;
subsequent endpoint assignment and animation startup use the replacement.
These references distinguish slot identity from target identity and provide
explicit owning copies separately from noncounting storage leases.

Typed native failures stop the remaining operations without rolling back native
side effects. Owner clicks return true only after `HandleClick` succeeds;
failures cross the boolean hook through `raise_native_failure` and emerge from
`select_detected` as checked failures. Unexpected Rust panics remain panics.
Null dereferences or dangling native references are outside the tested domain.
Native oracle cases retain original strong owners and inspect null only through
valid handle operations, restoring the slot before native dereference.

The Rust-only replay uses 198 committed native records: 144 derived animation
lifecycle/slot/camera rows, six owner/context failure rows and 48 virtual fitting
alias/count/replacement/failure rows. Existing seven animation-reference rows
remain applicable. Animation/view/endpoint/owner/context/widget counts and event order come from real OCCT at
`3d097a0328e71b826377d4814ab05ec3c3d23871`; replay links only Rust.
Whole ViewCube/owner/sensitive class claims remain partial pending their separate
generated RTTI and base-alias obligations.

```sh
cargo test --locked -p rustycad-tkernel -p rustycad-tkv3d -p martensite-vulkan --all-targets
```

The actual widget capture and completed-frame performance examples retain
strict native color/depth failures. Pixel-row precision cases remain deferred;
no tolerance or baseline acceptance accompanies this transport work.
