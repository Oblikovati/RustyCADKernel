# Martensite sensitive group selection

The captured, defined-visible `Select3D_SensitiveGroup` family is complete at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: 34 unique declarations comprise
29 methods, three RTTI methods, one base-type alias and the class record. The
original twenty core/reference bindings remain independently verified. The
completion adds nine methods and the class record, and replaces the previously
excluded native `base_type` row with the real public
`Select3DSensitiveGroupBase = dyn Select3DSensitiveSet` alias. Six private fields
are not API-visible and receive no credits. Copied membership/element
conveniences remain unclaimed; their native reference contracts use live views.

Groups distinguish handles by identity. Repeated handles coalesce; equal geometry
in separate handles remains distinct. Sequence construction averages distinct
centers. Single addition takes the running half-sum; sequence addition adds half
of its newly inserted center sum divided by total membership, including existing
members. A duplicate-only sequence therefore changes the cached center. Removal
swaps the last member into the removed index and recomputes a mean. Native empty
sequence construction and final removal divide zero coordinates by zero and
retain NaNs; Clear explicitly resets the center to zero.

Nonpoint all/check-all matching calls the selection-type callback twice, reading
each corresponding policy after its respective callback. It reuses one child
pick, copies successful minima with native `<=` tie/NaN semantics, preserves
successful earlier output writes on a later miss/failure, and does not reset the
prior group detection index. Check-all continues after child misses to update
child detection state. The ordinary sensitive-set route retains deferred BVH
construction, native traversal order and strict nearest-depth behavior.

Native Add/Remove/Clear operations do not mark retained BVH content dirty. Cached
content bounds/tree may therefore remain stale until explicit MarkDirty. The
corpus records this behavior; it does not silently rebuild after each mutation.
Safe full-inside/empty paths observe post-mutation state, followed by explicit
repair and real-volume retesting. Unchecked stale dynamic-array accesses remain
outside native defined behavior and are excluded from parity claims.

Child bounds/center callbacks can throw after earlier map, primitive-index, cache
or center writes. Owner propagation likewise updates the group and earlier
children before a later callback fails. Connected copies reconstruct separate
children, retain owner identity and must-match mode, reset check-all mode and
sensitivity, and inherit native sequence-add center policy. Nonfallible trait
hooks use the existing typed Failure bridge; fallible matching/BVH APIs catch
only Failure and resume unrelated Rust panics.

The live container adapter observes membership mutations through a borrowed map
view without retaining locks. The subentity adapter binds an allocated node,
so a surviving reference still identifies the same child after rehashing and
removal of other nodes relocates its numeric index. Two aliases observe shared
child owner mutation. Checked one-based map access reports the offending index.
Native equivalence applies while the referenced node survives; aliases after
node removal/Clear/destruction are dangling upstream and receive no claims.

Six local native generators produced 6,912 actual-volume snapshots, fourteen
mutation snapshots, 36 fault/repair/retest snapshots, 320 changing-type/policy
cases, 28 primitive snapshots and nine retained-reference rows. All repeated
byte-identically. Numeric evidence includes mixed circles/segments/spheres,
multi-leaf groups, orthographic/perspective cameras, clipping, inclusion/overlap,
preseeded outputs, equal/NaN depths, arbitrary center-axis fallback and owner/copy
identity. Rust-only regressions load bounded committed assets without OCCT,
external I/O, a native compiler or a graphics device. Provenance hashes and
scoped canonical bindings are in the batch runtime receipt.

Application groups compose the core state and implement
`Select3DSensitiveGroupSource`. Shared qualified-call adapters preserve native
virtual dispatch to overlap, inclusion, distance and geometry overrides.
`select3d_group_downcast` returns a source interface retaining those overrides
and the original object identity. An application subclass without its own
native RTTI retains the Group descriptor, its SensitiveSet parent and all
ancestor membership; the public base alias projects the same object. Default
constructor arguments, null structural state and inherited protected owner,
sensitivity, detection and BVH mutations are covered by native fixtures.

JSON diagnostics preserve native fragment bytes, including duplicate class
keys, depth-dependent recursion and negative NaNs. Only unstable native pointer
addresses are normalized in the oracle. Child output is buffered: a failing
child contributes no partial nested fragment, while the parent prefix and
previous children remain. Native checked failures, stream failures, callback
policy mutation and earlier bounds-cache writes are preserved. Application
geometry overrides affect virtual base diagnostics and distance; the final
Group fields still describe the core's own stored state.

Completion evidence adds 1,152 qualified/application hook snapshots (2,304
rows), thirty ordinary JSON cases, thirty JSON failure/mutation cases, twenty
application JSON/distance cases, four default/RTTI sequences and three inherited
protected-state cases. The four generators repeated byte-identically, producing
2,460 rows. Production crosswalk and claim preflight verify all 34 unique
bindings against pinned compiler visibility and fresh canonical Rustdoc source
hashes. This closes this class's captured surface, without claiming completion
of the broader SensitiveSet, Entity or visualization toolkit families.
