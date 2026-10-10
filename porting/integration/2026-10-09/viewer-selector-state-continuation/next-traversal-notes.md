# Next continuation: ViewerSelector traversal and selection orchestration

Pinned source: `src/Visualization/TKV3d/SelectMgr/SelectMgr_ViewerSelector.{hxx,cxx}`
at OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

## Implemented and validated

`viewer_selector{,_members,_results,_objects,_dump}.rs`: native constructor/defaults,
public protected base/fields, virtual SortResult, lazy picked ranks, borrowed criterion
and entity-handle references, configuration, object/selection lifecycle, mode/status
and active-owner queries, real pool configuration, actual volume/set access, DumpJson.
64 implemented ledger mappings and one partial class. Eleven direct tests; 170 native
state traces, including exact JSON (only two process-local pointers normalized).
Shared fallible libstdc++ sort preserves every partial move on comparison failure;
eight shapes / 9,368 failure-or-success runs and four Rust sort tests cover it.
Keep the class partial until all missing contracts are supplied.

## Suggested next seam

Implement protected checkOverlap + private updatePoint3d and updateZLayers, with
native protected-method fixtures, before the full two-level traversal. Keep callbacks
outside selector locks: Matches, owner accessors, persistence Compute and virtual
sorting can reenter other state. Mutating a borrowed picked result is prevented by
the guard; release it before calling a selector mutation.

checkOverlap calls Matches even for a null owner, then returns if unmatched/null.
Layer lookup defaults to Graphic3d_ZLayerId_Default for a null selectable; otherwise
uses its actual z layer. Populate Entity, owner Priority, Depth, MinDist. A selectable
with presentations and exactly TMF_2d persistence prefers priority; initialize display
priority INVALID, then use AIS_InteractiveObject::Presentation if non-null. Check
that AIS native interface exists; do not pretend any PrsMgr object is AIS.
For an existing owner increment its stored NbOwnerMatches BEFORE updatePoint3d or a
possible comparison failure. New criterion inherits the incremented count. Box hits
never replace an existing criterion; other selection types replace only if closer.
New owners updatePoint3d then insert with count one. checkOverlap itself does not
mark sorted=false; TraverseSensitives owns that invalidation.

updatePoint3d only changes Point/Tolerance/Normal for Point selection. Explicit picked
normal is f32-normalized only when squared length exceeds ShortRealEpsilon; preserve
operation precision. Invalid pick with no point copies PickedPoint and returns before
transform/tolerance. Otherwise use DetectedPoint(depth). Combine Entity::InvInitLocation
with the passed inverse transformation, invert once for point and homogeneous w=0
normal transformation; do not renormalize the transformed normal. Uniform tolerance
is direct; sensitivity/pixel tolerance uses null-camera fallback factor/33, orthographic
cameraScale*factor, or abs((point-eye).dot(direction))*cameraScale*factor.

updateZLayers clears first, iterates Viewer::GetAllZLayers in driver order; increment
position if depth clears OR depth-write setting changes from previous (initial true),
then bind actual layer. Preserve partial map if a settings lookup fails.

## Full traversal ordering

computeFrustum scales only if the main manager has a scalable active volume and
entity effective sensitivity is less than overall tolerance. Effective sensitivity
adds custom tolerance only when IsCustomTolSet. Compose init-location inverse with
object/entity inverse. Scale+transform builds fresh; scale-only caches by integer
scale; transform-only fresh; neither aliases/copies the object manager. In transformed
branches copy object view clipping after constructing global manager transformation.

traverseObject reads its sensitive set, returns empty, builds actual BVH before
object persistence checks. Object persistence with null camera returns. Early root
bounds rejection is disabled when any entity has persistence or flipping. Override
object clipping wins; zoom/rotate persistence clips by world anchor against active
global planes, then uses object-only clipping. Box selection checks global view planes
at whole-tree and leaf stages, including overlap-disabled partial touching rules.
Visit left child first and push right. Active sensitive entities only. Entity persistence
replaces inverse transform with persistence*object transform. Flipper sees world-view
multiplied by object transform; preserve OCCT's comment about unavailable group transform.
Flipping multiplies before inverse persistence. computeFrustum then checkOverlap.
When overlap disallowed and box/polyline, remove only newly inserted owners (reverse
index loop) whose recorded match count is less than entity-set owner count.

TraverseSensitives acquires a BVH pool Sentry, clears stored picks and marks unsorted,
reads window/pixel size, camera matrices and cached eye/direction, then UpdateBVH.
Traverse five subsets in enum order. Null camera only visits regular3d. 2d subsets
transform to eye space using a copied-mapping identity-orientation camera; ortho3d
copies the camera and forces orthographic. IMPORTANT: aProjectionMat/aWorldViewMat
are mutated across subset iterations in the pinned source; do not silently reset them.
CameraScale is updated per subset only if its manager camera is non-null. Actual
BVH traversal and view-affinity visibility precede traverseObject. Finally dispatch
virtual SortResult. Do not shortcut to brute-force or mark Pick implemented early.

Pick overloads initialize the appropriate volume, camera/viewport, z layers, clipping
and view ID before TraverseSensitives; inspect cxx after line 1150 before translating.
SelectionImageFiller and SelectMgr::ComputeSensitivePrs are still needed for image and
debug display APIs. Anonymous image fillers use Bullard random calls as constructor
arguments: determine compiler argument order with native fixtures.
SelectionManager needs private Activate/Deactivate/Remove; native activation toggles
all entity flags but adds/decrements tolerance only on relevant state transitions.

## Handoff and evidence

Preserve `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent 7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5.
25 commits plus uncommitted continuation work remain outside main. Git metadata is
read-only in this session; do not attempt sandbox-policy workarounds. Original Claude
worktrees are preserved; dirty linked-slot work on worktree-agent-aeed1901b085eca95
remains unvalidated. Full TKernel runtime verifier still requires a clean checkout;
the focused STL audit is not advertised as full TKernel acceptance. Full port scope
remains 56 toolkits / 76,041 APIs. All 4,937 TKernel + 209 TKV3d Rust tests, Clippy,
7 tooling tests, 127,416 TKV3d and 75,190 TKMath comparisons passed at this checkpoint.

Further dependency check: `crates/tkv3d/src/ais/mod.rs` currently exports only
AisSelectionScheme. AIS_InteractiveObject is not ported. Its 160-line header and
162-line cxx depend on AIS_InteractiveContext (raw non-owning context pointer), with
Presentation() using context MainPrsMgr and drawer DisplayMode. Do not implement
checkOverlap's AIS branch with a guessed generic-presentable substitute. Either
port the real AIS/context seam first, or keep checkOverlap explicitly partial and
start with independently complete updateZLayers plus geometric point/frustum helpers.
