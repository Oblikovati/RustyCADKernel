# Martensite ViewCube interaction

`AisViewCube::new()` now creates the concrete geometry, selection, immediate
highlight and camera-navigation widget. Attach it to an interactive context,
whose default drawer provides the datum and boundary aspects used by the
presentation. Normal context display activates its 26 actual selectable parts:

```rust,ignore
let cube = AisViewCube::new()?;
context.display(&cube.cast(), false)?;
cube.set_fixed_animation_loop(false);
// Applications using their event loop advance the real camera timer explicitly.
cube.update_animation(true)?;
```

The constructor retains the upstream Topmost triedron persistence, default
labels and geometry, stopped half-second camera animation, two camera handles,
and cyan unlit dynamic drawer linked to the object's drawer. The existing
embeddable presentation contract remains available to application subclasses.
`AisViewCubeSource` adds dispatchable interaction methods; application observers
can receive continuing-frame and completion notifications through
`set_animation_events`. The completion notification happens before detaching
its view. An absent observer reproduces the native base class's empty hooks.

Each side, edge and corner owns real indexed primitive-array sensitivity with
priority five and sensitivity factors two, four and eight. Disabled parts do
not acquire owners. Selection is point-only and rejects rays within ten degrees
of a part tangent. An inherited connected copy deliberately returns the base
primitive-array type, as upstream, without the ViewCube ray guard. Immediate
highlighting submits the picked part's actual triangles and keeps persistence,
view affinity, style and layer identity. Native `ClearSelected` and
`HilightSelected` deliberately leave global selection inactive; these are
faithful upstream policies, with explicit regressions, rather than unfinished
selection callbacks.

Owner clicks ignore mouse-button, modifier and double-click inputs, as native.
Automatic start can be disabled; fixed looping can be replaced with event-loop
stepping. The target preserves the closest quarter-turn up direction unless
reset is requested, fits selected geometry with scene fallback, and leaves an
empty scene unchanged. Original view-camera identity and immediate-update
policy are restored after target preparation. The protected starting and ending
camera handles are copied in place and passed directly to the real animation.
A missing interactive context or required animation/triangle array returns a
typed failure instead of following a native null dereference. An attached
context without a last active view, or a null owner, leaves animation unchanged.
Scene-only fitting does not introduce a context dependency.

The proof uses OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`
and the real locally compiled TKV3d/TKService/TKMath/TKernel libraries. The CPU
oracle creates a context-linked native drawer, actual sensitive entities,
actual presentations and camera animations. Its named seekable animation
subclass calls the native timer/update implementation at deterministic
timestamps; it does not replace camera interpolation. Two executions produced
identical 1,808 records. Rust regression replay uses only committed snapshots,
with no native libraries, sleeping, window system or GPU requirement.

The six snapshot files cover 58 sensitive entities under three geometry
policies, ten ray guard/matching cases, twelve immediate highlight cases, three
click policies, and 156 orientation/scene/projection configurations. Each camera
configuration records target/handle identity and four real animation frames and
completion transitions. Provenance and compiler bindings are in
`porting/runtime-evidence/view-cube-interaction/`.

The three ledger fragments contain 43 implemented declarations and three
**partial class records**, all pinned API-visible public or protected entries.
They do not repeat the nine presentation-core declarations. Whole-class
completion is not claimed: ViewCube appearance setters, reset/unset style
management and remaining protected geometry/appearance storage are outside this
slice. Owner and sensitive records retain partial status because their complete
native declaration surface, including generated type aliases/RTTI entry points,
has not received exhaustive compiler mapping and native proof. The runtime
names and parent chains themselves have Rust regressions. Renderer device
capture and performance proof are maintained by the renderer work; this CPU
slice changes no renderer submission interface and claims no additional GPU
coverage.

Run the functional suite with `cargo test --locked -p rustycad-tkv3d --all-targets`.
