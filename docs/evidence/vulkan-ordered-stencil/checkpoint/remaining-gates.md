# Unfinished ordered-stencil checkpoint

Application blocker work supersedes this renderer slice. Keep the original
`porting/martensite-ordered-stencil` branch and checkout: this is not a completed
family, an integration-ready handoff, or an API/class promotion.

The actual retained command/event timeline, native opaque/deferred filtering,
cap sentry state and physical clears, per-view feedback, and Vulkan1.3 dynamic
stencil backend are implemented. Focused CPU regressions and all-target Clippy
pass. Captured evidence is from successive WIP binaries, not a final-source
freeze: 78 ordinary native/Rust pairs and 42 correctly scene-seeded pairs match
exact color/context and the unchanged depth gate. Source-seeded value1 visibly
rejects geometry; value0/2 and disabled testing pass. Original destination-seed
failures and the no-stencil-read observer error remain preserved as harness
errors, with the source/trace diagnosis; no production numerical fix or tolerance
change was used to eliminate them.

The following are still required before completing or integrating the full scope:

- Execute the already-authored remaining stencil-mask-bit and real point/line/text
  discriminators; the 87-case catalog is intended scope, not 87 accepted fixtures.
- Query actual stencil attachment formats as well as the already-observed bits,
  framebuffer bindings, stored value, compare/reference/mask and executed state.
- Complete same-view/adjacent-view reuse and success/failure/success submission,
  clear/remove/destruction lifecycle and persistent context regression coverage.
- Bind final producer/source/executable/library identities, regenerate the whole
  final corpus twice per native profile, and commit independent Rust-only assets
  and strict replay; verify the configured Mesa25 cohort independently.
- Finish explicit new-function bounds/refactoring and final leaf formatting;
  run full affected all-feature/all-target device gates, shader checks and bounded
  tool modules, plus isolated extracted-package Rust replay without OCCT.
- Adopt then-current checked main through an owned checkpointed merge, preserving
  related view/dump/clipping/geometry seams; rerun relevant combined-source checks.
- Freeze complete inputs and optimized accepted workloads; obtain root's quiet-host
  audit/GO before measuring time and peak memory on both devices. No benchmark has
  been run or accepted for this slice.

The existing renderer's named raster/text/depth and undefined-domain diagnostics
remain unchanged. Windows/macOS and application acceptance remain separate.
