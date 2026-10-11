# Martensite AIS point-cloud evidence

The implemented AIS_PointCloud and AIS_PointCloudOwner contracts pass 77 exact
native comparisons, 12 direct regressions, 11 shared presentation tests and all
43 TKV3d library tests. All-target TKV3d/Vulkan Clippy passes with warnings denied;
56 renderer harness Python tests pass.

The source hashes in both renderer receipts match the final feature sources.
Each profile ran 66 strict native-OpenGL/Vulkan capture comparisons: lavapipe
32 passed / 34 failed; native AMD 12 passed / 54 failed. All 132 geometry/highlight
role assertions per profile passed. Runtime checks report zero Vulkan validation
errors. Raster placement, bounding-box/highlight and depth/color discrepancies
remain failures; no threshold was relaxed. Normal-bearing fixtures use Unlit.
Lit point normals and transparent point policies remain renderer prerequisites.

Both profiles ran 30 release frame workloads, 20 warmup frames and 100 samples.
The independent repeat passed all lavapipe budgets, but native text-glyph median
and p95 exceeded their 1.25/1.5 ratios. That failure remains recorded; this is not
an accepted overall performance baseline. Point-cloud update median/p95 were
1.608/1.732 ms on lavapipe and 1.380/1.517 ms on AMD. These include 1,024-point
buffer rebuilding, selection recomputation, production redraw and readback.

`summary.json` records counts and artifact hashes. Compressed receipts retain
all raw outcomes, source hashes, device/driver identity and timing samples.
`captures.tar.gz` contains generated captures only, not a checkout or build cache.
Paths in receipts identify the generating checkout, which is deleted after merge.

Reproduce CPU checks: `cargo test -p rustycad-tkv3d`.
The numeric oracle corpus is integrated into `tools/verify_toolkit.py TKV3d`.
Reproduce rendering (requires the pinned native OCCT build):

```sh
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full --case-family point-cloud \
  --profiles lavapipe host --output target/point-cloud-renderer.json \
  --captures target/point-cloud-captures
```

This capture command intentionally exits unsuccessfully while strict differences
remain. Compiler inventory and merge/checkout deletion are recorded separately
in the integration receipt after main integration.
