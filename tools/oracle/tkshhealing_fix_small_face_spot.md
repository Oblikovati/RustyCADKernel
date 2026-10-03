# `ShapeFix_FixSmallFace` native oracle

`tkshhealing_fix_small_face_spot.cxx` probes a regular box solid. OCCT returns
the same topology and keeps `Shape()` bound to the original input. This exercises the
public constructor/`Init`/spot scan/result path. A manually built tiny circular face
(radius `1e-8`) and a regular box in a compound provide the positive `tiny_and_regular`
case: the spot is removed while all six box faces remain. An initial attempted
sub-confusion `BRepPrimAPI_MakeBox` failed during fixture construction, before the
fixer ran.

The `tiny_and_regular` row records output identity and face/edge/vertex occurrence
counts after the removal and cleanup path. `thin_solid` exercises `FixStripFace` with
a 1-by-1e-4-by-1e-4 box at precision 1e-3; the two thin dimensions collapse to the
shared-edge midpoint while three faces remain. Its vertex bounds are part of the row.

The Rust port implements the spot and strip phases, including shared-edge endpoint
matching and adjacent-face p-curve transfer. `Perform` invokes spot then strip in native
order. Empty-container `FixSplitFace` and empty-face `SplitOneFace` results are pinned.

The nonempty split path remains partial. Native `CheckSplittingVertices` reads `myPrecision`,
which its constructor leaves uninitialized and `ShapeFix_FixSmallFace` exposes no setter for.
A pinned regular-box run happened to return six faces unchanged, but that does not establish
defined behavior; Rust returns an explicit construction failure instead of guessing a
tolerance. Recursive face-split construction is unported and no parity claim is made for this
domain.
