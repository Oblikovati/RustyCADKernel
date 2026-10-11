# Martensite sensitive primitive initialization

Martensite exposes all five native `Select3D_SensitivePrimitiveArray`
initialization overloads: ranged and whole-buffer triangulations, ranged and
whole-buffer indexed points, and sequential vertex-only points. The buffers stay
aliased; the initialization range indexes the index buffer rather than limiting
its vertex values. Positions retain the native Vec2/Vec3/Vec4 interpretation;
Vec4 weights do not divide the position used for picking.

The Rust-only regression suite replays 339 committed native snapshots generated
locally from OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. Cases cover
sequential and reversed u16/u32 indices, ranged and rejected input, patches of
neighboring primitives, subgroups, eager/lazy bounds and translated initialization
locations. Native subgroup partitioning, including discarded division remainders,
remains unchanged. Bounding boxes include every buffer vertex, independently of
the selected range. Subgroup bounds retain native repeated location application.

Three previously untested behaviors now have explicit regressions:

- Grouped `InitPoints` adds the previous center before dividing by subgroup count;
  `InitTriangulation` resets that center. Reinitialization preserves this native
  behavior instead of silently changing observable selection geometry.
- Vec2 triangle centers use component division before conversion to Vec3; Vec3
  centers use reciprocal multiplication. Preserving this distinction prevents
  single-precision center drift from propagating into subgroup centers.
- Rejected position types retain the discovered attribute stride while clearing
  the position pointer. Failed initializations retain different center state for
  points and triangulations.

Detection/BVH trait implementations are split into focused modules. The pinned
ledger fragment covers initialization, patch controls, bounds, element boxes and
centers only. It does **not** claim class-wide completion, matching, detection
maps, owner propagation, connected copies, BVH lifecycle or JSON dumping. Existing
selection regressions remain active; these remaining APIs require their own
native comparison evidence before inventory claims.

Run `cargo test -p rustycad-tkv3d --all-targets --locked` to replay these fixtures
alongside the toolkit suite. OCCT is used only by the local fixture generator
`tools/oracle/martensite_sensitive_primitive_init.cxx`; Rust tests and CI require
no OCCT installation. No renderer code changed in this slice, so existing GPU
functional/capture/performance gates remain applicable without new image baselines.
