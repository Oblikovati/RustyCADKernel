# Next continuation: real AIS context/object and selector traversal

Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. Worktree
/tmp/rustycad-resume, branch porting/resume-claude, parent 7fac5b5bf.
Git metadata remains read-only; preserve this checkout and all original Claude worktrees.
The original linked-slot refactor remains unvalidated and must not replace current storage.

## Completed geometry

SelectMgrViewerSelectorGeometryMembers::update_z_layers is protected native grouping.
SelectMgrViewerSelectorMembers::clear_sensitive follows live SequenceCursor links and
releases selector locks for Erase/Clear(true)/Remove, clears sequence then updates view.
select_mgr_update_point3d and select_mgr_compute_frustum expose the native private
helper bodies for traversal/probes. Ten direct tests, 2656 new numeric cases, 243 total
TKV3d tests, 130729 native comparisons. Two implemented API mappings (private functions
are not counted as additional public/protected APIs). Class remains partial.

Keep native float normalization threshold/rounding, no post-transform renormalization,
criterion-entity initial location versus argument-entity sensitivity, invalid pick early
return, camera-less factor/33, orthographic scale, perspective directional distance.
Only scale-without-transform enters the integer cache; clipping copied after cache
lookup never mutates cached clipping. Neither scaling nor transformation shares object
manager active identity. Failed construction leaves result/cache unchanged. General
singular point transforms yield native release NaNs; collapsed frustums throw a direction
construction error. Preserve both; do not turn one into the other.

Native private methods are exported in the pinned shared library. Oracle fixture
uses standard C++ explicit-instantiation member-pointer access; no copied algorithms
and no altered OCCT headers/build. Null-camera rectangular rebuild crashes in
cacheVertexProjections (Camera()->IsOrthographic()); archived repro+stack. Null-camera
point updates and unmodified rectangle identity cases remain compared. Rebuild fixtures
retain a camera, as native code requires.

## Next AIS object/context slice

AIS_Selection, GlobalStatus and SelectStatus are complete; prior AIS checkpoint explains
fallible filters and stable native selection cursors. The underlying StdSelect selector
is an alias, still partial. Next implement real AIS_InteractiveContext construction and
state, and abstract AIS_InteractiveObject base/trait, rather than a no-op context seam.

Context header protected fields (hxx ~1653): object->GlobalStatus DataMap; selection
manager; presentation manager; viewer; raw last-active-view; last-picked owner; highlight-
selected flag; AISSelection; AndOrFilter; default drawer; six styles; detected Sequence<int>;
current detected/highlighted indices; picking strategy; auto-highlight and auto-activation.
Keep context class partial until its full methods are translated. A partial constructor
that omits default style initialization is not the native constructor.

Native cxx115 constructor creates PrsMgr manager(viewer.structure_manager), retains viewer,
flags true, selection, OR filter, default drawer, cursors zero, strategy FirstAcceptable,
auto flags true, SelectionManager(new StdSelect selector). Styles: default drawer at None,
new drawers for Selected/Dynamic/LocalSelected/LocalDynamic/SubIntensity. Defaults setup,
ZLayerDefault and display mode0. Dynamic/LocalDynamic link default with CYAN1 styles and
Top/Topmost layers. Selected/LocalSelected link default with GRAY80, layerUnknown.
SubIntensity has layerUnknown, color highlight method and GRAY40, without default link.

Native initDefaultHilightAttributes cxx58 clones line/wire/free/unfree low-level aspects,
plane edge aspect, sets own shading/point/plane/datum aspects, applies color throughout
including plane arrow/iso and every nonnull datum line. Width2 for wire/line/planeedges/
free/unfree; marker O_POINT scale2; AutoTriangulationfalse. Keep assignment semantics and
avoid sharing aspects that native copies. InitAttributes cxx1977 sets Brass material,
hidden line GRAY20 width1 DASH, pixel tolerance2, datum axes lengths100 and LIGHTSTEELBLUE4,
plane lengths200 with SKYBLUE edges. Existing drawer/Prs3d and Graphic3d APIs should cover.

Context destructor clears selection, releases manager, clears context of every registered
object, marks each stored selection BVH Renew. Object context is raw/nonowning natively;
use existing weak ownership patterns without introducing a strong cycle. Object SetContext
stores context before linking drawer; clearing context retains old drawer link. GetContext
returns a strong handle. HasPresentation/Presentation use actual context MainPrsMgr and
object drawer DisplayMode, not generic presentable shortcuts. SetAspect modifies the last
group's shading/line/point/text aspect only if presentation and a group exist. Drag default
returns false. Preserve borrowed application-owner handle and dynamic AIS type identity.

Redisplay(object,updateViewer,allModes) cxx930 calls RecomputePrsOnly then
RecomputeSelectionOnly. RecomputePrsOnly: null no-op; SetToUpdate all; UpdatePresentations;
if updateViewerfalse return. If registered and displayed, viewer.Update. If active views
nonempty, RecomputeTransformation(first active view camera), even if object is unregistered.
RecomputeSelectionOnly cxx999: null no-op; ActivatedModes into list; deactivate each; manager
RecomputeSelection (defaultforcefalse); only registered+displayed reactivates prior modes.
ActivatedModes cxx3822 is the corresponding status lookup. Existing Rust presentable has
set_to_update_all/update_presentations and virtual recompute_transformation.
This is a real achievable Redisplay seam; do not replace it with an invented no-op trait.

## Viewer traversal

Remaining checkOverlap needs actual AIS downcast and Presentation for display priority.
TraverseSensitives/traverseObject, four Pick overloads, ToPixMap and DisplaySensitive
remain missing. The earlier viewer-selector-state next-traversal-notes.md remains useful;
its claims that geometry helpers, ClearSensitive and friend activation are missing are
now obsolete. Preserve two-level BVHs and callback order rather than scanning all entities.

Freeze source/tools/ledger before final gate and rustdoc capture; refresh normalized
inventory/coverage/publication/legacy tests/metrics/status. The full clean-tree TKernel
gate still refuses this dirty checkout; do not bypass. Archive source-bound reports and
full continuation patches, preserving historical evidence and main's separate baseline.
