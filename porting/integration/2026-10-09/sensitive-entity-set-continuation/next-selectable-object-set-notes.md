# Next port: SelectMgr_SelectableObjectSet

Pinned native source: `src/Visualization/TKV3d/SelectMgr/SelectMgr_SelectableObjectSet.{hxx,cxx}` at `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The sensitive-entity set and retained primitive-tree handle are now implemented.
Reuse `BvhTreeHandle` for all five independent selectable-object subset trees.
`SelectMgrSelectableObject` already inherits virtual `bounding_box` through
`PrsMgrPresentableObject`; member helpers expose presentations, persistence and
object transformation. `Graphic3dCStructure` member helpers expose
`has_group_transform_persistence` and `has_group_flipping`; structure helpers
expose `groups`, and group helpers provide bounding boxes, flippers and persistence.

Native details observed in the bodies:

- Five indexed maps: regular 3D, persistent 3D, persistent 2D, orthographic 3D,
  orthographic 2D. Empty construction creates clean, stable trees for each.
- The regular builder actually has **4 bins**, leaf size 1, depth 32 and the
  boolean constructor option true. The header comment says 32 bins; follow code.
  All other builders are linear, leaf size 1, depth 32.
- Append computes the appropriate subset BEFORE searching for existing membership.
  Remove searches every subset and uses swap-last removal. ChangeSubset does
  nothing for an absent object or unchanged classification; otherwise remove/add.
- Without object persistence, any presentation CStructure with group persistence
  or group flipping moves the object to persistent 3D. Otherwise use persistence
  mode's 2D and orthographic bits to classify it.
- UpdateBVH skips empty subsets, retaining stale tree nodes. Only dirty nonempty
  regular 3D is rebuilt without a camera. With a camera, persistent 3D observes
  full view state, 2D only projection state; all observe viewport changes.
- Camera-present update clears all four persistent dirty flags, including empty
  subsets. Regular 3D dirty is cleared only after its nonempty rebuild. Null
  camera still stores viewport size but leaves last view state unchanged.
- Orthographic persistent subsets clone the camera and force orthographic
  projection. 2D adapters receive identity world-view; 3D receives camera view.
- Persistent adapters cache per-object bounds and swap the cache with map order.
  Start with virtual object bounds; include each eligible valid group box after
  applying its flipper and persistence. Flip uses WorldView * ModelWorld for an
  object transformation; native deliberately excludes the group's transform.
  Then apply the object's persistence to the accumulated nonvoid bounds.
- Public Iterator is live over subset map cursors. `More` advances through empty
  subsets and mutates cursor state; `Next` only advances current map. Preserve
  iterator invalidation behavior rather than cloning a flattened snapshot.
- DumpJson emits all subsets, flags, each pointer, empty separators and viewport
  dimensions. Depth is ignored. Normalize pointers through existing dump support.

Suggested modules: object-set ownership/public interface, subset builder adapters,
persistent bounds helper, iterator. Tests should compare subset membership/order,
retained trees, camera/viewport triggers, empty stale-tree behavior, live iterator,
group flipping/persistence and dump depths. Keep the full 76,041 API scope.

Do not integrate the original unfinished TKernel linked-slot refactor. Worktree
`/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/.claude/worktrees/agent-aeed1901b085eca95`,
branch `worktree-agent-aeed1901b085eca95` at `d618906825cab733d312701b51f795dd3d99fe61`,
still owns that staged/untracked work. Its dirty linked-slot changes have not been
validated; preserve all original Claude worktrees until integration is possible.
