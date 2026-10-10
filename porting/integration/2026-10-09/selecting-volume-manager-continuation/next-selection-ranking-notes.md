# Next: selection ranking and tolerance bookkeeping

The concrete SelectMgrSelectingVolumeManager is now implemented in three focused
modules and the SelectBasics fallible getters propagate through Select3D matching.
The manager fixture adds 4,800 native comparisons; all 96,090 TKV3d comparisons,
159 Rust tests, Clippy, and one native plus one Rust legacy case pass. No TKernel
production files changed in this checkpoint. Final inventory/checkpoint details
are in receipt.json. Continue in /tmp/rustycad-resume on porting/resume-claude.
Git metadata remains read-only; preserve the checkout and Claude worktrees.

The next unported small SelectMgr dependencies are SelectMgr_ToleranceMap and
SelectMgr_SortCriterion. Their headers/sources were inspected against OCCT
3d097a0328e71b826377d4814ab05ec3c3d23871; no implementation has been added.

ToleranceMap retains zero-frequency keys. Add increments an existing count and
updates the largest only when the count becomes one, otherwise returns. First
inserted key becomes largest even when negative. Decrement of an absent key is a
no-op, but of a retained zero count raises Standard_ProgramError before mutation.
When the last instance of the largest key is decremented, the new maximum starts
at -1 and scans nonzero counts. Tolerance() returns 2 whenever largest < 0,
otherwise largest + custom if custom >= 0, or just largest. IsCustomTolSet is
strictly custom > 0, so custom=0 participates in Tolerance but reports false.
ResetDefaults changes only custom, not map or largest. Use the owned NCollection
DataMap, if appropriate; do not translate this into removal of zero keys.
Native signed overflow is undefined; normal-range cases should be exact.

SortCriterion is a value record with public entity Handle, Point3, Vec3<f32>
normal, depth/min_distance/tolerance f64, selection/display/Z-layer priorities,
owner-match count, and prefer-priority flag. Defaults are null/zero/false.
IsCloserDepth first checks ZLayerPosition, then depth difference > sum tolerance.
Otherwise it computes abs(cos(angle(normal, other_normal))) if both Vec3<f32>
moduli > 0, using gp_Dir constructors and Angle (can throw for tiny nonzero normals).
Do not replace the angle+cos path by a dot product; exact bits affect thresholds.
Adjusted depths depth-tolerance are compared when difference > confusion and
(normals cos > .5 with nearly equal tolerance, or cos near1). Remaining order is
selection priority >, THEN display priority > even if selection priority is lower,
THEN equal selection priority and MinDist <. This is not a lexicographic ordering.
IsHigherPriority has conventional distinct Z-layer, selection, display checks,
then min distance when depth difference <= confusion, else depth. PreferPriority
is only stored; these two comparison methods do not consult it. Test nontransitive
and boundary/float-rounded normals with exact native comparison.

After these leaf dependencies, inspect SelectMgr_SensitiveEntitySet and
SelectMgr_SelectableObjectSet before the much larger ViewerSelector and
SelectionManager orchestration. Their BVH/transform-persistence dependencies
already partly exist in TKMath/TKService. Check inventory gaps, rather than
assuming all methods in previously ported classes are claimed.

SensitiveEntitySet inspection: owner counts increment only on a newly inserted
sensitive, but persistence/flipping counters increment on EVERY Append, even a
duplicate. Preserve this native imbalance rather than recomputing counts from the
unique map. Append(selection) marks dirty even if empty; rejected non-Select3D RTTI
entities are deactivated. Remove swaps with the last indexed-map item before
RemoveLast and decrements based on the entity's current persistence/flipper state.
Box returns void for persistent entities; Center still uses geometry center and
chooses Z for any axis other than 0 or 1. Methods expose 0-based IDs over 1-based
IndexedMap. Owners skip null handles and unbind when decremented to zero.
SelectableObjectSet has five subsets plus a count sentinel. Its iterator walks
subsets in enum order and More advances across empty subsets. Subset selection
also checks presentations' group persistence/flipping when object persistence is
null. TKMath exports BvhPrimitiveSet, BvhPrimitiveSet3d, and access/builder types;
TKService already has camera-state and transform-persistence APIs. Read CXX bodies
and their exact update rules before implementing the object-set BVH cache.
