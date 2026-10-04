# Differing-offset tangent-face analysis

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Twenty native configurations cover coincident Bezier support faces, adjacent
planar rectangles sharing one edge, and rectangles sharing two connected edges.
The rectangle cases also include an additional perpendicular side face, exercising
minimum-face edge selection and generated side-edge orientation adjustment.
Each uses global offset 1 with first-face overrides 1, 2, -2, and 0. The 60 rows
also compare repeated Perform and Clear followed by Perform.

Rust compares artificial-face counts/orientations, source-edge classifications,
replacement identity, generated vertex presence, vertex ancestor counts and stored/generated edge orientations, every
new face edge's ancestor/classification counts, and descendant counts exactly.
The connected case constructs two artificial faces with a shared side edge.
Repeated Perform preserves prior classifications while clearing generated topology;
Clear additionally drops the face-offset map, matching native behavior.

The implementation translates selection, coplanar block grouping, prism creation,
orientation, generated side-edge ancestry, opposite-edge replacement and reanalysis.
Descendants are rebuilt in native ancestor insertion order after mutation.

These cases do not prove all generated-shape identities or ordering, noncoplanar
blocks, more complex side-face ancestry, curved tangencies, locations,
progress cancellation or exceptional partial state. All affected API claims remain
partial. These supplemental fixtures do not complete an OCCT legacy regression.
