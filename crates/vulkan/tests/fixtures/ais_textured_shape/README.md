# Martensite AIS_TexturedShape native rendering assets

The local oracle uses actual pinned OCCT `AIS_TexturedShape`, real bounded BRep
faces, retained pixmaps, and `PrsMgr_PresentationManager::Display`/`Update`.
It captures 24 class variants on AMD OpenGL and software OpenGL separately.
All 96 generation commands passed; all 48 RGBA/depth repeat pairs were byte
identical. `native-provenance.json` identifies the generator, binary, libraries,
profiles and commands; `native-asset-manifest.jsonl` identifies each buffer.
Rust tests have no OCCT dependency.

`face-boundaries-meshed` uses real meshing and matching boundary polygon tables;
the original `face-boundaries` case has no boundary polygons and therefore
cannot prove rendering of the native line groups. Native aspect replacement
also affects those line groups. Triangle edges and textured boundary lines
remain explicit renderer prerequisites until their checked implementation lands.

The ordinary native replay checks four proved mapping variants on either
device with exact RGBA and the existing `1e-6` depth gate. The complete matrix
remains a separate strict diagnostic; it retains unsupported paths and native
software filtering/line raster differences without changing comparison gates:

```sh
cargo test -p martensite-vulkan --test toolkit textured_full_native_matrix -- --ignored --nocapture
```

The original AMD matrix matched all 22 supported color captures exactly;
opaque depth differences were at most `1.0430813e-7`. Software comparisons
retain one-byte texture filtering differences and larger line/bounds raster
differences. Neither these assets nor the selected replay tests claim complete
class, image codec, inherited selection, or HLR parity. Performance results
require a separate coordinated optimized run; no baseline is accepted here.
