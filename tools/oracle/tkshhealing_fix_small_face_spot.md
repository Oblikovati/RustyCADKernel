# `ShapeFix_FixSmallFace::FixSpotFace` native oracle

`tkshhealing_fix_small_face_spot.cxx` probes a regular box solid. OCCT returns
the same topology and keeps `Shape()` bound to the original input. This exercises the
public constructor/`Init`/spot scan/result path without entering the tiny-face repair
branch, whose single-face box fixture raises a native `Standard_DomainError` during
`FixShape`. A manually built tiny circular face (radius `1e-8`) and a regular box in a
compound provide the positive `tiny_and_regular` case: the spot is removed while all
six box faces remain. The initial attempted sub-confusion `BRepPrimAPI_MakeBox` failed
during fixture construction, before `FixSpotFace`, and is not treated as a native
fixer failure.

The `tiny_and_regular` row records output identity and face/edge/vertex occurrence
counts after the removal and cleanup path.

The Rust port currently implements the spot-face phase and its vertex/context helpers.
Strip-face replacement, recursive face splitting, and the aggregate `Perform` path
remain pending.
