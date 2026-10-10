# Next continuation: ViewerSelector and selection orchestration

Read the pinned source directly:
`src/Visualization/TKV3d/SelectMgr/SelectMgr_ViewerSelector.{hxx,cxx}` at
`3d097a0328e71b826377d4814ab05ec3c3d23871`. The selecting-volume manager,
sort criterion, tolerance map, sensitive set, selectable-object set and real BVH
thread pool are now implemented. No ViewerSelector source edits have been made.

ViewerSelector is a named transient with virtual `SortResult`. Preserve virtual
dispatch, subclass-visible protected state and mutable `SelectableObjects` /
selecting-volume access, using the established transient trait/base pattern where
needed. Do not map the whole class as implemented while traversal or public image
and display operations are missing.

The constructor initializes depth tolerance=0, type=SensitivityFactor, closest=true,
cameraScale=1, prebuild=false, sorted=false, left-child-first=false, and a real
four-bin regular BVH builder with leaf size one / native maximum depth. Pixel tolerance
changes early-return if the effective tolerance already equals the argument; negative
values reset defaults. Activate marks every sensitive active and increments tolerance
only when selection state changes; Deactivate clears all sensitive flags and decrements
only from Activated. These private members are needed by the friend SelectionManager.

Object lifecycle already has usable set primitives. AddSelectableObject creates a
sensitive set only when its map lacks the object. AddSelectionToObject appends and
builds immediately, adding the object then retrying if absent. Move changes the subset.
SetEntitySetBuilder updates every existing entity set. Modes appends to the caller's
list even if the object is absent; its bool result reports containment. IsActive and
IsInside reject absent objects, then inspect the object's selection state. Status has
separate selection and object-string overloads; preserve exact whitespace.

SortResult returns sorted=true on empty without rebuilding index storage. Otherwise
it resizes a one-based index array and uses the actual SortCriterion comparator.
Picked / PickedData sort lazily. Traversal covers five BVH subsets, camera persistence,
entity tolerance rescaling, transforms, clipping, z-layers, view affinity and owner
aggregation. Inspect the complete cxx before implementation; it is much larger than
the simple state members. Do not shortcut traversal to brute-force probes or fake
inherited methods.

SelectionImageFiller is an internal abstract transient without an RTTI declaration
in its own header. Its factory depends on ViewerSelector picked results and object
iteration. Concrete fillers are anonymous cxx classes. They use existing ImagePixMap,
Quantity color, Bullard generator, maps and pixel formats. Color generation traverses
objects/selections/sensitives in native order and deduplicates colors. The three
NextInt calls are function arguments: verify the native compiler's evaluation order
rather than assuming left-to-right RGB. Normalized depth has an intermediate GrayF
image and Flush behavior. Defer whole-class claims until all factory variants exist.
`SelectMgr::ComputeSensitivePrs` also remains and is needed by display-debug paths.

The BVH pool checkpoint has 23 implemented mappings and 2 explicit prohibited-copy
exclusions, 198 TKV3d / 4,936 TKernel all-target tests, 127,246 TKV3d numeric traces,
and 251 focused OSD thread comparisons. Full TKernel runtime acceptance requires a
clean Git checkout and was not run; the focused script is in the checkpoint.

Preserve `/tmp/rustycad-resume` on `porting/resume-claude` and the original Claude
worktrees. Git metadata is still read-only. The dirty linked-slot refactor on
`worktree-agent-aeed1901b085eca95` is untouched and unvalidated. Do not merge it or
claim it tested. Keep the full 56-toolkit / 76,041-API scope unchanged.
