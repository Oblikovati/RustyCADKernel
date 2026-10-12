# Martensite image dump option fields

OCCT `V3d_ImageDumpOptions` is a public value struct. Its eight integer, boolean
and enum fields were incorrectly excluded as private state. Martensite already
exposes the actual mutable counterparts on `V3dImageDumpOptions`:

| OCCT field | Rust field |
|---|---|
| `Width` | `width` |
| `Height` | `height` |
| `BufferType` | `buffer_type` |
| `StereoOptions` | `stereo_options` |
| `TileSize` | `tile_size` |
| `ToAdjustAspect` | `to_adjust_aspect` |
| `TargetZLayerId` | `target_z_layer_id` |
| `IsSingleLayer` | `is_single_layer` |

For example, `options.width = -1` changes the field directly. Validation belongs
to the consuming view operation; this field mapping does not introduce a setter
that rejects values accepted by direct native assignment.

The pinned OCCT 3d097a0 header-only oracle records defaults, twelve mutations
covering all six named buffer types and four stereo variants, both boolean values,
negative dimensions/tile/layer identifiers, and independent copied value fields.
The committed fifteen rows repeat byte-for-byte and replay using Rust alone:
`cargo test -p rustycad-tkv3d --test toolkit image_dump_options_values`.

`LightName` now uses the project-owned lifetime-bound `V3dLightName` pointer
transport; the complete struct is `Copy` with native shallow-pointer semantics.
The original value-field receipt remains historical evidence for the bounded
slice; [the pointer and consumer proof](image-dump-light-name.md) records the
subsequent native alias, encoding, null, lifetime and layout behavior.
