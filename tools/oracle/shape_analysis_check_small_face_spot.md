# ShapeAnalysis_CheckSmallFace::IsSpotFace oracle

`shape_analysis_check_small_face_spot.cxx` calls the pinned OCCT method directly on
faces without wires, with an empty wire, with a repeated-vertex circular edge, with a
loop whose curve midpoint exceeds the tolerance, and with tiny and regular box faces.
Each row records the status, output point, and output tolerance. The sentinel output
values on `no_wire` and rejected faces verify that OCCT leaves both output arguments
untouched on those paths. `empty_wire` records the native `Precision::Infinite`
arithmetic, while `local_tolerance` exercises the negative-tolerance vertex branch.

The Rust slice implements only `IsSpotFace`; other public methods and status accessors
on `ShapeAnalysis_CheckSmallFace` remain unported. The empty-wire behavior is retained
as a defined native result, including its large finite sentinel coordinates.
