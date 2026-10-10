# Next continuation: AIS state and the remaining viewer traversal

Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
Current source `/tmp/rustycad-resume`, branch `porting/resume-claude`, parent
7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5. Git metadata is read-only, so preserve
this checkout and every original Claude worktree. The original dirty TKernel
linked-slot refactor remains unvalidated; do not silently substitute it.

## Completed in this checkpoint

SelectionManager's whole public/protected API: injected borrowed selector handle,
global membership, lifecycle, modes, sensitivity, force/non-force recomputation,
structure clear/restore and update. Private loadMode/buildBVH also translated, plus
ViewerSelector's friend Activate/Deactivate. StdSelectBrepSelectionTool has only
PreBuildBVH; its record is partial. Other helpers remain unimplemented.
19 implemented mappings and one partial record. 219 TKV3d Rust tests (10 new),
127,716 native comparisons (300 new), Clippy all TKV3d targets and 7 tooling tests.
Native source-bound toolkit gate includes one native and one Rust mapped legacy case.

Preserve native differences:
- Repeated Load returns before revisiting children or loading a requested new mode.
- Activate ignores erased children. Non-presenting parents still propagate operations.
- Non-forced RecomputeSelection includes inactive stored modes despite header wording.
- Forced recomputation clears/restores child structures but recomputes only this object's
  primitives; do not turn it into recursive forced RecomputeSelection calls.
- Remove uses ClearSelections()'s default update=false, preserving update flags.
- Sensitivity mutation precedes tolerance decrement; an error retains the new sensitivity.
- Foreground PreBuildBVH threshold is >=800000, with immediate exact-group children only.
  Background queues main entity plus immediate downcast-group children through real workers.
- BVH Stop is not a queue drain: tests wait for named completion notification before stop.

The unchanged native kernel crashes on Activate(unloaded, mode2), Remove, then
Activate(mode0): Remove skips selector cleanup outside myGlobal and leaves cleared
sensitive wrappers. Minimal input and diagnostic stack are archived. Randomized
lifecycle cases explicitly load relevant objects before activation; standalone unloaded
activation is still compared. Own-presentation flags are chosen before lifecycle.
Callbacks can fail and all intermediate statuses/order are still compared.

## Next dependency: AIS_Selection / AIS_GlobalStatus

ViewerSelector::checkOverlap must downcast an AIS_InteractiveObject and call its
Presentation for display priority. AIS module still exports only SelectionScheme.
Read AIS_Selection.{hxx,cxx} (130/202 lines) and AIS_GlobalStatus.{hxx,cxx} (88/28).
GlobalStatus is a small transient with display/highlight/sub-intensity state, drawer
handle and ordered unique selection-mode list. Borrow native reference-returning
members using guards, preserving actual state rather than snapshots.

AIS_Selection is virtual: Clear, Select, AddSelect, ClearAndSelect, SelectOwners,
protected appendOwner. Preserve a public base/fields and trait virtual dispatch,
including internal Clear/Select/AddSelect calls. Its storage is NCollection_List of
owner handles, live list iterator, and DataMap owner->list iterator. Current TKernel
has ListCursor/ListNativeCursor and append_at/remove_native; inspect those before
choosing storage. Do not replace native live iterator semantics with a snapshot Vec.
Constructor resizes map to 100000. Clear resets owner flags, list, map and cursor.
Select checks null/HasSelectable, then filter only if detected; calls Owner::Select
without applying its result to an already-selected bound owner. It advances the
public cursor BEFORE testing IsForcedHilight; forced highlight returns Added without
removing. Removal patches the next owner's iterator in the map. AddSelect ignores
filters and rejects an already-bound owner. appendOwner dereferences filter (null is
native-invalid), then virtual AddSelect. SelectOwners ignores overlap; ReplaceExtra
clears if input count equals selection count AND every input is bound (duplicates
can satisfy this: do not replace with set equality). Clear scheme still dispatches
Select for every input after Clear. SelectStatus and other AIS enums need porting.

StdSelect_ViewerSelector3d is merely a typedef of SelectMgr_ViewerSelector; add a
Rust type alias with honest target-class partial status accounting as appropriate.

## Context and interactive object

AIS_InteractiveObject header/cxx are ~160 lines each. Constructor embeds selectable
base, initializes raw non-owning context pointer null and owner handle null. SetContext
returns on same pointer, otherwise stores pointer before linking drawer to context
DefaultDrawer; clearing context retains drawer link. GetContext returns a strong handle.
Presentation/HasPresentation use context MainPrsMgr with drawer DisplayMode. Redisplay
calls context Redisplay(this,false,AllModes), so do not supply an invented no-op seam.
ProcessDragging returns false. SetAspect applies one of shading/line/point/text aspects
to the last existing presentation group. DumpJson delegates native selectable base.

AIS_InteractiveContext ctor (cxx115+) allocates PrsMgr manager, AIS_Selection, AndOrFilter,
Drawers and SelectMgrSelectionManager(new StdSelectViewerSelector3d), sets highlight
styles and InitAttributes. Fields include AIS_GlobalStatus map, selector manager,
presentation manager/viewer, weak/raw last active view, owner, selected owners,
filters, default drawer and highlight styles, detected sequence/cursors and flags.
Do not pretend a three-field fake context is complete. Implement real subsets as partial
until all context operations, constructor style initialization and ownership are there.

## Remaining ViewerSelector

The previous `viewer-selector-state-continuation/next-traversal-notes.md` records
updatePoint3d, checkOverlap, updateZLayers, computeFrustum and two-level traversal in
detail. Its note that friend activation is missing is now obsolete. Pick/traversal,
layer collection, image/debug rendering remain missing; ViewerSelector stays partial.
An independent next slice can port updateZLayers and private geometry helpers while
the AIS object/context dependency is built. Never replace AIS presentation lookup
with a generic PrsMgr object approximation or BVH traversal with brute-force scanning.

TKernel cursor inspection: ListCursor is Copy/Clone with opaque address/previous;
PartialEq compares current identity. `list.append_at(value,&mut cursor)`, cursor.init,
more/value/next, and `list.remove(&mut cursor)` are the likely
AIS storage primitives. Cursors remain usable across insertion; next consults live
links. ListNativeCursor adds live observer handles for destructor/removal callbacks
but is not needed merely to preserve an independent public cursor and map copies.
Do not use lifetime-bound ListIter for stored cursors. GlobalStatus defaults: display
mode0, highlighted=false, sub-intensity=false, empty modes and null style.
