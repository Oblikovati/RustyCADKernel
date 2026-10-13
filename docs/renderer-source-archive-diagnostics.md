# Martensite source and archive renderer diagnostics

Run `tools/diagnose_source_archives.py --root kernel --output diagnostic-output
--revision 4c884b0e01435243f4c23ee7eda93aaff9029641` from a separate tooling
checkout. The workflow checks out this immutable v0.2.4 source and uses the
release package job's headless prerequisites and rendering environment. Only
`vulkan-tools` is added for observation; no display or extra fonts are injected.

The tool uses the canonical 31-package inventory, Cargo packaging, tracked test
support and `stage_workspace`. It builds independent source and extracted-stage
executables. Each cohort first runs its full default-parallel toolkit, then the
three original strict texture/triangle/XR filters and their raw capture targets.
Red test and capture exits are retained rather than promoted to release success.

Evidence includes actual Cargo artifact paths and hashes, source/SPIR-V/fixture/
manifest bookends, default/all feature graphs and hashes, CPU/package/driver
metadata, whitelisted environment and observed child-process library hashes.
Missing mapped libraries remain explicit. Source/stage manifests are compared
without concealing Cargo normalization differences. The workflow always uploads
receipts and raw buffers, excluding executables and build caches. It runs no
native OCCT harness and changes no renderer, golden asset or tolerance.
