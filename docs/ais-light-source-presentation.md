# Martensite interactive light presentations

Martensite now supplies a concrete `AisLightSource` backed by a live `Graphic3dCLight` and a native-compatible `AisLightSourceOwner`. Existing property and transformation bindings remain available to application subclasses.

```rust
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dCLight, Graphic3dTypeOfLightSource};
use rustycad_tkv3d::ais::{AisLightSource, AisLightSourceMembers};

let light = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
let presentation = AisLightSource::new(&light)?;
presentation.set_size(50.0);
```

The constructor creates the native point, disabled marker, arrow and shadow, translucent unlit shading, text, linked dynamic highlight drawer, transform persistence, and display modes. Computation submits actual indexed spheres/cones, segments, markers, and UTF-8 text for all four light kinds. Display mode 1 follows the native highlight geometry; unsupported modes preserve the native early return. Directional arrow counts retain native 1/3/5/9 buckets.

Applications implementing `AisLightSourceSource` can call `ais_light_source_initialize` to initialize native aspects, then override the four drawing hooks and the two transform refresh hooks. Mode 0 dispatches those virtual hooks; mode 1 avoids light-state refresh. The constructor deliberately uses the base persistence implementation, matching C++ constructor dispatch. Aspect refresh remains an explicit subclass obligation.

Selection creates a directional sensitive sphere and origin sensitive point with native owner priority and transform-persistence sensitivity. Owner clicks switch only on the exact unmodified left button, independent of double-click status. The weak selectable observation does not extend object lifetime. Native bool adapters preserve typed `Failure` unwind; fallible operations are available for callers that prefer a `Result`.

Directional dragging stores the initial local frame, resets and picks the real sphere at both pixel positions, rotates the light using the native quaternion construction, and refreshes owner highlight transforms. Native Start/Confirmed/Update/Abort return true even when the draggable flag is false; Stop clears the retained highlight and returns false. Abort preserves the current frame. Other light kinds refuse every drag action. Directional owner highlight draws the detected point and two rotated sphere arcs, retaining native aspect callback order, explicit/default/immediate layer selection, and undetected early return.

The classes remain partial in the ledger. This batch proves construction and the listed operations, not every protected field, RTTI declaration, inherited operation, arbitrary marker image, or malformed tessellation combination. Native null-handle preconditions are retained. It does not change the Vulkan renderer implementation, add Windows/macOS validation, or establish legacy OpenGL pixel parity.

## Local verification

`cargo test -p rustycad-tkv3d --all-targets --locked` replays committed Rust-only fixtures: 89 presentation, 48 selection, 128 owner click, 24 directional highlight, and 24 real context/view dragging snapshots. Native OCCT is only needed to regenerate these assets locally. The generators in `tools/oracle/martensite_light_*.cxx` reference OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`; each asset reproduced byte-for-byte across two runs. Buffer fixtures compare every position and index, constructor/group aspects, UTF-8 bytes, transform coefficients, and live light directions. Additional Rust regressions cover weak owner expiry, typed error unwind, virtual transform hook dispatch, and clearing the retained local datum before a failed native direction conversion.

`cargo run -p martensite-vulkan --release --example ais_light_source_capture -- 0 /tmp/light-ambient` captures RGBA/depth through the real AIS context and renderer, then records 100 completed frame redraw times after 20 warmups. Kinds 0/1/2/3 select ambient/directional/positional/spot. `MARTENSITE_VULKAN_DEVICE` selects the enumerated host device; `VK_ICD_FILENAMES` can isolate lavapipe. The geometry probes disable optional light names because the current renderer rejects the native Shadow text display style; native name bytes/aspects are covered by the CPU oracle replay. Timing is a local small-scene diagnostic, not an OCCT performance comparison or regression threshold. Evidence is under `porting/runtime-evidence/light-presentation-2026-10-11/`.
