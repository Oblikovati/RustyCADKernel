# ADR 0076: Stable body topology indices for read-only queries

Status: accepted (2026-10-15). Consumer: the application's "first cube" slice
(presentation and picking of any owned body).

## Context

Presentation meshes and pick results must name faces, edges and vertices without
exposing live `Shape` handles to the application. The name must be the same on
every run and thread for the same body. Handle addresses, pointer values and
hash-map iteration order are not stable, so they cannot define it. No shared
enumeration existed on the parallel worktree branches when this was written.

## Decision

`rustycad_kernel::topology::BodyTopology::enumerate(&Shape)` numbers the distinct
faces, edges and vertices of a body from zero, in the `TopExp_Explorer` order of
the body (depth first over the stored child order), deduplicated by `IsSame` in an
indexed map whose indices are insertion order. `FaceIndex`, `EdgeIndex` and
`VertexIndex` are `u32` newtypes. `face(index)` returns the face as first met,
with its orientation in the body; `face_index(&shape)` accepts either orientation.

The order depends only on the body's topological structure. Two separately built
but identical bodies enumerate the same way, and so does a copy made with
`BRepBuilderAPI_Copy`. Consumers that need the copy map entities through the copy
history, not by position (see ADR 0077).

These indices are display and query keys for one body revision. They are not
persistent names: a modelling operation can renumber them. Persistent naming is
still owned by the operation history and naming layer.

Failures use `query_error::BodyQueryError` with operation, entity, class
(`NullBody`, `Native`, ...) and cause.

## Evidence

Unit tests in `crates/kernel/src/topology_tests.rs`: box 6/12/8 and cylinder
3/3/2 counts (the seam once), round trip of every index, reversed-face lookup,
a foreign body's face not found, identical vertex order for two separately built
boxes, and a null-body refusal. `BodyTopology` is asserted `Send + Sync` at
compile time.

## Limits

The enumeration renumbers when the structure changes. The unmerged `extrude`
`OperationResult` (branch `worktree-agent-a7079cb6ed3b8e56a`, ADR 0068) indexes its
solid with the same rule (`topexp::map_shapes_of_type` in insertion order). So
`ResultEntity::Face(i)` there and `FaceIndex(i)` here name the same face of that
solid. When both are on main, one implementation should serve both, with a test
that pins the agreement.
