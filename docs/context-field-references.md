# Martensite context field references

`current_viewer_ref()` and `main_prs_mgr_ref()` return references to the actual
context handle fields. They release their brief read guards before returning,
so a native-valid callback can change the context while retaining the reference.
For example, retain `context.current_viewer_ref()`, call a setter on the context,
and use `reference.observe()` to read the current target without owning a copy.
The existing `current_viewer()` and `main_prs_mgr()` read-guard conveniences are
separate APIs and must not be held across context mutation.

The virtual entity-owner `update_highlight_trsf` arguments now accept
`&dyn HandleReference<V3dViewer>` and
`&dyn HandleReference<PrsMgrPresentationManager>`. Ordinary `Handle` arguments
continue to work. The directional-light call forwards the genuine context fields;
its other drag behavior and coordinate interface are unchanged in this slice.

Six pinned native CPU rows exercise actual owner overrides replacing both fields,
calling the public `SetDefaultDrawer`, raising typed or unrelated failures, and
retrying. Both native runs are byte-identical; Rust-only replay compares every
field identity, intrusive count and failure state exactly. There is no OCCT
runtime dependency in the Rust tests. This restores only the two getter contracts;
context, controller and presentation classes remain partial.
