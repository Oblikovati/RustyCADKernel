# Martensite textured shape presentations

`AisTexturedShape` is a concrete retained object using actual BRep topology,
native-compatible inherited shape fields and `PrsMgrPresentationManager`.
It builds real default wireframe, shading, textured shading and bounding-box
presentations. The default serial wireframe policy consumes genuine Isolines;
the explicit parallel parameter remains unported. The wider `AIS_Shape`
family remains partial, including inherited selection and HLR.

```rust
use rustycad_tkv3d::{ais::*, prs_mgr::*};

let object = AisTexturedShape::new(&shape);
object.set_texture_pix_map(&pixels);
object.set_texture_origin(true, 0.25, -0.5);
object.set_texture_repeat(false, 2.0, 3.0);
manager.display(&object.cast(), 3)?;
```

The native repeat flag is stored but does not suppress the repeat values used
by `Compute`, or the fresh texture's default repeat sampler. Mode 3 uses the
enabled origin/scale parameters and the stored repeat values. Native failure
ordering and failure messages are preserved rather than imposing additional
mapping validation. The native source setters remain subclassable through
`AisTexturedShapeSource`; `AisTexturedShapeBase` retains genuine parent state.
The source interface shares the concrete runtime descriptor and its actual
`AIS_Shape` ancestry.

`texture_pix_map()` returns a live reference adapter to the retained handle
field. Reading it after subsequent source assignments observes the actual new
field value. Assigning a pixmap retains its object and row-order flag. Although
the OCCT header says the driver ignores `IsTopDown`, pinned `TextureRoot::GetImage`
and the actual native captures show that rendering honors that flag. The port
preserves the source rows and flag instead of silently rewriting either.

`ais_textured_shape_update_attributes` copies the current shading aspect before
texture construction, creates a fresh texture when mapping is enabled and
updates every live presentation group. Native callbacks can append groups or
presentations during the operation; the same native iteration observes them.
The public `UpdateAttributes` entry resolves the associated context's current
mode presentation. A separate native/Rust replay proves that this entry keeps
existing geometry while replacing texture/aspect ownership and updating the
actual group aspect; direct manager display avoids inherited selection setup.
`UnsetColor` preserves the native distinction from mode-3 aspect refreshing.
`SetMaterial`/`UnsetMaterial` use the actual inherited shape bodies and refresh
mode-3 presentations. The shading catch retains partially emitted groups and
recovers typed native failures crossing void/bool adapters; unrelated panic
payloads escape. Texture construction and availability use an owned injectable
boundary. File/predefined codec behavior is not claimed complete.

The current class remains partial. The renderer dependency still rejects
triangle edges and actual textured boundary lines. `face-boundaries-meshed`
uses real meshing and matching polygon tables to expose that dependency;
an empty boundary group is not rendering evidence. Inherited selection and
HLR return explicit typed unimplemented failures, so callers requiring those
families must wait for the parent port.

The committed CPU and native rendering assets replay without an OCCT Rust
dependency. Native class assets cover 24 variants separately on AMD and software
OpenGL, with 48 byte-identical repeat pairs. Four currently proved native mapping
cases pass the ordinary GPU replay on both Vulkan devices using exact RGBA and
the existing `1e-6` depth gate. The full strict matrix is a separate diagnostic
that retains renderer rejections and software filtering/line differences.

```sh
cargo test -p rustycad-tkv3d --test toolkit textured_
cargo test -p martensite-vulkan --test toolkit ais_textured_shape
cargo test -p martensite-vulkan --test toolkit textured_full_native_matrix -- --ignored --nocapture
```

`ais_textured_shape_capture` exercises the actual retained class on either
Vulkan device. `ais_textured_shape_performance` prepares complete redraw/readback
workloads using the shared 20-warmup/100-sample contract. Final source-frozen
functional captures and optimized timings follow checked renderer integration;
the intermediate receipts do not accept a performance baseline or complete
class parity.
