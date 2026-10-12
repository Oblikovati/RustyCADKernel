# Martensite ColoredShape computation checkpoint

This feature branch remains unfinished. It adds actual OCCT
`AIS_ColoredShape` grouping, presentation and selection construction behavior;
it does not declare the class or its parent families complete.

Local CPU tests exercise real topology, selecting volumes, live custom-map
values, owner/selection argument slots, partial failures and repair. The current
all-target/all-feature TKV3d run passes 904 tests. TKbrep scoped Edge/Face tests,
combined Clippy with warnings denied, formatting and toolkit discovery also pass.

Pinned native probes use OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871` and the existing local compiled
libraries. They are local porting oracles; Rust regression tests read committed
assets and require no OCCT runtime. Native CPU recording drivers do not establish
raster parity.

The native-sensitive output adapters preserve contracts absent from the owning
conveniences:

- `get_sensitive_for_face_into` appends directly into an unlocked stable-node
  sequence. A real curve callback appends to that same sequence before face
  construction finishes; the original first node remains stable, with native
  owner counts. The three-row fixture was captured twice identically.
- `get_edge_sensitive_into` uses the actual intrusive reference slot. The old
  sensitive pointer remains visible while its last owner is destroyed, before
  incoming ownership is installed. The two-row native fixture was captured twice
  identically. Rust inspects pointer identity and weak ownership count during
  retirement; it never projects a dying sensitive object.
- Scoped Edge/Face reads preserve the explicit adaptor topology copies without
  adding hidden owned cast copies. Real curve evaluation observes two topology
  owners, matching native, and one after completion.
- Virtual `SetSelectable` receives the original live argument reference.
  Overrides can replace that argument before invoking the base body. Typed
  native failures retain partial state, while unrelated panic payloads resume.

Remaining work includes current-source declaration/preflight evidence, an audit
of presentation argument replacement during actual native driver callbacks, and
real retained ColoredShape rendering captures and optimized workloads on both
lavapipe and the native host. Existing BRepOwner/StdSelectShape highlighting and
other parent-family surfaces remain partial. This checkpoint adds no API ledger
rows or class-completion credits.
