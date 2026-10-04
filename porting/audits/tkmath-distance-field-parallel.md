# BVH distance field parallel execution audit

The pinned `BVH_DistanceField::Build` sends each Z slice through
`OSD_Parallel::For` when `IsParallel()` is true. Rust now uses the OCCT worker
pool for disjoint Z planes after snapshotting the already-clean geometry and
triangulation trees. If a relevant tree is dirty, it keeps the existing
serial traversal and lazy-build order instead of sharing mutable BVH caches.

The detailed replay is in `tkmath-distance-field-parallel.json` and
`tkmath-distance-field-parallel-cases.jsonl`. The JSONL records all 48
deterministic `append_field_requests(random.Random(20140911), ...)` inputs,
per-request hashes and flags, and bounded native parallel outcomes. The JSON
records the compiler/link profile and summary. Native serial and both Rust flag settings match on every
request. All 16 prebuilt cases have bit-identical native serial and parallel
whole outputs, including active voxels. The additional zero-capacity case also
matches exactly.

The native parallel path is unstable when it lazily builds shared BVHs. In one
bounded sample, 18 of 48 native parallel invocations crashed, aborted or timed
out; one successful dirty-tree case had a different active voxel from native
serial. Repeated probes for cases 26 and 31 alternate between SIGSEGV and
successful output, so these counts are observations rather than rates. The
case list, per-process status, and replay receipts are in the JSONL file. This is an upstream concurrent-lazy-build behavior gap, not
a Rust parity pass for dirty-tree parallel execution. The serial fallback
retains the pinned serial behavior and avoids sharing mutable lazy caches.

The evidence uses OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Native source SHA-256 and probe build command are recorded in the JSON receipt.
