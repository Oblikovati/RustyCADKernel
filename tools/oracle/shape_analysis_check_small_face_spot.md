# ShapeAnalysis_CheckSmallFace::IsSpotFace oracle

`shape_analysis_check_small_face_spot.cxx` calls the pinned OCCT method directly on
faces without wires, with an empty wire, with a repeated-vertex circular edge, with a
loop whose curve midpoint exceeds the tolerance, and with tiny and regular box faces.
Each row records the status, output point, and output tolerance. The sentinel output
values on `no_wire` and rejected faces verify that OCCT leaves both output arguments
untouched on those paths. `empty_wire` records the native `Precision::Infinite`
arithmetic, while `local_tolerance` exercises the negative-tolerance vertex branch.

The Rust slice currently implements `IsSpotFace`, `CheckSpotFace`, its status query,
and the tolerance setter/query. The other analysis methods and status families on
`ShapeAnalysis_CheckSmallFace` remain unported. The empty-wire behavior is retained as
a defined native result, including its large finite sentinel coordinates.

The final `check_spot` row exercises `CheckSpotFace` status persistence and `SetTolerance`/
`Tolerance`. OCCT's overall `Status` and initial `Tolerance` fields are uninitialized by
its constructor, so Rust exposes those values as `None` until a defined setter or check
assigns them; it does not synthesize a value for native indeterminate state.
