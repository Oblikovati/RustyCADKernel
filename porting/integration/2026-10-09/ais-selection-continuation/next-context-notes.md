# Next continuation: context/object and viewer geometry

Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871. Worktree
/tmp/rustycad-resume, branch porting/resume-claude, parent 7fac5b5bf.
Git metadata remains read-only; preserve source and all original Claude worktrees.
Do not substitute the original unvalidated linked-slot refactor.

This checkpoint adds complete AIS_Selection, AIS_GlobalStatus, AIS_SelectStatus and
StdSelect_ViewerSelector3d alias: 39 implemented mappings, 14 new direct tests,
357 new native cases (205 selection / 80 status / 72 composition).
Selection has stable native list/cursor/map storage, virtual protocol and borrowed
actual objects/value. Keep forced-highlight cursor timing, duplicate-list behavior
from externally cleared owner flags, and ReplaceExtra count+membership semantics.
`SelectMgrFilter::is_ok` is now fallible; callers must propagate Result. Composition
stops on the first decision or failure; disabled objects bypass filter callbacks.

Next object/context details are in the preceding selection-manager checkpoint's
next-ais-notes.md. Its AIS_Selection/GlobalStatus and typedef work is now completed.
AIS_InteractiveObject requires a real context's DefaultDrawer, MainPrsMgr and Redisplay.
Do not supply a no-op Redisplay seam. Context may be partial while implemented subsets
remain honest; constructor must still initialize real presentation manager, viewer,
selection manager, selection, filters, default drawer and six highlight styles.
Context destructor clears selection, releases selection manager, clears context on
registered objects and marks their selection BVH status Renew. Preserve ownership
boundaries: object context is a non-owning pointer, GetContext returns a handle,
SetContext stores before linking drawer, clearing retains the previous drawer link.

Independent viewer-selector helpers can be ported next without AIS dependencies:
- updateZLayers clears the map, iterates View->Viewer()->GetAllZLayers in native order,
  increments aPos when a layer clears depth OR its depth-write setting differs from
  the previous (initial true), then binds layer ID to aPos. Existing viewer.rs already
  provides get_all_z_layers and z_layer_settings; fields.z_layer_order is a DataMap.
- updatePoint3d returns immediately unless point selection. Explicit pick point uses
  float normal length and sqrtf normalization only above ShortRealEpsilon. Invalid
  result copies its point and returns before transform/tolerance. Otherwise use
  DetectedPoint(depth). Compose entity inverse initial location with inverse object
  transform, invert, transform point and nonzero normal (w=0; no renormalization).
  Uniform depth tolerance is direct; pixels/sensitivity fallback factor/33 without
  camera; orthographic camera_scale*factor; perspective abs(view-direction projection
  of point-eye)*camera_scale*factor. Keep float rounding at native boundaries.
- computeFrustum combines sensitivity scale and inverse initial location. Scale-only
  managers are cached by integer scale; copy object clipping after cache lookup/build.
  Scale+transform and transform-only do not use this cache; neither returns object
  manager copy. Preserve shared intersector and clipping storage/copy semantics.

Full checkOverlap needs actual AIS downcast and presentation display priority. Two-level
BVH traversal, four Pick overloads and pixel/debug rendering remain unimplemented;
ViewerSelector remains partial. Previous viewer-selector-state checkpoint's
next-traversal-notes.md explains these; friend activation is already completed.

Validation: freeze crates/tools/ledger before final source-bound gate and full rustdoc
capture. Capture is slow at TKBO/TKBool. Then normalize, coverage, publish, legacy tests,
metrics and status. TKernel clean-tree gate still refuses this preserved dirty checkout;
never bypass it. Archive new source-bound reports and full continuation patch without
overwriting historical artifacts or claiming continuation changes are on main.
