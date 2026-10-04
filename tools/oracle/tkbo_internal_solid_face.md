# Internal face in solid

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
160 rows cover a box's boundary face translated to five positions, all solid
and face orientations, and present/empty edge-to-face adjacency maps.
Each row compares failure status, classification, and retained map extent.
Replay: `cargo test -p rustycad-tkbo --test internal_solid_face`.

The source translation skips internal and degenerate edge occurrences, uses
angular classification for two adjacent faces, and falls back to ComputeState
with the solid's edge bounds. A singleton internal-edge support terminates
the angular scan even when ambiguous; a two-face ambiguity continues scanning.

Partial: singleton internal-edge support, non-manifold adjacency, curved and
freeform solids, malformed topology, and complete legacy/API parity remain
unproven. The Rust adjacency argument is borrowed immutably because this source
routine only reads the native mutable map.
