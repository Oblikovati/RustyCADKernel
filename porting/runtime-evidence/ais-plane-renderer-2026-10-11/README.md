# Martensite AIS plane renderer evidence

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`summary.json` records strict capture counts, every remaining failure, layer-role
assertions, performance comparisons and artifact hashes. `renderer-final.json.gz`
contains the complete CPU/GPU matrix and transitive source hashes.
`performance-repeat.json.gz` repeats the 29 release workloads against the same
source hashes, device and driver. Functional and performance gates remain open;
these records do not establish an accepted overall baseline.

`captures.tar.gz` contains only generated image/depth captures and difference
images, rooted at `captures/<profile>/`. It contains no worktree or build cache.

The native allocator probe and red/green regression logs isolate the viewer-layer
`[1, INT_MAX]` intermediate overflow. Allocation/recycling matches native output
`2147483647 1 2 1 3`. Clippy and toolkit/tool tests have separate logs.

Reproduce CPU validation with `cargo test -p martensite-vulkan --all-targets`.
Reproduce the paired GPU matrix with:

```sh
python3 tools/verify_vulkan.py --source ../OCCT --build target/occt-all-full \
  --output target/ais-plane-renderer.json --captures target/ais-plane-captures \
  --case-family ais-plane --profiles lavapipe host
```
