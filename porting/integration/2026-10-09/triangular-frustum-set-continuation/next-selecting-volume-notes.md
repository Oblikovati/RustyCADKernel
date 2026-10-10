# Next port: SelectMgr_SelectingVolumeManager

Inspected against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. No concrete manager
source has been added. Continue in `/tmp/rustycad-resume`, branch
`porting/resume-claude`, after preserving the triangular-frustum-set checkpoint.
The original goal remains 100% of the unchanged 76,041 defined APIs.

## Available foundations

All concrete selectors are now ported: AxisIntersector, RectangularFrustum,
TriangularFrustum and TriangularFrustumSet. `SelectMgrBaseIntersector` holds the
shared virtual interface. `SelectMgrViewClipRange` handles plane chains and native
clipped/unclipped ranges. `Graphic3dSequenceOfHClipPlane` owns shared plane handles.
The existing `SelectBasicsSelectingVolumeManager` trait is the sensitive-entity
interface, but there is no concrete `SelectMgrSelectingVolumeManager` yet.
The native manager is a value class, not a Standard_Transient; do not add RTTI or
force it into the handle hierarchy.

## Required error propagation

The existing SelectBasics trait returns plain values for DistToGeometryCenter,
DetectedPoint, GetNearPickedPnt, GetFarPickedPnt and GetViewRayDirection. The concrete
manager delegates to intersectors which can throw, so those methods need Result.
Do not swallow failures with defaults or panics. Propagate into Select3D match
implementations and the Select3DSensitiveSet::distance_to_cog seam, which currently
returns f64. Relevant callers include sensitive_box, sphere, segment, curve,
triangle, triangulation, poly, group, wire, interior_point_set and primitive_array_match.
Update named fake managers and their tests. Existing overlap methods already return
Result. Other getters (selection type, mouse position, overlap permission, scalable,
planes) can retain their existing infallible contracts.

## State and initialization

The constructor starts with null active volume/clip handles and overlap false,
despite its header saying it creates all volume types. Each InitPoint/InitBox reuses
an existing rectangular instance (including crossing point/box modes), otherwise
allocates a fresh one. InitPolyline reuses only a frustum set; it assigns the active
handle and then copies IsOverlapAllowed into the child's permission flag. InitAxis
reuses only an axis. InitSelectingVolume accepts any BaseIntersector handle, including
null, without resetting clipping or overlap state. Build and configuration setters
assert that active volume exists. WindowSize with no active volume leaves caller
outputs unchanged (Option is appropriate). Camera with no volume returns null.
Deprecated Build overloads perform the corresponding Init then Build, preserving
configuration on reused volume instances.

AllowOverlapDetection changes only the manager flag; it does not immediately update
an existing frustum set. IsOverlapAllowed is that flag OR active selection == Point.
This can make parent and child permissions differ until another InitPolyline.
Test this explicitly instead of "fixing" the native behavior.

## Copy and transform

ScaleAndTransform with no active volume returns a fresh default manager, discarding
other state. With a volume it delegates, then copies overlap and both clip-plane
handles and clip ranges. CopyWithBuilder copies those fields even with no active
volume; with one, it copies the intersector and immediately calls Build on the copy.
It therefore differs from direct intersector copies, which await reconstruction.
Default value copies share handles and copy the range value. Honor the native custom
intersector interface rather than switching only on known concrete types.

## Queries and clipping

Every overlap returns false with no active volume, preserving pick and inside outputs.
Pick forms pass myViewClipRange; inside-pointer forms do not. DistToGeometryCenter
returns RealLast with no volume; DetectedPoint instead asserts. No-volume near/far/ray
getters return origin, RealLast and +X respectively; mouse position returns RealLast.
GetPlanes clears when empty and otherwise delegates (triangles append, rectangles
and sets replace). GetVertices uses a native unchecked static_cast to Rectangular;
only invoke the native probe for actual rectangular volumes. Choose and document a
safe Rust result for wrong volume kinds rather than reading unrelated storage.

SetViewClipping(view,obj,world?) first stores both handles. If active selection is
not Point, it returns WITHOUT clearing the old range. For Point it resets ranges,
then adds nonempty view and object sequences in that order. The picking ray comes
from world if provided, otherwise self. It does not validate world's selection type;
only access the world volume when a nonempty sequence needs a ray. Preserve getter
failures and partial range writes. A null world active handle is a native invalid
access; provide an explicit Rust failure if needed. The other SetViewClipping overload
copies all three clip fields verbatim. SetViewClipRanges copies just the range.
DumpJson prints only active/clip handle pointers, the range by depth, and overlap flag.

## Differential and integration checks

Add a stateful native/Rust manager fixture that exercises volume switching, reuse,
configuration before initialization, custom selectors, copied/rebuilt managers,
identity-form reset, point/box/polyline/axis overlaps with seeded outputs, late
permission changes, clipped range retention outside Point, view/object chains and
an independent world-space manager. Canonicalize only pointer values and separately
check handle identity and sharing. Include an actual sensitive-entity Matches path
to prove the updated fallible trait is wired through its consumers. Re-run existing
TKV3d tests and all native cases; refresh rustdoc, mappings, snapshot and metrics
only after source changes settle. Do not mark global coverage complete at this
checkpoint; Git metadata remains read-only, so preserve recovery patches and all
unfinished worktrees until integration can finish.
