# Next: SelectMgr sensitive-entity BVH set

ToleranceMap and SortCriterion are complete in this checkpoint (24 API mappings,
8 new Rust tests, 29,826 new numeric cases). Their native checks use the unchanged
OCCT pin 3d097a0328e71b826377d4814ab05ec3c3d23871. Continue toward the full 76,041
API scope in /tmp/rustycad-resume on porting/resume-claude; Git remains read-only.

The next concrete port is SelectMgr_SensitiveEntitySet; then
SelectMgr_SelectableObjectSet, ViewerSelector and SelectionManager. Native source
inspection already established the following; recheck current code when implementing.

SensitiveEntitySet inherits BVH_PrimitiveSet<double,3> and declares native RTTI.
The template BVH_PrimitiveSet and BVH_Object do NOT declare their own RTTI, so its
native named parent resolves to BVH_ObjectTransient, then Standard_Transient.
Rust TKMath currently represents BvhObjectTransient as a plain Clone/Debug value
(is_dirty: bool, properties: Option<Arc<dyn BvhProperties>>), and BvhPrimitiveSet
contains Cell-cached box plus a value BvhTree and an optional Arc<dyn BvhBuilder>.
It has no existing Standard handle/RTTI interface. Add a real owned interface at
this seam; do not register an unrelated fake parent or misstate its parent as
Standard_Transient. A handle-owned sensitive set must still satisfy Send+Sync,
which the embedded Cell-based primitive-set state cannot do without a lock.
Select3DSensitiveSetBase already uses Mutex<BvhPrimitiveSet<BvhVec3d>> and a
SetPrimitives adapter to bridge the mutable BvhSet/BvhPrimitiveSetAccess APIs.
Inspect that pattern for reuse without coupling the new set to sensitive-entity
matching. Native Select3D_BVHBuilder3d is only a typedef; Rust already exports
Select3DBvhBuilder3d = Arc<dyn BvhBuilder<BvhVec3d>>. Null builder is representable
as BvhBuilderHandle and should retain native failure timing.

Store the sensitives in the owned IndexedMap<Handle<SelectMgrSensitiveEntity>>
and owners in DataMap<Handle<dyn SelectMgrEntityOwnerSource>,i32>; TKernel's Handle
implements HashKey. The signed IndexedMap methods find_key_int/swap_int retain
native index failures. Public set indices are zero-based over one-based map keys.

Append increments owner counts only for a newly inserted sensitive, but increments
persistence/flipping counts on EVERY Append, even duplicates. Preserve that native
imbalance. Append(selection) marks dirty even if empty; an entity whose native RTTI
fails IsKind(Select3D_SensitiveEntity) is deactivated. This RTTI check can matter for
custom Rust interfaces even though BaseSensitive is typed to the trait.
Remove finds the index, swaps with the last item, decrements counters based on the
entity's CURRENT persistence/flipper fields, then RemoveLast and removeOwner.
Null owners are skipped; owner entries unbind at zero. Removing absent entities
still results in MarkDirty after the selection loop. Box returns a void box for a
persistent entity, otherwise BoundingBox. Center uses its geometry center and
selects Z for every axis other than 0 or 1. Box/center, owner counts, uniqueness,
BVH swap order, dirty/cache state and inherited RTTI all need native comparisons.

SelectableObjectSet has five subsets plus a count sentinel. Its iterator visits
subsets in enum order; More advances across empty subsets. Subset classification
also scans presentations for group persistence/flipping when object persistence
is null. Existing camera-state and Graphic3dTransformPers APIs can support the
cache updates; inspect its CXX bodies before translating those updates.

Earlier unfinished Claude linked-slot/list refactor is still untouched in
.claude/worktrees/agent-aeed1901b085eca95 (d6189068), with staged deletions and
untracked linked_slots sources. Do not delete or integrate it without validation.

Additional foundation detail: BvhProperties currently uses Arc plus Any downcast
and is Send+Sync; BvhTree is Clone but stored by value in primitive state. Native
BVH() returns a shared tree handle that survives rebuilds. Do not substitute a
cloned snapshot if consumers need retained tree identity. The existing sensitive
matching helper with_built_tree temporarily takes/restores the tree while holding
the content lock; use a scoped read/borrow or a stable shared owner for the new
public set interface. Builder assignment does not mark dirty. Fresh primitive
sets start clean with a void cached box; Append/Remove are what mark them dirty.
Native BVH_PrimitiveSet::Update is protected, while the current Rust access trait
makes update callable; preserve semantics when implementing the concrete set.
