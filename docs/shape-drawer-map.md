# Martensite shape/drawer map prerequisite

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` uses the explicit
`NCollection_DataMap<TopoDS_Shape,Handle<AIS_ColoredDrawer>,TopTools_ShapeMapHasher>`
specialization inside ColoredShape. It has no `AIS_DataMapOfShapeDrawer` typedef.
`AisShapeDrawerMap` is an owned Martensite adapter, with no fabricated upstream
alias, ColoredShape class record, parent-family or generic-map completion credit.

The existing deep DataMap supplies bucket selection, prime growth, rehashing,
head insertion and removal order. Its inline payload moves on growth/removal;
this specialization therefore stores one allocated node containing the actual
original Shape and handle-valued field. Hash/equality borrow that stored key
and delegate ShapeMapHasher. The narrow `find_hashed` prerequisite avoids
constructing temporary Shape owners during borrowed lookups. Replacement keeps
the first key, orientation and location; it changes the actual handle slot.
The native pre-lookup rehash also occurs on a replacement when extent exceeds
the current bucket count.

Node aliases point to actual allocated storage through weak references. They
never own a Shape or drawer and never hold a mutation-blocking lock. UnBind,
Clear and destruction explicitly release the stored key/value fields, including
when a temporary storage lease remains. Const and mutable container adapters
borrow the same map. Key/value snapshots retain ownership only on explicit
request. Erased references fail safely in Rust instead of accidentally following
a rebound equivalent key; native dangling reads are undefined and receive no
invented behavioral credit.

The native corpus uses real BRep vertices, reversed/located keys, an equivalent
numeric transform with a distinct datum identity, and a real AIS_ColoredShape.
It covers pointer/slot identity through rehash and removal, first orientation,
shared TShape flags, copied-key ownership, location storage and datum release,
nullable keys/values, handle replacement and retained unused alias counts.
Two runs of each generator are byte-identical. Rust-only replay preserves the
native map projection, plus safety regressions for erased references.

The ColoredShape lifecycle fixture calls actual native CustomAspects,
UnsetCustomAspects and ClearCustomAspects. It records the raw native parent and
shape baseline separately. The isolated Rust adapter compares added map-owned
references and raw child counts without inventing an AIS_Shape parent or copied
parent facade. It preserves reuse, null-value allocation, replacement versus
unregistration, outside-topology keys and container/node/value reference identity.

The map-only lifecycle helpers report creation/change; presentation invalidation
belongs to the future genuine AIS_Shape caller and its native before/after order.
No API credit is taken for that unfinished caller. Map copy/assignment, allocator
variants, full native iterators and inherited wrapper surfaces are not claimed
complete by this bounded adapter. The full ColoredShape appearance propagation,
selection, topology dispatch and actual rendering remain pending the checked
AIS_Shape/Shaded/WF dependency. GPU captures and performance tests will use that
real caller; this prerequisite introduces no synthetic rendering substitute.
