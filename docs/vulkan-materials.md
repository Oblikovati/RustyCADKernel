# Martensite shader material records

Martensite now provides the material coefficient records required by the lighting
path. This ports `OpenGl_MaterialCommon`, `OpenGl_MaterialPBR` and
`OpenGl_Material` from OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871` (`OpenGl_Material.hxx` and the material
initializers in `OpenGl_Workspace.cxx`) into the Vulkan crate. It does **not** yet
enable lit draw submissions: normals, light snapshots, GPU shading, lit capping,
native rendering comparisons and lighting performance workloads remain open.

## Equivalent interfaces

| OCCT interface | Martensite equivalent |
| --- | --- |
| `OpenGl_MaterialCommon` fields, constructor | `RenderMaterialCommon` fields, `Default` |
| `Shine`, `ChangeShine`, `SetColor` | `shine`, `change_shine`, `set_color` |
| `OpenGl_MaterialPBR` fields, constructor | `RenderMaterialPbr` fields, `Default` |
| `Metallic`, `ChangeMetallic`, `Roughness`, `ChangeRoughness`, `SetColor` | Corresponding snake-case methods |
| `OpenGl_Material::Common`, `Pbr` | `RenderMaterial::common`, `pbr` |
| `Init` | `init`, with explicit `RenderMaterialColorSpace` instead of an OpenGL context |
| `SetColor` | `set_color` |
| `IsEqual`, `operator==`, `operator!=` | `is_equal`, `PartialEq` and `Eq` |
| `PackedCommon`, `PackedPbr`, `NbOfVec4Common`, `NbOfVec4Pbr` | Corresponding snake-case methods; packed arrays are returned by value |

The color policy represents the context's **active framebuffer sRGB state**.
Linear records are appropriate for the current Vulkan sRGB attachment. Legacy
non-sRGB targets encode only common RGB coefficients; alpha, shininess and PBR
records are unchanged. This API does not add a non-sRGB framebuffer backend.

Aspect materials multiply ambient, diffuse and emissive RGB by the interior
color, leaving specular RGB unchanged. Physical materials use their own colors.
PBR roughness remains normalized in the packed record. Initialization preserves
unused PBR parameter channels. If front and back borrow the same material,
initialization copies the front record, including its interior color and unused
channels, just as the native pointer-identity branch does.

Highlight color replaces ambient with RGB times 0.25 and diffuse/PBR base RGB
with RGB. It preserves alpha, emission, shininess and other PBR parameters.
Material equality compares every float's bits, preserving native `memcmp`
behavior for signed zeros and NaN payloads. No Rust/C++ memory-layout ABI is
promised.

## Verification

Run Rust tests with:

```sh
cargo test -p martensite-vulkan --all-targets --locked
```

Run the native differential probe against the pinned OCCT build with:

```sh
python3 tools/verify_renderer_material.py --occt-build target/occt-all-full
```

The probe checks 417 records containing 23,352 float bit patterns: defaults and
all 26 material names, both framebuffer color policies, separate and aliased
faces, edited materials, and highlight changes. The native oracle links OCCT
only for tests. It uses an uninitialized context with explicit sRGB capability
flags and no write-control entry point, allowing the actual
`SetFrameBufferSRGB` policy and material initializer to run without a GPU.
This is coefficient evidence, not rendered-pixel evidence.

Nine Rust tests additionally cover mutable coefficients, packed ordering,
reserved channels, transparency preservation and bitwise equality. The Python
comparison rejects missing, duplicate, malformed or renamed cases and catches
one-bit differences. Raw output and its hashes are retained by the command.
The receipt is `porting/runtime-evidence/vulkan-materials-2026-10-10.json`.

The renderer capability remains partial. These replacement records do not
justify crediting the entire OCCT lighting family or changing the in-scope API
denominator. The next step is live normal extraction and light/material
submission, followed by GPU shaders with functional captures and release
performance workloads on lavapipe and the native GPU.
