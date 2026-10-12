# Martensite restricting-face edge references

The bounded `StdPrsToolRFace::edge_ref` port maps the actual defined-visible
`StdPrs_ToolRFace::Edge` declaration at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Together with the previously
proved nine traversal members, this implements the ten core members. The
ToolRFace class stays partial: allocation operators and its class record are
unclaimed. New Explorer/ShapeIterator support receives no parent API credits.

```rust,ignore
let tool = StdPrsToolRFace::with_surface(&surface)?;
let root_reference = tool.edge_ref()?;
tool.init()?;
let edge_reference = tool.edge_ref()?;
tool.next()?;
let observed_edge = edge_reference.snapshot()?;
```

Edge validates `TopoDS::Edge(myExplorer.Current())` when the reference is
acquired. A default explorer's root shape is null, so acquisition succeeds.
After Init, that retained root reference sees a Face. At the end, a new Edge
acquisition raises `Standard_TypeMismatch`; an already retained reference
continues reading its original source slot without repeating that cast.

The pinned Explorer CXX implementation resets popped iterators in place and
reuses the backing slots. Its header's destructor comment is stale and does
not describe the actual pop body. A retained iterator-shape reference observes
successive sibling edges, a null shape after pop, and the new edge when that
slot is reused for another wire or after Init. Root and iterator references
remain distinct even when their values happen to compare equal.

The bounded Explorer prerequisite now retains real `ShapeIterator` slots with
an active depth, resetting them on pop/Clear and replacing them on push. Its
reference accessor reads the iterator's actual inline shape field, including
inactive null slots. It does not keep a separately synchronized edge cache.
The borrowed ToolRFace adapter identifies that source and releases the read
lock before returning. A snapshot explicitly copies metadata at observation
time; the adapter itself owns no shape handle and does not block traversal.
TShape flags remain shared through copied values while copied orientation
metadata is independent, matching native topology ownership.

The constructor reads the nonvirtual parent Face and stores a forward-oriented
value. A same-name application Face member does not intercept it. Source
surface handles can then be released and the source destroyed: the real face
and its curves remain owned through copied topology. The Rust application
source projection borrows that actual parent core; the existing dispatch
interface shares the genuine BRep descriptor rather than adding an ancestor.

Two native generators produced 21 rows twice with identical bytes. Rust-only
replay covers actual rectangular/annular faces, pointer-equivalent slot
identity, shared flags versus copied orientation, complete iteration/reset and
repeated initialization, exact checked cast failure, real parent geometry and
source destruction. Explorer regressions additionally verify root storage,
unchanged source address across same-depth reuse and checked invalid Rust
slot indices. Dangling references after native owner destruction/reallocation
are undefined and are excluded; ordinary valid face traversal uses the native
inline capacity. Rust checked invalid-slot errors are safety extensions and
receive no C++ API credit.

Run both affected toolkits with one command:

```sh
cargo test -p rustycad-tkbrep -p rustycad-tkv3d --all-targets
```

Evidence and the single scoped claim are under
`porting/runtime-evidence/tool-r-face-edge-2026-10-12`. No OCCT code is linked
into Rust tests, no new CI oracle is required, and no raster/performance claim
is made by these CPU topology probes. IsPlanarFace stays with its existing
wireframe owner.
