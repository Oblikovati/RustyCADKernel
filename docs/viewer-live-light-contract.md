# Martensite viewer live-light contract

The pinned OCCT source `3d097a0328e71b826377d4814ab05ec3c3d23871`
retains `NCollection_List<Handle<Graphic3d_CLight>>` fields in the viewer and
view. Its detached and deprecated iterators identify physical list nodes.
A callback can replace the current node's handle, remove a different node,
or append a node without replacing the current node's storage. Subsequent
native argument uses reread that storage; a copied `Vec<Handle<_>>` cannot
preserve either this traversal or its intrusive reference counts.

Martensite uses the existing allocator-backed list and cursor types for
these fields. `V3dLightListRef` borrows the actual field;
`V3dLightNodeRef` implements `HandleReference` over its actual handle node.
The common `V3dHandleChainRef` mechanism also serves view nodes, avoiding a
second implementation of the existing live view-list contract. List reads
release their storage lock before invoking external view or driver methods.
The explicit owning `active_lights()`, `defined_lights()`, `active_light()`,
and `defined_light()` conveniences remain distinct from native reference
getters and are not bindings for those getters.

`V3d_View::UpdateLights` intentionally creates one owning local light-set
handle, adds the current active-list keys, and passes that local handle by
reference through the actual pure virtual `Graphic3d_CView::SetLights`.
`lights_ref()` observes the actual retained driver field, including null and
replacement assignments. The owning `lights()` convenience is not the
native const-reference contract. Strong original target handles in the
fixtures exclude dangling-object undefined behavior from alias tests.

There are two relevant membership rules. `IsActiveLight(null)` returns
false. `SetLightOn()` instead checks list membership including null, so a
repeated call must not append a second null node after `UpdateLights`
throws `Standard_ProgramError`. The append preceding that failure remains
observable. Global-light rejection in `SetLightOff(light)` occurs before
removal. The native no-argument viewer `SetLightOff()` calls the individual
view operation before clearing its global list, so an active global light
can cause `Standard_TypeMismatch`; Martensite preserves this ordering.

Viewer activation intentionally owns its local driver-view handle across
callbacks, while rereading its caller's view reference for light setup,
grid setup, and redraw. Native SetLights callbacks can replace the caller
view slot, change active or defined light nodes, change active view nodes,
or throw before retained-light assignment. The replay fixtures preserve
these observable mutations and the original reference-count traces.

The activation list insertion also requires evaluating the referenced
handle before taking the viewer write guard. Rust method-call receivers
are evaluated before their arguments; taking that guard first deadlocks
when the argument is a node reference into the same viewer. The standalone
activation regression and fix are integrated separately by the coordinator.
This differs from ordinary Rust assignment, whose right-hand side is
evaluated before the left-hand place; the earlier camera self-assignment
probe proves the original assignment is already safe.

## Local evidence

`tools/oracle/martensite_viewer_live_lights.cxx` and its named recording
driver substitute drawing I/O only. The real native viewer/view bodies,
light-set keys, retained lists, virtual callbacks, and ownership counts
produce twelve bounded cases. The Rust-only replay uses actual embedded
`V3dViewSource` bases and actual retained driver storage.

`tools/oracle/martensite_viewer_light_aliases.cxx` exercises physical node
aliases and retained driver light-set aliases, including self-assignment.
`tools/oracle/martensite_viewer_light_edge_cases.cxx` covers repeated null
membership failures and callback replacement of a borrowed caller light-set
slot. Final run, source, library, and executable provenance belongs in the
runtime receipt; authored probes are not a claim that final gates passed.

Removing the currently referenced node, dangling owner storage, or a
reallocated native alias has no successful native contract. Martensite
emits a typed diagnostic for an invalid node rather than claiming parity
with that undefined behavior. No viewer/view class or constructor is
completed by this bounded family. SetViewOn claims remain partial until
all scoped callback and failure-order proof is complete.
