# Wireframe small-edge analysis

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Two state rows check default construction and Load/ClearStatuses retaining the
configured drop flag and angle. Ten analysis rows check return, four output-map
sizes, adjacency counts/orientations, status and shape identity.

Cases cover a polygon with one short edge, a standalone edge, repeated same-
orientation edge, opposite-orientation seam, empty wire, ordinary/reversed face,
a wire shared by two faces and preloaded small-edge sets. The analyzer does not
clear caller maps. Empty free wires return false even with preloaded small edges.
A standalone edge is not explored. Repeated non-seam edges are classified as
multiple instead of small. Face adjacency retains original face orientation.
CheckSmallEdges does not change the tool's repair statuses.

Curved/located/degenerate inputs and exceptional mutation behavior need more
oracle coverage. FixWireGaps, FixSmallEdges and MergeSmallEdges are not yet ported.
No legacy-test mapping or full wireframe behavioral parity is claimed.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_wireframe`.
