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
  requires the paired renderer captures. This replay awaits concrete-class
  registration with the genuine isoline prerequisite.
- `native-wireframe.txt`: genuine finite plane wireframe presentations across
  U/V/common aspect routing and plane isoline policies. This asset is awaiting
  registration of the full isoline prerequisite; it is not an implementation
  completion claim.

The wider `AIS_Shape` family remains partial. Inherited selection and HLR are
explicit unimplemented paths. File/predefined image decoding, triangle-edge
rendering, and textured line rendering are separate unproved dependencies.
These CPU assets do not prove rendering parity or complete either parent or
concrete class. Native failed-source diagnostics are preserved verbatim.

Run registered replay tests with:

```sh
cargo test -p rustycad-tkv3d --test toolkit textured_
```
