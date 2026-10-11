# Martensite platform font evidence

The old Windows library adapter always returned missing, which the strict Windows
font probe reproduces before the fix. The repaired adapter loads FreeType, resolves
its exports and rasterizes the embedded WOFF. Windows registry discovery finds
usable system fonts. Windows GNU binaries under Wine pass 82 library tests, two
strict runtime font checks and all four retained group-flipping tests, including
CI's formerly failing `flip-text`. Native MSVC CI remains pending integration.

The Linux TKService suite passes 237 tests and 3,142 exact native numeric
comparisons. Host and Windows all-target Clippy pass; the macOS target compiles.
A native macOS directory scan still requires its actual runner.

The paired OpenGL/Vulkan group-flipping matrix completes on both devices with
source hashes matching `c28965c2c`. Lavapipe passes 50/54 strict captures; native
AMD passes 51/54. Lavapipe retains four 14-pixel text differences, while AMD
retains three capping-depth differences. Neither tolerance is changed. Both
profiles pass 20 Vulkan runtime checks and 15 lighting scenes / 61,440 analytic
pixels, with zero validation errors.

Both profiles collect 32 release workloads with 20 warmup and 100 sampled frames.
An independent same-device/driver repeat passes all native budgets. Lavapipe's
window-resize/presentation median exceeds its 1.25 ratio budget. These actual
failures remain in the receipts; no successful renderer baseline is promoted.

`captures.tar.gz` contains only generated RGBA/depth/images/diffs. It contains no
checkout, Wine prefix, toolchain, native source or build cache. Reproduce with:

```sh
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full \
  --output target/platform-font-gpu/receipt.json \
  --captures target/platform-font-gpu/captures --case-family group-flipping \
  --profiles lavapipe host --samples 100 --warmup 20
```

The harness intentionally exits with failure while strict capture gates remain.
The change repairs platform behavior of existing APIs; it does not add API claims
or complete the platform configuration matrix.
