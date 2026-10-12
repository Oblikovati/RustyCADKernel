# Martensite tracked XR device

The captured defined-visible `AIS_XRTrackedDevice` family has nineteen native
bindings at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`: two constructors,
eight role/unit/laser members, four protected presentation/selection hooks,
three RTTI members, the base alias and the class record. The class record's
canonical compiler source is its forward site in `AIS_ViewController.hxx`;
its actual definition site in `AIS_XRTrackedDevice.hxx` is also retained.
Private fields/nested texture helpers receive no API credits. Broader inherited
AIS, selection, texture and rendering families are not declared complete here.

`AisXrTrackedDeviceSource` borrows one real `AisXrTrackedDeviceBase` and inherits
the interactive/presentable/selectable interfaces. Concrete construction binds
the final owning object. Application wrappers can embed the actual core and
retain inherited native descriptor identity separately from their Rust caster
metadata. Real virtual Compute, ComputeSelection and AcceptDisplayMode calls
reach application overrides; qualified common functions execute the native
base bodies on that same state. `AisXrTrackedDeviceParent` is the actual
interactive base alias. Nonvirtual properties and computeLaserRay reside in
blanket Members so overrides cannot intercept native setter refreshes.

The empty constructor shows axes. The model constructor retains triangle-array
identity, initializes native default material/white shading, and optionally
retains an Image_Texture source. A private holder forwards the exact supported
format handle and image result/failure, snapshots a nonempty texture identity,
and otherwise retains a freshly generated TextureRoot identity. The bounded
Texture2D prerequisite fixes an extra same-name source-interface ancestor:
Texture2D/source share the native descriptor, its parent is TextureMap, and
Texture2Dplane's parent is Texture2D. Real base projections remain available.

Role and unit setters store their values without invalidating managed mode
caches. Unit scale affects newly computed axes, not mesh or laser coordinates.
Color uses native Quantity tolerance; equal colors leave the stored value
unchanged. Length uses native float inequality, including NaN being unequal to
itself. Both changed setters assign before synchronously computing the ray.
Live color references observe later assignments, including those preceding a
checked callback failure; independent copied colors preserve old values.

Only mode zero computes geometry or selection. Compute assigns the infinite
flag, creates the main group, submits a retained mesh or fallback axes, then
creates/stores the new ray group and refreshes it. Failure preserves prior
writes/groups and the previous ray pointer until replacement actually succeeds.
Laser arrays explicitly disable min/max evaluation, keeping them out of camera
bounds. Native IsEmpty is bounds-based: a finite excluded-bounds ray group stays
empty and successive positive/color/NaN refreshes accumulate arrays. Zero or
negative length does not erase those finite rays. Infinite groups report
nonempty even without primitives, so refresh clears them before length checks
or later callback failure. This behavior is deliberately preserved.

Selection appends a SensitivePrimitiveArray with a real owner referring to the
actual most-derived object. It ignores InitTriangulation's boolean result, so
empty/incomplete nonnull arrays still add a sensitivity. Axes and laser never
enter selection. Existing sensitivities share original vertex/index buffers;
model mutation affects precise picking while retained BVH/center state follows
native staleness, and releasing the caller's mesh handle preserves device-owned
geometry. Repeated selection calls append independent sensitivities.

Six final local native generators repeated byte-identically: 960 presentation,
60 actual-volume selection/model-mutation, twelve texture/RTTI, eight retained
color/failure/repair, eight genuine managed-cache, and ten derived-dispatch rows
(1,058 total). Rust regressions use committed bounded fixtures with no native
compiler, OCCT, graphics device or external I/O. A separate native Texture2D
fixture proves the prerequisite. Typed Failure transport preserves partial
state and resumes unrelated Rust panic payloads. Checks and canonical source
hashes are recorded in the runtime receipt.

Native null dereferences, dangling references, invalid enum values, unchecked
invalid native array indices and concurrent data races remain undefined and
receive no parity claims. Valid reference cases retain their owning device.
