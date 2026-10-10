# Sensitive debug display

`SelectMgr::compute_sensitive_prs` translates the pinned OCCT `3d097a0`
selection debug presentation. It appends real graphics groups for boxes,
cylinders, circles, faces, polylines and derived curves, supported wire edges,
points, segments, triangles, and triangulations with free edges. Unsupported
entities still participate in flipping callbacks. Prior groups are retained.

The helper preserves native floating-point operation order, incremental circle
sampling, edge ordering, point/line/free-edge group order and styles. Single
triangles shrink transformed vertices around their original center, matching
the upstream discrepancy; triangulations calculate the center after applying
the object and initial-location transforms. `Select3DSensitivePolySource`
provides the shared polyline base for concrete polylines and sensitive curves,
including their native runtime-type parent relationship.

`SelectMgrViewerSelectorDisplayMembers` implements both display overloads.
`display_sensitive` creates a structure per recorded selectable, includes only
activated selections, then redisplays every retained structure at Topmost
priority and updates the view. `display_sensitive_selection` optionally clears
old structures first, renders the explicit selection regardless of activation,
appends it, displays it and updates the view. Structures append only after
successful geometry construction. Callback failures retain preceding effects.

No selector or selection guard spans a graphics callback. The sensitive array
iteration keeps its original end, as native DynamicArray iterators do; the
object selection loop observes selections appended by a flipping callback.
The retained debug-structure loop uses the existing sequence cursor.

Six direct tests cover all geometry kinds, inherited polyline access, repeated
and inactive selection displays, replacement/cleanup, partial failures and
callback-driven appends. A recording null driver captures actual native and
Rust aspects, float vertex buffers, edge indices, flipping callbacks and
retained structure state. The differential corpus contains 4,320 cases,
including translation, rotation, negative scale, persistence, repeated calls,
empty selections and injected group-creation failures.

Run `cargo test -p rustycad-tkv3d --all-targets` and
`python3 tools/verify_toolkit.py TKV3d --source ../OCCT --native-build target/occt-all-full`.
All declared non-boilerplate ViewerSelector APIs now have implementation
mappings; the overall OCCT port remains incomplete.
