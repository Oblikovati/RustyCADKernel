# Next continuation

Continue from `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata is read-only in this session;
25 committed continuation commits and the archived uncommitted patch are not on main.
Preserve original Claude worktrees, especially the dirty TKernel linked-slot refactor.
The active goal remains 100% of defined port-scope APIs; this checkpoint is partial progress.

## Current native-verified slice

AIS context construction now uses actual presentation/selection managers, style graph,
viewer, selection and filters. Abstract interactive objects use real context and manager
lookup. Context selected-owner storage is `Handle<dyn AisSelectionSource>` so replacing
it preserves destructor virtual Clear dispatch. Native constructor shared plane-edge
aspect semantics are intentionally preserved: all highlight plane edges share the
original default Graphic3d handle; subsequent initialization makes their color SKYBLUE.
Raw myLastActiveView is uninitialized in the pinned native constructor. Rust uses null;
the native fixture initializes it explicitly before reading. Native unsafe lifetime
states are not treated as defined behavior requirements.

Fourteen direct tests, 570 new differential cases; total TKV3d 257 direct tests and
131,299 numeric cases, no NaN representation differences, one native and one Rust mapped
legacy case. All-target Clippy, formatting and seven tooling tests pass. Native cases:
145 constructor/style traces, 417 object update/failure/aspect/dump/view traces, eight
selection replacement and shutdown cases. Callback failures preserve native partial
changes. Presentation failure stops before selection; selection failure follows temporary
deactivation. Context recomputation uses camera from first active view even if the object
is not registered. Supported aspect application changes only the last existing group.

88 new implemented mappings and one partial AIS_InteractiveContext class mapping are
added. No claim for filter protected storage was added. No claim for Redisplay(kind,
signature,update), object/owner HighlightStyle overloads, broad display/selection methods,
or context full DumpJson. AIS_InteractiveObject's own API declarations are implemented;
its real rendering/selection callbacks remain abstract as in OCCT.

## Recommended next dependency chain

Complete SelectMgr_ViewerSelector now that actual AIS runtime downcasting and
Presentation() exist. Native source is
`OCCT/src/Visualization/TKV3d/SelectMgr/SelectMgr_ViewerSelector.cxx`:

1. checkOverlap (269): OwnerId and Selectable are read before Matches; Matches still runs
   for a null owner, then false/null owner short circuits. ZLayerPosition defaults when
   Find fails. Entity/Priority/Depth/MinDist copied into criterion. PreferPriority only
   when selectable has at least one presentation and persistence mode exactly 2d; its
   DisplayPriority starts INVALID, then real AIS Presentation() can override it. Existing
   owner match count increments before point update and comparison. Box repeats retain
   the existing criterion except count. Other repeats replace only if IsCloserDepth.
   New owner sets count1, runs updatePoint3d then adds. Do not invent sorted-flag changes.
   This is private and should not add a fake public API ledger mapping.
2. traverseObject (410): retrieve sensitive entity set, use its actual BVH; compose
   object/persistence transforms and clipping; DFS visits left first and pushes right.
   Persistence/flipping bypass child box tests. Iterate actual active wrappers, combine
   inverse persistence/object transform and Flipper Compute with WorldView*ModelWorld;
   call existing computeFrustum helper and checkOverlap. Non-overlap box/polyline
   selection removes newly added owners missing some entities, using owner counts.
3. TraverseSensitives (696): BVH pool Sentry; clear picked and mark unsorted first;
   pixel size max(1/width,1/height) or1; camera eye/dir update when camera exists; update
   selectable object BVHs. Iterate four subsets in native order. Camera-null skips
   non-3d subsets. 2d/ortho2d creates identity-orientation cameras and changes the local
   matrices reused later; preserve this ordering. Ortho3d copies camera with orthographic
   projection. CameraScale uses orthographic scale or 2*tan(FOVy*pi/360), times pixel size.
   Filter ViewAffinity by view ID unless -1. SortResult at end, preserving virtual dispatch.
4. Four Pick overloads then actual DisplaySensitive and ToPixMap. Both helpers from the
   preceding checkpoint (`select_mgr_update_point3d`, `select_mgr_compute_frustum`) call
   genuine geometry and already have native corpus. Private native members can be called
   through the existing explicit-instantiation friend/member-pointer fixture technique.

Do not reuse the native invalid sequence clearing a rectangular volume's camera before
ScaleAndTransform rebuild: pinned native code dereferences that camera. Its earlier
crash reproducer and stack remain archived in selector-geometry-continuation.

The noexcept destructor Clear-failure edge now aborts instead of discarding failure.
The numeric verifier runs a separate native/Rust SIGABRT comparison after normal cases,
recorded under numeric_report.shutdown_failure. Core dumps are disabled. A direct Rust
subprocess test covers the same failure path. The two new class ledger mappings use their
actual AIS definition headers, not the merged inventory row's first forward-declaration site.

Two class records are canonically assigned to TKVCAF by the merged inventory because their
first source site is a forward declaration there. The ledger uses that existing toolkit
ownership (without changing inventory scope or ownership rules), pins the exact class IDs,
and hashes the actual AIS definition headers under TKV3d. Context remains a partial record;
interactive object's own declarations are implemented. This metadata assignment does not
claim a TKVCAF behavior port. All member implementations remain TKV3d mappings.
