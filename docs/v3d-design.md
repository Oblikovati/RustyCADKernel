# Martensite TKV3d design notes

This note fixes how the OCCT toolkit TKV3d maps to Martensite (`crates/tkv3d`, `rustycad-tkv3d`:
packages BRepFont, Select3D, SelectBasics, SelectMgr, Prs3d, StdPrs, StdSelect, DsgPrs, PrsDim,
PrsMgr, AIS and V3d). The rules of [geom-design.md](geom-design.md) sections 1 to 4,
[ocaf-design.md](ocaf-design.md) section 1 and [visualization-design.md](visualization-design.md)
apply: an abstract class is a trait over an embedded base, `Handle<dyn Trait>` holds it, the
fields that any handle may change sit in one `SharedState`, exceptions are `Failure` values with
the offending value, one snake-case module per package, the package prefix stays in each Rust
name. Code comments and ledger notes cite this note as "v3d-design.md section N".

## 1. Presentations and the driver

`Prs3d_Presentation` is a typedef of `Graphic3d_Structure`, so a presentation is a
`Handle<dyn Graphic3dStructureSource>`. Nothing in TKV3d talks to OpenGL: V3d and PrsMgr reach the
graphic driver only through the TKService traits (`Graphic3dGraphicDriver`, `Graphic3dCView`,
`Graphic3dCStructure`, `Graphic3dGroup`). The tests run on the TKService null driver
(`rustycad_tkservice::null_driver::NullGraphicDriver`); Martensite's new Vulkan 1.3 driver uses
the same traits. Its [implementation sequence](visualization-design.md#6-vulkan-implementation-sequence)
is active work under the current porting priorities.

## 2. Selecting volumes

`SelectBasics_SelectingVolumeManager` is the trait `SelectBasicsSelectingVolumeManager`. Its
overloads that differ only by the last argument get one name each: the pick-result form keeps the
native name (`overlaps_box`), the `bool* theInside` form takes the suffix `_inside` and an
`Option<&mut bool>` (`None` is the native null pointer), and the depth-less point test is
`overlaps_point_only`. The deprecated `Overlaps` aliases are the same trait methods.

## 3. Drawer link fallback

`Prs3d_Drawer` getters that fall back to the link drawer read their own fields, release the
lock and then ask the link. A drawer linked to itself therefore cannot deadlock, where the native
code recurses without end. The uninitialized native fields (`myPreviousDeviationCoefficient`,
`myPreviousDeviationAngle` until the first set, `Prs3d_ArrowAspect::myIsZoomable` in two
constructors) start at 0 and `true` in the port; their values are only observable through
`DumpJson` natively, where they are garbage.

## 4. Numeric probes

The numeric code (quadric tools, arrows, frustums, sensitive entities, view and camera
operations) is compared bit for bit with the native toolkit by
`tools/oracle/tkv3d_probe.cxx` and its Rust twin `crates/tkv3d/examples/numeric_probe.rs`
through `tools/verify_tkv3d_numeric.py`, on the request corpus of `tools/tkv3d_corpus.py`.
The native float steps of `Prs3d_ToolQuadric::FillArray` (`1.0f / n`) are kept.

The V3d view requests run a viewer on a null driver on both sides: the Rust probe uses the
TKService `NullGraphicDriver`, the native probe the `SvDriver` of
`tools/oracle/tkservice_probe_null_driver.cxx`. A view refers to its viewer without owning
it (native `V3d_View::MyViewer` is a raw pointer), so a caller keeps the viewer alive while
it uses the view; a view whose viewer is gone skips its redraw.

## 5. Presentable objects

`PrsMgr_PresentableObject` is abstract and its virtual members are overridden down to the
AIS objects, so it is the trait `PrsMgrPresentableObject` over an embedded
`PrsMgrPresentableObjectBase` that holds the protected fields in one `SharedState`. Each
virtual member has a default body that calls a `prs_mgr_*` function with the native base
body; an override calls that function where the native code calls
`PrsMgr_PresentableObject::Member`. The non-virtual members are the blanket traits
`PrsMgrPresentableObjectMembers` and `PrsMgrPresentableObjectUpdates`. A constructor calls
`prs_mgr_presentable_bind` once the handle exists, so the presentations and children receive
the object handle (`this`).

The raw back pointers (`myParent`, `PrsMgr_Presentation::myPresentableObject`) are weak
handles. The structure manager keeps strong handles to displayed structures, so a
presentation never relies on its destructor to leave the scene: the members that drop a
presentation (`Erase`, the deprecated `Update(mode, true)`, the object destructor) erase it
first. The virtual members of `Graphic3d_Structure` that return nothing natively
(`Compute`, `computeHLR`) report a failure of the object computation as a fail message.
Bounding boxes come from the driver structure, which a view computes when it displays the
structure: without a view the box stays void, as natively.

## 6. Selecting volume foundations

`SelectMgrFrustumBuilder` retains the camera handle and the native pixel-to-world
and viewport coordinate conventions. `SelectMgrViewClipRange` preserves the native
single-pass, insertion-order merging of clipping sub-ranges. Its
`get_nearest_depth_into` method exposes the output argument, including mutations
when the lookup returns false; `get_nearest_depth` is the convenience form for
callers that only need a successful depth. Void query ranges preserve the caller's
output value where native `Bnd_Range::GetMin` leaves it untouched.

These helpers are tested without a rendering backend. Their differential corpus
covers camera projection and depth modes, viewport invalidation, plane chains,
parallel and opposite rays, finite/void ranges, and JSON dumps. The original Claude
worktree is preserved while its recovered frustum core is validated here.

The base intersector is now a trait over `SelectMgrBaseIntersectorBase`. Its required
build, transform and overlap methods have no default geometry implementation. The
native concrete base bodies retain camera identity, return infinite sentinels, clear
planes and leave window outputs untouched. The default ray is +X, matching
`gp_Dir()` at OCCT 3d097a0; it does not assume a camera looking along Z.

The ray kernels preserve native arithmetic and intersection order, including
negative parameters, sphere roots ordered by magnitude, cylinder/cone tangency
rejection, and filled versus outlined circles. Optional results encode untouched
native output arguments: a single cylinder crossing has an entry but no exit.
The 933 additional native cases compare initialized output values on both success
and failure. A named fake exposes base defaults in Rust while rejecting abstract
geometry calls; native probes qualify base calls on a rectangular frustum.

`SelectMgrBaseFrustum` adds the shared builder and tolerance state. Concrete volumes
delegate the inherited virtual setters to `SelectMgrBaseFrustumBase`; its camera
setter invalidates the builder's viewport, while clearing the builder preserves
the current camera. Replacing a builder adopts its camera without changing the
pixel tolerance. Rust window/viewport setters do nothing with a null builder and
window queries return `None`; native code dereferences null for those invalid calls.

The boundary/sphere test retains OCCT's closed-boundary convention: it ignores the
last point entry and explicitly wraps to the first. A miss or edge crossing leaves
the caller's inside flag unchanged; a projected vertex inside the sphere sets it.
Collapsed projections preserve direction-construction failure. Ray intersection is
called virtually, including derived overrides. The differential dump uses a native
`ostringstream`, matching Rust's readable `DumpStream`: an opaque `cout` has a
different native separator rule and is not the same output-stream fixture.

`SelectMgrFrustumCore<const N: usize>` now implements the native template geometry
through composition with `SelectMgrBaseFrustumBase`. Protected arrays are exposed
as fixed-length immutable and mutable slices; their lengths retain the template
dimension. The concrete triangular, rectangular and polyline volumes compose this core
as described below.

The SAT and quadric routines retain native arithmetic and branch order. Triangle
tests preserve the output normal on early plane rejection and write it before late
axis rejection. Degenerate segments, side-plane-only sphere tests, hollow cylinders,
and the circle origin convention retain native behavior. The native and Rust probes
compare 4,620 additional cases across both side counts and camera modes, regular,
sheared and degenerate states, transforms and optional inside outputs. Ten focused
Rust tests exercise the same observable contracts.

The template dump includes all arrays, cached projections and inherited state at
each depth. Production dumps retain pointer values. Only the differential probe
canonicalizes process-local builder and camera addresses after checking non-null
builder and shared camera identity; numerical values are compared without tolerance.

`SelectMgrTriangularFrustum` now composes the template core and shared base state.
Build preserves native near/far vertex ordering, edge directions, unnormalized plane
normals and cached extrema, including signed-zero selection by `std::min/max`.
Init only stores input pixels. A transformed copy ignores the scale and builder
arguments, retains the source camera and pixels, and starts with a fresh builder and
default tolerance. CopyWithBuilder keeps only the pixels and adopts the new builder;
its geometry stays empty until Build. Clear releases only the builder. Building a
cleared Rust volume reports `Standard_NullObject` with the invalid state rather than
dereferencing null as native code does.

Triangular volume overlap calls preserve all pick fields and ignore clip ranges.
Inside-pointer overloads leave the inside flag untouched. The sphere overload with
an inside pointer tests the bounding box; the pick overload uses the sphere core.
The point-only overload returns false. Boundary polygon/triangle queries return
false even when every edge overlaps, while an unknown triangle sensitivity returns
true. GetPlanes appends five equations to the existing output. Native comparisons
cover these contracts, type ancestry, camera/viewport/depth configurations, reversed
and degenerate input triangles, affine and collapsed transforms, copied/rebuilt and
cleared volumes, and all dump depths: 12,352 additional cases and nine direct tests.

`SelectMgrAxisIntersector` now implements the selecting ray and every overlap
method. Init selects Point mode. Queries, copy/transform and geometric getters
preserve native initialization assertions, so the shared GetNearPnt, GetFarPnt and
GetViewRayDirection methods return `Result` even though the base defaults succeed.
Build and camera/window/viewport/tolerance configuration remain native no-ops.
Identity-form ScaleAndTransform returns an uninitialized default object; a numeric
identity matrix with Other form follows the normal transform branch instead.

The axis has no native RTTI declaration: its dynamic and static descriptors remain
BaseIntersector. Rust trait projection uses a separate caster descriptor, as described
in geom-design.md section 2. DumpJson still emits the explicit AxisIntersector class
name, independently of the inherited runtime type. This distinction is checked by
the native probe and a direct regression assertion.

Pick mutation follows native branch order. Linear misses invalidate depth, point and
normal while retaining distance to center; clipped point hits keep their writes.
Box picks update only depth. Polygon interior tests its plane without containment;
triangle interior includes degeneracy, parallel-edge and nearest-edge fallbacks.
Quadric depth stays in local coordinates even when point and normal are transformed.
Sphere root ordering may create reversed Bnd_Range bounds and raise a construction
failure; it is not replaced by a void range or reordered into an idealized interval.
Inside flags are refined only after successful hits. Protected helper misses retain
output values via optional results. Ten direct axis tests, three generic handle tests
and 22,213 new native comparisons cover these contracts and all dump depths.

`SelectMgrRectangularFrustum` now builds point and box volumes through the existing
frustum core. `SelectMgrSelectionRectangle` keeps its native unsorted bounds and
RealLast defaults. Init retains built geometry until Build. Point construction
uses the pixel tolerance; box construction uses the stored bounds. Orthographic
caching updates only even plane entries and preserves native operand order for
signed zero. GetPlanes replaces the output rather than appending to it.

Copies preserve the native branch order. ScaleAndTransform rejects negative
scales despite the upstream header suggesting them. Scale one plus Identity form
returns an uninitialized default with the supplied builder. Other transforms
retain selection pixels, compute a new depth scale from ray lengths, use default
pixel tolerance, and adopt the supplied builder's camera before caching. A numeric
identity matrix with Other form takes the normal transform branch. CopyWithBuilder
requires a non-null builder, retains tolerance and pixels, and starts with empty
geometry. Deep native dumps require a camera; the null-camera identity reset is
compared only at dump depth zero. Null-builder Rust projection reports a failure
instead of dereferencing null.

Protected segment picks clamp to the finite near/far ray. Plane picks update only
depth on success and invalidate on failure. Polygon interior also retains the old
picked point and normal, while triangle picks set them. Quadric picks try the
second root after clipping, write point and normal even after final clip rejection,
and rescale depth; box pick depths retain native local units. The shared native
triangle arithmetic and quadric normal helpers serve both rectangular and axis
intersectors. Rectangle GetMousePosition returns the base sentinel outside Point
mode; DetectedPoint requires Point mode. Native RTTI remains BaseFrustum while its
dump class is explicitly RectangularFrustum. Twelve direct Rust tests and 31,423
new native comparisons cover these contracts and the public planar predicates,
including native coordinate-substitution degeneracies.

`SelectMgrTriangularFrustumSet` now decomposes the input polygon using the existing
TKMesh Delaunay implementation, preserving frontier orientation, deduplicated mesh
nodes, and native map iteration order for child construction. It does not replace
the native decomposition with a fan. Input pixels retain their HArray1 identity:
Init resizes and assigns the existing shared array, so copies observe later pixel
edits when rebuilt. Native one-based winding and boundary expressions are retained;
incompatible lower bounds produce array failures. Build clears old children before
meshing. Empty/degenerate inputs retain their native empty results or failures.

Transforms preserve pixels and transformed boundaries but reset overlap permission;
copies with a builder preserve permission and child count while leaving child
geometry and outer boundaries unbuilt. Child triangular transforms retain their
original camera while the parent set adopts the caller's builder camera. Native
RTTI remains BaseFrustum, with separate Rust interface casters. GetPlanes clears
once and appends each child's equations; dumps preserve child order and explicit
class names. Default destruction releases fields without an artificial callable
Rust destructor.

Point-only selection uses child SAT directly, since the triangular child's virtual
point-only overload always returns false. Full-containment box checks refine the
inside flag from all eight corners but still return true on partial overlap.
Polygon, segment and triangle matches reject outer-boundary crossings unless
partial overlap is allowed. Pick outputs and clipping remain governed by the
native child methods. Sphere containment retains its native projection and final
near-vertex/far-vertex angle pairing. Cylinder containment checks six projected
silhouette points. Circle boundary tests use the actual GeomIntIntSS plane/plane
intersection and line-normal arithmetic; unused native circle construction still
raises for invalid radii. Empty cylinder/circle queries preserve List::First's
Standard_NoSuchObject failure. DetectedPoint always raises Standard_ProgramError.

Twelve direct Rust tests and 18,163 additional exact native comparisons cover these
contracts, including concave/reversed/repeated/short polylines, shared-array copies,
unbuilt copies, affine and collapsed transforms, optional inside flags, seeded
pick fields, circle-boundary crossings and all dump depths. TKMesh and TKGeomAlgo
are now direct TKV3d dependencies through their existing project-owned interfaces.

`SelectMgrSelectingVolumeManager` is a value class with shared active-volume and
clip-plane handles. Point/box initialization reuses rectangular volumes; axis and
polyline modes reuse only their own concrete types. Initialization retains built
geometry until Build. Changing manager overlap permission updates a polyline's
permission only on its next Init. CopyWithBuilder immediately builds the copied
intersector, while ScaleAndTransform of an empty manager returns fresh defaults.
Clipping updates always replace plane handles, but only Point resets and recomputes
ranges, with view planes before object planes and an optional world-space ray.
The custom intersector interface remains virtual throughout query/copy operations.

SelectBasics distance and ray getters return `Result`, matching intersector failures;
Select3D matching and distance-to-center helpers propagate those failures, including
a failure after an overlap has already written pick output. Empty managers preserve
pick/inside outputs, use native sentinels, and reject initialization-dependent calls.
GetVertices safely returns None for a nonrectangular active volume, where the native
unchecked static_cast has undefined behavior. Nine manager state tests, a regression
through sensitive matching, and 4,800 native scenarios exercise these contracts.
The native fixture configures newly allocated volumes before geometric queries;
OCCT otherwise dereferences a null camera. That undefined path is not parity evidence.

`SelectMgrToleranceMap` retains zero-frequency keys in the owned NCollection DataMap,
so decrementing a known inactive key raises while decrementing an unknown key does
nothing. Custom tolerance is a nonnegative offset to the adaptive maximum; with
no nonnegative active maximum, the returned default remains 2. Custom zero is
stored and used even though IsCustomTolSet reports false.

`SelectMgrSortCriterion` preserves the public candidate record and both native
comparison predicates. Depth ranking is not a total or lexicographic ordering:
higher display priority can win even when selection priority is lower. Normal
comparison first computes the native f32 modulus, then constructs f64 directions
and uses Angle followed by abs(cos), retaining threshold arithmetic and float
rounding. The caller chooses the predicate; the stored preference flag does not
change either predicate's body. Eight Rust tests and 29,826 native comparisons
cover count traces, underflow/overflow of normal components, confusion and
60-degree boundaries, tolerance signs, layers and priority inversions.

`SelectMgrSensitiveEntitySet` owns an indexed map, owner frequency map and shared
primitive BVH. Removal swaps the last sensitive into the removed slot. Duplicate
appends increase persistence/flipping counters without increasing owner counts;
removal uses current entity flags and owner, preserving native counter imbalance.
Rejected runtime types deactivate their wrappers. Selection-level append/remove
marks dirty even when empty, while assigning a builder does not.

Live map guards and retained BVH handles preserve ownership and observation. Native
runtime inheritance reaches BVH_ObjectTransient and Standard_Transient. Seven Rust
tests and 608 native traces exercise ordering, counters, owner changes, runtime
rejection, cached bounds, builder changes and retained tree contents. Fixtures keep
selections alive because their destructor clears sensitive owners. Dirty builds with
null builders are omitted from native comparisons because OCCT dereferences null;
Rust reports Standard_NullObject, covered by direct tests.

`SelectMgrSelectableObjectSet` maintains five indexed maps with independent dirty
flags and stable tree handles. Its regular builder has four bins (the native header
comment incorrectly says 32); persistent subsets use linear builders. Value copies
copy maps and flags but share trees and builders. Thus rebuilding a copied set can
change the original's tree without changing its indexed map, as in native OCCT.
The live iterator follows More/Next separately, observes map edits, and does not
revisit subsets it has already passed. The native count sentinel is exposed as
SelectMgrBvhSubset::COUNT; only the five valid subsets can index a Rust tree.

Regular bounds come from the virtual object query. Persistent bounds include valid
groups with persistence/flipping metadata, first flip using WorldView*ModelWorld
and then apply group persistence, before applying object persistence to the union.
The group-local transform is deliberately excluded. World-space persistent trees
observe all camera changes; screen-space trees observe only projection changes.
Orthographic subsets use a copied camera with forced projection. Empty subsets
retain stale trees, and null-camera updates still record viewport size. A failure
preserves the earlier completed regular update, persistent dirty flags and previous
view/viewport state. DumpJson ignores depth and keeps native separators/field order.

Twelve Rust tests and 542 native traces exercise these contracts, including copied
sets, copied/live cursors, failures from virtual persistence, group-local transforms,
void groups and camera changes. The headless driver omits renderer flip brackets,
so the fixture explicitly installs the same selection metadata as OpenGl_Group's
enable branch. The tests cover selection calculations, not renderer behavior.

### Background BVH workers

`SelectMgrBvhThreadPool` ports `SelectMgr_BVHThreadPool` at OCCT `3d097a0`.
It retains queued sensitive handles, starts workers only on a buildable entity,
and preserves pending jobs across stop/restart. `WaitThreads` observes both the
idle event and each worker's nonrecursive build mutex. The public worker array
contains the actual OSD wrappers: copies share native thread handles but get fresh
BVH mutexes, while assignment retains the destination mutex. An owning, noncopyable
sentry locks workers in array order. Its owned guards cannot cross threads.

`OsdThread` now accepts an owned callback in addition to an opaque function pointer.
Its interior state allows worker inspection while another caller joins; no wrapper
lock spans the join. Callback rebinding changes future runs without detaching a
copied running handle. Existing copy/detach quirks remain, including self-assignment
and rejecting self-join without consuming the handle. A typed worker argument
replaces reconstructing a C++ pointer from an integer. Signal operations and the
messenger are injectable; the default constructor uses the existing native host
and default messenger. Existing OSD limitations for converting hardware signals
into exceptions remain; this port does not claim that unsupported conversion.

The pinned native pool tests stop before acquiring the queue mutex. An empty worker
can consequently reset the wake event after shutdown sets it, stranding another
worker. Rust synchronizes the shutdown wake and empty-queue reset under the queue
mutex and rechecks stop there. A direct ordering regression and repeated restarts
cover that refinement. Tests also exercise parallel builds, idle-before-completion,
retained entity lifetimes, callback reentry, failure continuation, worker replacement,
copy/assignment, and sentry ownership. The unchanged native oracle also exhibited that hang. Its fixture parks all workers
in barrier-controlled BVH calls before shutdown; both probes then observe the native
stop and join with a defined wake ordering. The numeric corpus compares 180 completed-round
native traces, including duplicate jobs, failures, clamped counts and restarts.
Both implementations remove a job from the queue before acquiring its worker mutex,
so `WaitThreads` alone can miss that in-between job. The fixture observes completed
counts after all workers have entered parking jobs, rather than asserting a stronger
completion guarantee than the native algorithm provides;
message ordering is sorted only when multiple workers make it nondeterministic.
The two declared-but-undefined, explicitly prohibited native sentry copy operations
are recorded as exclusions rather than supplied with invented implementations.

### Viewer selector state and lifecycle

`SelectMgrViewerSelector` embeds a public `SelectMgrViewerSelectorBase`; derived
transient selectors implement `SelectMgrViewerSelectorSource` and can override the
native virtual `SortResult`. Nonvirtual extension traits handle configuration,
object/selection membership and results. Protected camera, tolerance, indexed owner
map, cached rank array, layer order and background builder fields remain accessible.
The actual selecting volume and object set are exposed, rather than detached copies.

Ranking preserves native invalidation: changing closest-first preference does not
invalidate a sorted result; clearing picks retains index storage; invalid ranks
are checked before virtual sorting. Picked criteria and entity handles are borrowed
through read guards, preserving their stored addresses without copying them. Release
these guards before mutating the selector. Sorting uses the shared libstdc++ introsort
translation, now fallible: a failed comparison propagates after the exact native
partial element moves, without unwinding through a lock. Eight native audit shapes
check every comparison-failure prefix, including the heap fallback. Existing
infallible callers use the same implementation through an `Infallible` adapter.

Lifecycle preserves native update order, object containment versus mode status,
append-only mode/owner queries and duplicate active owners. Resetting sensitive
activation leaves selection mode status and tolerances unchanged. Builder replacement
does not invalidate existing trees. Repeated prebuild enable replaces the pool,
and old workers are joined after releasing the selector state lock. DumpJson retains
native field order, live object counts and depth-limited children. Eleven Rust tests
and 170 native traces cover this slice, including retained permutation storage and
exact status/dump text (only process-local pointer values are canonicalized).

Subsequent integrations completed Pick overloads, BVH traversal and overlap
collection, view-dependent layer updates, selection images and sensitive debug
presentations. See `viewer-selector-traversal.md`, `selection-images.md` and
`sensitive-display.md` for their behavior and native evidence. SelectionManager
friend activation orchestration is implemented in the following section.

### Selection orchestration and prebuilding

`SelectMgrSelectionManager` retains an injected viewer-selector handle and a global
object set. Selector access borrows the retained handle. Load, remove, activation,
deactivation, status queries, structure restoration and ordinary updates descend
through children in native order; activation skips erased children. Objects without
own presentations still propagate to children. A repeated Load returns immediately,
so it neither adds a new mode nor revisits children. Forced recomputation follows its
distinct native path: clear/restore structures recurse, while primitive recomputation
and transformation update apply to the requested object. Non-forced recomputation
includes inactive stored modes despite the narrower native header description.

The manager's ViewerSelector friend operations toggle every sensitive's active flag,
but update tolerance counts only on state transitions. Sensitivity replacement updates
tolerance counts whenever both registries contain the object, even when deactivated;
a decrement failure retains the earlier changed sensitivity. Remove clears selection
entities with the native default `update=false`. Re-activation of a removed empty mode
replaces its selection handle. Global membership, BVH flags, cleared entities and
callback side effects retain native partial state after an exception. Locks never
span the selectable's ComputeSelection or transformation callbacks.

`StdSelectBrepSelectionTool::pre_build_bvh` implements the native inclusive 800,000
sub-element threshold. It also examines immediate children of an exact sensitive
group; it does not recursively descend into nested groups. Shape construction helpers
on that namespace remain unported. Background mode queues each entity and immediate
group children through the actual pool. Ten direct tests include a completion-notified
background entity, foreground boundaries, nested groups and failure stopping order.
The numeric oracle adds 252 stateful manager traces and 48 threshold traces.

An unchanged native probe crashes on Activate of an unloaded object, Remove of that
object, then Activate of a different mode. Activate can add selector membership without
adding global membership; Remove then clears its selection entities but skips selector
cleanup. The subsequent sensitive BVH rebuild dereferences a cleared entity in
SelectMgr_SensitiveEntitySet::Box. The corpus retains standalone unloaded activation,
but establishes global membership before subsequent randomized activation lifecycles.
It never treats native undefined memory access as a required Rust result. Protected
own-presentation flags are configured before the random lifecycle, rather than toggled
while selections remain registered. Callback failures and arbitrary update/status
transitions are still exercised and compared after every operation.

### AIS selected-owner and global context state

`AisSelection` implements the virtual AIS selection protocol over native `List` nodes
and `ListCursor` positions. Its protected fields retain the ordered owners, independent
public iterator and owner-to-position map, including the native 100,000-entry reserve.
`Objects` and `Value` borrow actual stored references. Native removal repairs the next
owner's map position and advances public iteration before `IsForcedHilight`; forced
highlighting can therefore retain the owner while advancing the cursor. Owner selected
flags and map membership are distinct: externally clearing a flag can cause duplicate
list entries. `ReplaceExtra` compares count plus membership, so duplicate picked owners
can clear a different selected list of the same length. Array lower bounds and overlap
flags do not alter iteration order; overlap is ignored by the pinned implementation.

All internal Clear/Select/AddSelect calls retain virtual dispatch. State locks are
released before owner and filter callbacks, allowing read-back and reentrant selection.
`SelectMgrFilter::is_ok` now returns `Result<bool, Failure>` so callback exceptions can
stop selection and composition without inventing a boolean result. AND, OR and combined
filters preserve callback order and early decisions; disabled objects bypass callbacks.
Native undefined null-filter dereference in appendOwner becomes a NullObject error only
after null/no-selectable owner short-circuiting. Reading an invalid selection cursor
returns NoSuchObject rather than dereferencing an invalid native iterator.

`AisGlobalStatus` preserves independent display/highlight/sub-intensity flags, unique
ordered selection modes and retained drawer identity; modes and style are borrowed.
`StdSelectViewerSelector3d` is the native typedef, not a new runtime type. This does not
complete ViewerSelector picking or AIS_InteractiveContext. Fourteen direct tests and
357 new native cases cover selection traces (205), status traces (80), and filter
composition (72), including failures, reentrant callbacks, null owners, removal while
iterating, duplicate picks and externally changed owner state.

### Viewer-selector geometry and debug cleanup

The selector now groups Z layers in driver order. A depth clear or a change in depth
writing increments the group, starting with a previous depth-write state of true.
The map is cleared before driver callbacks, so failure retains only the completed
prefix. ClearSensitive follows live sequence links, invoking Erase, Clear(true) and
Remove without holding selector locks; it clears the sequence before updating a
non-null view. A null debug structure returns a safe NullObject failure while retaining
the sequence (native dereference is undefined).

Private updatePoint3d and computeFrustum bodies are available to the upcoming traversal
through shared helpers. A point pick updates the criterion's position and normal before
transformation and tolerance. Invalid picks without a point return before either step.
Normal normalization uses native float arithmetic and the ShortRealEpsilon threshold;
transformation uses a double homogeneous vector with w=0, narrows to float, and does not
renormalize. Initial location belongs to the criterion's entity, while sensitivity
belongs to the separate entity argument. Pixel/sensitivity tolerance uses factor/33
without a camera, camera scale for orthographic projection, or the absolute view-direction
projection of point minus eye for perspective. Uniform tolerance is independent of camera.
The pinned release gp_Mat inversion does not raise for a zero matrix: native nonfinite
coordinates are preserved and compared by the existing NaN-classification policy.

Frustum scaling uses the selector's active volume and custom-tolerance offset. Initial
location composes on the left of the object inverse. Only scaled volumes without a
transform enter the integer-scale cache. Cached active-volume identity is shared; each
result receives object clipping after cache lookup/construction, without modifying the
cached clipping state. A request with neither scaling nor transformation copies the
object manager. Two-level traversal and presentation lookup were completed in the subsequent
viewer-selector traversal integration; see `viewer-selector-traversal.md`.

Ten direct tests and 2,656 native cases were added: 720 point cases, 912 frustum/cleanup
cases and 1,024 layer patterns. The oracle calls the actual linked private OCCT methods
through explicit-instantiation member-pointer access; no native algorithm was copied.
Rectangular rebuilding after clearing Camera() crashes natively in cacheVertexProjections,
which unconditionally dereferences Camera()->IsOrthographic(). Rebuild fixtures retain
a camera; null-camera point calculations and unchanged-frustum cases remain compared.
The minimal crashing request and stack are archived independently from parity claims.

### AIS interactive context and abstract object

The context now constructs actual presentation/selection managers, selection, OR filters,
viewer links and all six native styles. Highlight styles link the original default drawer;
replacing the default does not relink existing objects or styles. In OCCT 3d097a0,
assigning the plane-edge Prs3d_LineAspect shares its underlying Graphic3d handle. All four
highlight styles therefore share that handle with the default drawer; later InitAttributes
sets their plane edges to SKYBLUE while their independent line aspects keep cyan/gray.
Native widths, material, point marker, plane sizes, datum colors and auto-triangulation
are preserved. Context getters borrow the actual retained handle, including MainSelector
and the polymorphic selection container. SetSelection replaces it without clearing either
container; destruction dispatches virtual Clear on the current container, drops the
selection manager, clears registered object context links and marks their selection BVHs
Renew. The protected filter field is present but is not newly counted in the API ledger.

AisInteractiveObject embeds real presentable and selectable bases. SetContext stores a
non-owning pointer before linking the drawer, ignores same-pointer assignments, and leaves
the old drawer link on clearing. Application owners are retained and borrowed. Redisplay
calls its context with viewer update disabled. HasPresentation and Presentation query the
context's actual manager with the object's drawer display mode. SetAspect updates only
the last existing group for shading, line, point or text; null/unsupported aspects and
absent presentations/groups do nothing. Its virtual defaults reject dragging, return the
unknown kind and signature -1, and expose the AIS DumpJson override through the shared
selectable override macro. Both native enum types and the legacy kind aliases are ported.

Context Redisplay sequences presentation recomputation before selection recomputation.
Presentation failure prevents selection work. RecomputeSelectionOnly temporarily
deactivates modes recorded in global status, recomputes, and reactivates them only for a
registered displayed object. Failures preserve the completed prefix. RecomputePrsOnly
with viewer update enabled also recomputes transformation against the first active camera
for unregistered objects. Callbacks run without context state locks. ActivatedModes
appends to the caller's list. LastActiveView checks membership in the viewer's active
list. The pinned native constructor leaves its raw last-view pointer uninitialized;
Rust initializes it to null and the native test fixture explicitly initializes it before
reading. Expired raw context links retain pointer identity through WeakHandle; safe Rust
cannot reconstruct a live handle from a destroyed native context.

Fourteen direct tests and 570 native comparisons cover constructor style graphs (145),
object update/exception/aspect/dump/view traces (417), and selection replacement/context
shutdown (8). Only process-local pointer text is normalized in DumpJson; identities are
checked separately. AIS_InteractiveObject's own declarations are complete, while
AIS_InteractiveContext remains partial: display/erase/remove, selection actions,
advanced attribute methods and its full dump are not yet implemented. Mouse/axis
detection and dynamic highlighting are now ported; see `ais-detection.md`.

A failing virtual Clear callback during context destruction now aborts, matching the
pinned native noexcept destructor. An isolated Rust regression test and a separate native/
Rust probe process check require SIGABRT. Core dumps are disabled in the native verifier;
this fatal request is kept outside the ordinary line-oriented corpus.

## AIS selection actions

The [Martensite selection action notes](ais-selection-actions.md) describe owner and
object selection, camera picks, grouped highlighting, style-slot replacement,
callback traversal and the associated native evidence. The null-driver tests
remain separate from Vulkan rendering acceptance.

## AIS animation timelines

Martensite ports the `AIS_Animation` timeline, progress record, protected state and
`AIS_AnimationTimer` alias from OCCT `3d097a0`. `AisAnimationSource` supplies virtual
lifecycle and frame hooks; derived atoms embed `AisAnimationBase` and can delegate
an override to the exported `ais_animation_*` base bodies. `AisAnimationMembers`
supplies the nonvirtual queries and child edits. For example, add a child handle,
set its start and own duration, start the parent, then call `update(pts)` before
rendering a frame. The timer is injectable and does not schedule redraws itself.

Children execute before the parent's own callback. The final callback executes
with stopped state, and pausing the parent stops its children. Child sequence
traversal follows live stable links without holding locks across virtual calls,
so a callback can append an already-started child for the same frame. Release
`children()`'s read guard before editing the list. Native cached-duration quirks
are retained: child edits refresh the parent, child time setters do not, `clear`
retains the old child-duration cache, and self-copy empties the child list.

`python3 tools/verify_ais_animation.py --source ../OCCT --native-build BUILD`
compares 319 independent native timeline, identity-edit and frozen-timer cases.
Eleven Rust tests cover reentrant callbacks, RTTI, signed zero/NaN and deterministic
running timers. These timeline checks cover the base timeline. Camera animation is tracked below;
object animation remains open visualization work.


## AIS camera animation

Martensite ports `AIS_AnimationCamera` from OCCT `3d097a0`. A camera atom retains
its target view and start/end camera handles, interpolates into the existing live
view camera, restores the view's immediate-update mode, and invalidates the view.
It schedules no immediate redraw. Missing view or endpoint handles skip the frame;
without a positive own duration the atom applies the end camera. Endpoint edits
remain visible because setters retain handles rather than copying camera values.
`target_view()` corresponds to native `View()` and avoids Rust `Handle::view()`.

For example, construct `AisAnimationCamera::new("orbit", &view)`, set endpoints and
own duration, call `start(false)`, then apply `update(pts)` before redrawing. It
implements `AisAnimationSource`, so a parent timeline can schedule it with ordinary
relative start times. The live view camera keeps its original handle identity.

`python3 tools/verify_animation_camera.py --source ../OCCT --native-build BUILD`
compares 594 native endpoint/lifecycle/camera cases. The Vulkan harness's
`--case-family camera-animation` retains eight modes across every size, projection
and translation case on lavapipe and the native host. The `camera-animation-updates`
release workload measures interpolation, invalidation and GPU redraw/readback
with 20 warmup and 100 measured frames. Capture results and raw timing distributions
are retained in the runtime receipt; strict differences remain failures.

The [2026-10-11 runtime receipt](../porting/runtime-evidence/camera-animation-2026-10-11/summary.json)
records 48/48 strict native captures per device and zero validation errors. Three
camera tests, three retained-stream tests, the full TKV3d suite, 42 Python tests
and Clippy pass. Lavapipe passed the timing comparison. The final host timing
window exceeded camera/resize p95 and AIS remove/display limits; two subsequent
identical host repetitions passed all 23 comparisons. The failed window and raw
repetitions are retained. These measurements do not establish an accepted baseline
for the entire renderer, whose other capability and parity gaps remain open.

## AIS context location

Martensite ports `AIS_InteractiveContext::SetLocation`, `ResetLocation`,
`HasLocation` and `Location` from OCCT `3d097a0`. For example, call
`context.set_location(&object, &Location::new(transform))?` before rendering.
Placement replaces the local transform, applies pending selection updates and
refreshes the last dynamically highlighted owner's transform. Context locks are
released before virtual callbacks. These calls do not redraw or invalidate views.

An empty identity location resets an effective transform and skips highlight
refresh. A location containing an identity matrix is a nonempty datum and follows
the regular update path. Queries include inherited parent transforms; resetting
local placement leaves parent placement effective. `Location` returns a fresh
datum even when the effective matrix is identity. Null update handles are no-ops;
a null query returns `Standard_NullObject` where native dereference is undefined.

`python3 tools/verify_context_location.py --source ../OCCT --native-build BUILD`
compares 1,152 independent native cases across initial/final placements, display,
erase, inherited transforms, identity-chain variants and dynamic highlights.
Nine focused tests cover nulls, datum identity, parent placement, transform replacement,
active selection updates without geometry recomputation, failure ordering and reentrant
highlight callbacks.
This supplies the context prerequisite for object animation; the object animation
classes and their GPU/performance fixtures remain open.
