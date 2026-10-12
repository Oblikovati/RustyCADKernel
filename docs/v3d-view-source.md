# Martensite V3d view subclasses and native references

`V3dViewSource` exposes the four real native virtual operations: Redraw,
RedrawImmediate, AddClipPlane and RemoveClipPlane. A subclass embeds an owned
`V3dView`, initialized with `new_embedded`, then binds its final handled object
with `bind_source`. It does not retain a separate concrete core view. The
Source and concrete class share the same runtime descriptor; registered casters
project the actual subclass and its actual embedded base.

```rust,ignore
let base = V3dView::new_embedded(&viewer, V3dTypeOfView::Orthographic)?;
let derived = Handle::new(MyView { base, callbacks });
let source: Handle<dyn V3dViewSource> = derived.cast();
V3dView::bind_source(&source)?;
source.redraw();
```

Qualified body adapters execute the inherited view behavior, including active
checks, device-loss retry, Z fitting, clipping and immediate invalidation.
Nonvirtual members execute on the actual embedded base. Their internal redraw
calls dispatch through the weak Source identity. Typed failures from void
overrides use the existing native failure bridge; unrelated panics are not
converted into native exceptions.

The viewer stores actual `NCollection_List` nodes containing generalized view
handles. `active_views_ref` and `defined_views_ref` borrow those lists;
their detached cursors expose live node-handle references. Call `next` after a
callback so appends and removal of other nodes remain visible. Replacing the
current node's handle preserves its identity, and an existing `value_ref`
immediately observes the new target. Snapshot conveniences explicitly acquire
ownership and are distinct from the native reference contracts.

`V3dViewMembers::camera_ref` forwards the actual virtual DSM/CView camera getter.
The getter selects a genuine handle slot; an old alias stays attached to that
slot even when later getter calls select another one. Camera and clipping
parameters borrow live `HandleReference` inputs. Viewer activation retains the
native original driver-view copy and rereads its view argument after callbacks.
XR setup and teardown continue to assign native `myCamera` storage directly.

The committed CPU corpus contains 35 records from OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`: six subclass/clipping rows, ten live
node rows, twelve failure rows, five camera-alias rows and two argument-retargeting
rows. All generators ran twice with byte-identical outputs. Rust tests replay
the assets without OCCT. Original objects and replaced targets remain strongly
owned; dangling reference/current-node removal cases are outside native validity.
Whole-class completion is not claimed by this bounded subclass/reference slice.

```sh
cargo test --locked -p rustycad-tkservice -p rustycad-tkv3d --all-targets
```


The actual Vulkan cube scene embeds a derived `V3dViewSource`, invokes both real
redraw overrides on its mapped view, and forwards its public Camera lookup to
the actual DSM implementation. OpenGl render extraction uses `myCamera`
directly (pinned `OpenGl_View.cxx:2527`); Martensite's `view_render_camera` reads
that same owned field. A selected alternate public Camera slot therefore does
not alter renderer projection. Rust regression also proves XR setup/unset changes
the owned field without changing the selected alternate slot.

Both lavapipe and AMD functional runs produce all eighteen buffers byte-identical
to the prior checked camera-transport evidence. The unchanged strict native
RGBA/depth gates still fail for all eighteen cases, as recorded in
`porting/runtime-evidence/v3d-view-source/strict-comparisons.json`; no raster
baseline or tolerance was accepted. Exclusive optimized timing uses the existing
eight cube/camera workloads, twenty warmups and one hundred samples on each
profile, with complete source/binary input manifests and raw vectors retained.

Viewer activation's active-light storage remains a copied vector. Its temporary
owners are dropped before Redraw, but full native live-light iteration and
callback ownership are not proved here. Both SetViewOn overloads and whole
class/constructor contracts remain partial. The four focused fragments contain
nineteen visible pinned method bindings; they do not claim private helpers or
unchanged nonvirtual members on arbitrary subclasses.
