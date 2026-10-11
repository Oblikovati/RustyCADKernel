# Local Martensite forced line alpha verification

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Linux host only;
no pushes, Windows/macOS runs or CI native-oracle dependency.

- Rust all-target tests: 247 passed (73 library, 148 toolkit, 26 example tests).
- Clippy with warnings denied, formatting and canonical suite discovery passed.
- 22 Vulkan Python modules passed in separate bounded processes; 13 suite-generator
  regressions passed. The added native-buffer hash and family-routing checks pass.
- Both devices pass analytic alpha/depth checks, all 11 native golden-buffer
  replays and Vulkan validation. All 44 committed native buffers are byte-identical
  on a second native matrix execution.
- Strict matrix: AMD 66/66, lavapipe 50/66. Software endpoint coverage failures
  remain visible; comparison thresholds are unchanged.
- Two release runs retain 100 samples and 20 warmup frames for every one of 36
  workloads. Final alpha median/p95: lavapipe 0.3711/0.7916 ms, AMD 0.2642/0.3610 ms.

[receipt.json.gz](receipt.json.gz) retains the strict matrix, source fingerprints,
shader compiler validation, both performance distributions and device identities.
[final-validation.json.gz](final-validation.json.gz) records the final compiled
source and post-refactor functional/golden replay. Raw RGBA/depth buffers are
stored under `captures/{lavapipe,host}`; selected color/depth PNGs provide view
captures for forced blending, depth occlusion and clipping.

[Performance audit](performance-baseline-audit.json) retains the historical
1.25 median / 1.5 p95 limits. The final software run misses the surface-bound
median by 0.030 ms and window-present p95 by 0.308 ms; the initial run passed all
ten historical software workloads. Native historical full-probe identity differs,
so no native historical regression pass is claimed. Alpha has no accepted
performance baseline. These conditions and software strict capture failures keep
overall renderer acceptance incomplete.

This capability slice adds no inventory API claims. Ordinary transparency,
custom shaders, textures and full gradient/depth line parity are not declared
complete. The coordinating agent integrates the committed branch into local main
and deletes its completed checkout/build artifacts afterward.

The completed branch is integrated and its checkout/branch deleted; see
[integration.json](integration.json). Verification from combined local `main`
passed both-device functional, analytic, lighting and native-buffer checks.
[Integrated receipt](integrated-main-cb6d08a35.json.gz) binds the complete source
at measurement revision `9ae6678687f4068af2b90feb149fa48acce50eae` to two further
36-workload runs (100 samples, 20 warmup). Alpha median/p95 is 0.3653/0.4367 ms
on lavapipe and 0.2602/0.3710 ms on AMD. All ten historical software budgets pass
in this run. Native identity mismatch, strict software endpoint differences and
unaccepted alpha baselines remain open; shared-host compilation is recorded.
