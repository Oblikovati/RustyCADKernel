# Face splitting by outside-wire map

OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Ten rows compare return, Result nullity, face/wire counts, working-face identity
and orientation, DONE status, context replacement and output face/wire orientations.
Cases cover two outside contours, missing map entries, an open contour, an empty
contour, one contour, reversed face, either hole orientation, an INTERNAL contour,
and a reshape context.

The method counts eligible input wires and mapped output wires, rejects an open
mapped contour, or returns false when fewer than two faces result. Both initial
hole orientations become reversed. INTERNAL wires are excluded. Successful split
updates Result to a compound and leaves Face as the last explored output face;
it does not set DONE status itself. The translation preserves mutation ordering.

These fixtures exercise planar polygons. Curved/located contours and classifier
failure cases require further differential evidence. No legacy-test mapping or
full behavioral parity is claimed.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_face_split`.

Eight additional rows call Perform with FixWireMode=0 and compare all eight DONE
bits as well. The cases cover two outer contours, reversed face, both hole
orientations, a single contour, context replacement, prior removal and prior
replacement by a non-face. Aggregate processing follows native periodic/seam,
intersection, orientation, natural-bound, split and small-area stage ordering.
The default wire-enabled aggregate and its progress overload remain incomplete;
periodic/seam and small-area aggregate combinations require additional fixtures.
