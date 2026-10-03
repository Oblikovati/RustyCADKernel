# ShapeFix vertex positioning

Oracle: clean OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkshhealing_vertex_position.cxx` links the configured native TKShHealing build;
its 14 rows are retained in `tkshhealing_vertex_position.expected`.
`shape_fix_vertex_position.rs` compares return flags, original vertex tolerance,
and each ordered edge endpoint's X/Y coordinates and tolerance.

Cases cover empty/single-edge input, a vertex below the requested tolerance,
missing first-edge geometry, first/last shared endpoints, incremental midpoint
updates, rejection and recursive regrouping, separating closed edge endpoints,
loop-edge reordering, and grouping the two ends of a closed edge separately.
The original vertex tolerance remains unchanged: the native attempted decrease
uses BRep_Builder::UpdateVertex, which only increases tolerance. New vertices
use Precision::Confusion. Existing edge orientation and geometric representations
are retained by ShapeBuild_Edge::CopyReplaceVertices.

Native code dereferences a null reshape context on a repair path. Rust returns
an explicit Standard_NullObject failure; the no-repair empty path accepts null.
This is an unapproved failure-domain difference, not a completed parity claim.
Internal non-endpoint vertices, transformed/reversed compound placements and
prepopulated replacement contexts are not comprehensively covered by these rows.
The fixtures are additional differential tests, not mapped legacy declarations.
