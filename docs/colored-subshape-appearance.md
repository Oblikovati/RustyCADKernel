# Martensite colored subshape appearance

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`AisColoredDrawer` embeds the actual `Prs3dDrawer` parent. Upcasts, downcasts,
application descendants and inherited virtual JSON dispatch use that object's
runtime owner. Its native constructor copies the link handle into the parent;
`link_ref()` borrows that actual slot and follows replacement/null assignment
without retaining the target. The existing copied `link()` convenience remains.
Public flags occupy actual writable `SharedState<AisColoredDrawerFlags>` storage;
borrow the state, take a short lease for each observation, and drop guards before
calling setters. No getter mirrors or separate flag caches exist.

All five flags default false. The color/transparency/width setters only mark
ownership: their arguments are ignored even for negative values, infinity or
NaN. They neither validate values nor create or alter presentation aspects.
`AIS_ColoredShape` performs actual appearance changes before marking ownership.
Two native generators repeat byte-identically; Rust replay covers their 28 rows,
including independent ownership transitions, public writes, intrusive counts,
inherited state and aliases. This prerequisite leaves the class record and
broader inherited drawer family partial; no ColoredShape or parent-family credit
is taken. GPU proof belongs to the eventual real ColoredShape caller.

## ColoredShape audit and next implementation seam

No `AisColoredShape` implementation is introduced by this prerequisite. Its
actual `AIS_Shape` parent and Shaded/WF bodies are owned by the TexturedShape
work. Integration must use that real interactive/source protocol, not another
independent drawer or shape wrapper.

- Constructors alias free/unfree/seen/face-boundary line aspects to the owned
  drawer's line aspect. The AIS_Shape constructor copies appearance in material,
  color, width, transparency order, only when the parent has the corresponding
  own property. The shape reference remains backed by native topology storage.
- `CustomAspects` retains arbitrary shape keys, including shapes not in the base
  topology. New entries allocate one linked ColoredDrawer and mark update;
  repeated lookup preserves identity. Map equality follows ShapeMapHasher
  (location and TShape, orientation ignored), not structural geometry equality.
  The later `myshape == key` skip uses orientation-sensitive shape equality;
  located/reversed key fixtures must keep these two comparisons distinct.
- `UnsetCustomAspects(false)` replaces the stored drawer with a fresh instance;
  `true` unbinds. Existing handles keep the old drawer alive. Clearing an empty
  map has no update; clearing a populated map updates after removing entries.
  Const and mutable map references must observe these actual mutations. A
  retained node handle reference must follow replacement while its node exists;
  erased-node/destroyed-map references are native undefined behavior.
- Null-shape custom color/transparency/width calls return before lookup. Other
  values proceed through inherited aspect setters and then mark ownership.
  Native failure cases must preserve newly allocated map entries, partial aspect
  writes and update flags; ignored-argument drawer behavior does not justify
  ignoring invalid values in ColoredShape.
- Global color/width/transparency/material changes skip drawers with the
  corresponding own flag; otherwise they change only already-owned aspects.
  They then invoke the real AIS_Shape base. UnsetWidth calls SetWidth(1), and
  UnsetTransparency calls SetTransparency(0), preserving native own-state quirks.
- External compounds expand to subshape assignments before ordinary explicit
  overrides. Solids/shells bind faces, wires bind edges, and explicit leaf keys
  overwrite inherited assignments. The base shape chooses a base drawer.
- Dispatch excludes hidden overrides and partitions real topology by drawer
  identity and TopAbs type. Closed shell eligibility requires a solid parent,
  native closed topology and triangulation; hidden/translucent custom faces open
  that partition. Vertex/wire and edge/face overrides intentionally suppress
  redundant parent edges. Partial shapes use EmptyCopied plus retained children.
- Wireframe/shaded compute must preserve deflection/tessellation/update order,
  per-type open/closed group order, and genuine Shaded/WF calls. Mode 2 delegates
  to AIS_Shape; other unsupported modes return. Selection has its own subshape
  recursion, owner allocation, hidden state and deflection behavior to prove.

The next oracle matrix needs actual nested compounds/solids/shells/faces/wires,
located/reversed keys, outside-container keys, map/node aliases and replacement,
callback failure/repair, complete group/array ordering and genuine selecting
volumes. Actual retained ColoredShape presentations then need Vulkan/lavapipe
and AMD captures and optimized functional/performance workloads.
