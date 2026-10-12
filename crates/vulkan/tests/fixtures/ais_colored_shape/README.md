# Martensite actual ColoredShape native buffers

These 128×128 RGBA/depth assets come from pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871, actual AIS_ColoredShape objects and PrsMgr display. Two finite MakeFace faces are compounded and genuinely meshed. Eight appearance/display/update workloads run on llvmpipe and AMD; each native capture was repeated twice with identical bytes.

`native-manifest.jsonl` binds both native runs. `functional-comparisons.json` records exact RGBA and the unchanged finite-depth <=1e-6 gate against the actual retained Vulkan class path. Rust-only replay embeds these assets without OCCT. Full command/source/renderer/output evidence is in `porting/runtime-evidence/colored-shape-compute-2026-10-12`.
