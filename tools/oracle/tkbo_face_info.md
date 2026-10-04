# BOPDS face classification initialization

Pinned oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`BOPDS_DS.cxx` HasFaceInfo, FaceInfo, ChangeFaceInfo, InitFaceInfo,
InitFaceInfoIn, UpdateFaceInfoOn and FaceInfoOn.

The 36-row native box fixture covers six faces across six stages: empty reads,
updates before initialization, lazy initialization, edge pave-block initialization,
explicit boundary refresh and repeated mutable access. Every row includes the
face pool index, presence flag, boundary block and vertex counts, internal vertex
count, and native vertex-map iteration order. Rust separately verifies shared
empty-record identity and explicit repeated initialization appending a new record.

This is a partial API slice. Internal vertex fixtures, common-block aliases,
In-map interference refresh, set overloads and refinement remain to be verified.
Supplemental fixtures are not mapped legacy test declarations. Invalid pool
references return Rust failures; native undefined access is not claimed equivalent.

Build the probe against the pinned native modeling installation, linking TKBO,
TKPrim, TKTopAlgo, TKBRep, TKMath and TKernel. Compare stdout to the expected file.
Rust replay: `cargo test -p rustycad-tkbo --test bop_ds_face_classification`.
