# `ShapeFix_FixSmallFace::FixSpotFace` native oracle

`tkshhealing_fix_small_face_spot.cxx` probes a regular box solid. OCCT returns
the same topology and keeps `Shape()` bound to the original input. This exercises the
public constructor/`Init`/spot scan/result path without entering the tiny-face repair
branch, whose single-face box fixture raises a native `Standard_DomainError` during
`FixShape`; that failure domain is not counted as green parity.

The Rust port currently implements the spot-face phase and its vertex/context helpers.
Strip-face replacement, recursive face splitting, and the aggregate `Perform` path
remain pending.
