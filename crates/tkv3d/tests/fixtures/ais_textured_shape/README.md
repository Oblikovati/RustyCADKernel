# Martensite textured-shape CPU oracle assets

These fixtures call actual OCCT APIs at revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`. They are generated locally and
replayed by Rust tests without linking or invoking OCCT. `cpu-provenance.json`
records generator, executable, library and fixture hashes and the actual local
compilation commands. Each generator was compiled and executed twice; both
stdout and stderr were byte-identical. Named native null/recording drivers
isolate graphics I/O while retaining the native topology, presentation, aspect,
texture and virtual-dispatch implementations.

- `native-state.txt`: inherited initialization, virtual source setters, source
  priority, native predefined-index parsing and stored mapping parameters.
- `native-aspects.txt`: texture/aspect creation and assignment order, all-group
  replacement, failed source behavior, and the actual `ToCString()` file
  constructor boundary. A native length/filler string retains an interior NUL;
  both the public filename pointer and constructed texture path consume its
  prefix. The full stored filename is retained for native description assembly.
- `native-appearance.txt`: color/material changes through real mode-3 manager
  presentations, including stale copied aspects after `UnsetColor()` and the
  retained material ownership flag after `UnsetMaterial()`.
- `native-width.txt`: six line categories, colored/default aspect restoration,
  unlinked defaults, no-op reset, and failure after publishing the own float.
- `native-transparency.txt`: facing-model selection, retained/released shading
  ownership, and rejection before publishing the drawer transparency scalar.
- `native-deviation-clear.txt`: genuine shared mesh cleanup, including the
  native unordered `infinity - infinity` comparison and auto-triangulation guard.
- `native-triangles.txt`: actual BRep face/mesh handles, optional surface/normals,
  reversed orientation and mirrored location, native normal/UV/index arrays.
- `native-tessellate.txt`: actual registered meshing factory execution over real
  boxes, exact absolute/relative deflection storage and skip behavior for complete
  meshes and null shapes. Rust replay uses the actual Martensite mesh factory.
- `native-volumes.txt`: actual located/reversed compound and compsolid traversal,
  one-dimensional filtering, real boxes meshed by the native/Martensite mesh
  implementations, and the distinct solid/shell closure and triangulation rules.
- `native-presentation-append.txt`: the native appearance overrides also iterate
  a live presentation sequence; a virtual group callback appends a retained
  mode-3 presentation and the same appearance operation refreshes it.
- `native-group-append.txt`: live sequence iteration when a virtual group-aspect
  callback appends a new retained group during the same update.
- `native-compute.txt`: protected concrete Compute over a genuine pretriangulated
  BRep face, modes 0/1/2/3/unsupported 4 and all three mapping booleans.
  Actual group/aspect/position/index callbacks are captured; this recording
  driver does not capture UV attributes, so effective mapped UV rendering still
  requires the paired renderer captures. All 40 rows now replay through the
  registered concrete class and the genuine Isolines prerequisite.
- `native-compute-failure.txt`: actual predefined texture construction with an
  invalid native texture directory proves the shading catch retains its emitted
  group, original group aspect, freshly copied aspect and null texture field.
  Rust regressions separately prove typed constructor/bool-adapter failure
  transport is caught while unrelated panic payloads escape; those injected
  bool failures are transport tests, not additional native oracle rows.
- `native-wireframe.txt`: genuine finite plane wireframe presentations across
  U/V/common aspect routing and plane isoline policies. All 16 rows replay
  through the registered default serial WF presentation path; this does not
  claim completion of its broader overload family.
- `native-public-attributes.txt`: actual public `UpdateAttributes` resolves the
  associated context's current mode-3 presentation. Its existing geometry group
  survives while texture and copied aspect identities change and the group
  receives the new aspect. This uses genuine context ownership and presentation
  manager display, without invoking the incomplete inherited selection family.
  `public-attributes-provenance.json` records byte-identical native repeats.
- `native-file-buffer.txt`: 15 actual captured `TextureFile` pointer rows cover
  native same/shorter Copy reuse, full embedded-NUL storage, protected mutation,
  owned truncation, static-empty lifetime and actual Move/Swap transfers.
  Native intrusive counts remain one; invalid released pointers are never read.
  `file-buffer-provenance.json` records repeat and generator/library identities.
- `native-drop-peers.txt`: an actual `Image_PixMap` subclass destructor reads
  retained drawer/aspect/texture peer counts during `AIS_TexturedShape` release.
  The original Rust `2 2 2` failure and native `3 3 3` trace prove that derived
  resources must release before their inherited base and later texture peers.

The wider `AIS_Shape` family remains partial. Inherited selection and HLR are
explicit unimplemented paths. File/predefined image decoding, triangle-edge
rendering, and textured line rendering are separate unproved dependencies.
These CPU assets do not prove rendering parity or complete either parent or
concrete class. Native failed-source diagnostics are preserved verbatim.

Run registered replay tests with:

```sh
cargo test -p rustycad-tkv3d --test toolkit textured_
```
