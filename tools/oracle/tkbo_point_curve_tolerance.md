# Vertex-on-curve tolerance correction

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_point_curve_tolerance.cxx` records 162 CorrectPointOnCurve cases using
endpoint and internal vertices, three discrepancies, three maximum tolerances,
three avoidance modes, three location modes, and both native scheduling flags.
Replay: `cargo test -p rustycad-tkbo --test make_pcurve point_curve_tolerance`.

The translation visits every 3D curve representation, matches stored vertex
parameters by curve identity and relative location, and also checks oriented
endpoints. It snapshots the initial maximum edge/vertex tolerance for the
squared threshold and 10% margin, and applies the strict maximum-tolerance gate.
Fixtures cover an edge translation and a separate endpoint translation.
Avoiding the edge alone does not prevent updates to unavoided vertices.

Rust executes this stage serially for both requested scheduling modes. Multiple
spatial representations, arbitrary curve types and complete legacy/API parity
remain unproven. The mapping is partial.
