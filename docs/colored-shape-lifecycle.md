# Martensite ColoredShape lifecycle

This bounded slice owns the actual `AIS_Shape` parent fields and a native-shaped
custom-drawer map. It ports the topology constructor, virtual `CustomAspects` and
`ClearCustomAspects`, nonvirtual `UnsetCustomAspects`, the two live map-member
references, the protected map field and the concrete class's own RTTI.
`AIS_ColoredShape` and `AIS_Shape` remain partial.

`AisColoredShape::new(&shape)` retains located/oriented topology, initializes
native UV/bounds/cache defaults, and shares the parent line-aspect handle with
free boundary, unfree boundary, seen line and face boundary aspects. Runtime
projections borrow one owned parent/subclass state and share one intrusive count.

The source protocol preserves virtual application overrides. Explicit
`ais_colored_shape_custom_aspects` and `ais_colored_shape_clear_custom_aspects`
functions call the native core body without redispatch. Rust source callbacks
return typed failures; a failure after the core operation preserves the map and
presentation status already changed by that operation.

Presentation invalidation follows pinned OCCT `3d097a0`:

- Creating or filling a null custom value binds before marking presentations.
- Reusing a nonnull value leaves clean presentations clean.
- Clearing a populated map releases its children before marking presentations.
- Unsetting a bound value marks presentations before replacing or removing it.
- Missing unsets, empty clears and raw map mutation do not mark presentations.

Native `NCollection_BaseMap::destroy`, lines 72–97, preserves `mySize` until node
destruction finishes. The Rust generic `ChainedSlots::clear`, lines 26–30, takes
its payload before destruction; its payload length cannot express this native
scalar while the specialization releases its lock for child callbacks.
`AisShapeDrawerMap` therefore owns the observable extent scalar. All extent,
empty and container-view queries read it. UnBind updates it before field release;
Clear resets it after native-order release. Removed node aliases remain checked
and cannot keep actual key/drawer ownership alive. The derived map is declared
before the parent state because Rust destroys fields forward: custom drawers
are released while the native parent topology/drawer ownership is still alive.
The parent shape fields likewise precede its interactive base so topology
destruction observes the still-owned inherited drawer; constructor evaluation
remains parent-first in both cases. A real BRep vertex destructor regression
reproduced the incorrect count 1 before correction and matches native count 2.

The native owner oracle observes destructor-time map extent, dirty status,
TShape count and linked parent-drawer count. The replacement path drops the
Rust owning Attributes convenience before assigning the replacement, so that
convenience does not add an observable reference during old-drawer destruction.
Destructor-driven map mutation and reads of native dangling node references are
outside the proved scope. Valid retained scalar queries are covered.

`AisShapeMembers` also exposes bounded live Shape/Attributes slot adapters. They
observe assignment without retaining a topology/drawer or a lock; explicit
`clone_shape` creates an owning copy. These parent prerequisites do not promote
the parent family to complete.

Committed native CPU fixtures run twice byte-identically and replay in Rust
without OCCT. The named CPU driver allocates real presentations; these recordings
are not rendering captures. Copy construction, custom/global appearance,
subshape expansion, Compute, selection and rendering/performance remain open.
The concrete geometry entry points return typed `Standard_NotImplemented`
failures identifying the requested mode. No class, parent-family or geometry
completion credit is taken for those adapters.
