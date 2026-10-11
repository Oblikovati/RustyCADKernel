# Martensite trihedron runtime evidence

The port implements native cached wire/shaded trihedron geometry, part owners,
selection modes, appearance ownership, selected/dynamic highlight hooks and
protected subclass state. The pinned native numeric suite passes 159,688 exact
requests. All 462 TKV3d and 226 Vulkan tests pass; all-target Clippy and 59 Python
renderer-tool tests pass. The compiler capture was taken at `fdc3ee03f`.

`renderer-corrected.json.gz` binds the current implementation hashes to both GPU
profiles. Both pass 20 Vulkan runtime checks, including signed polygon depth bias
and reset, plus 15 lighting scenes / 61,440 analytic color/depth pixels. Validation
reports zero errors. Lavapipe passes 60 of 96 strict captures; native AMD passes 1.
These remain failures, with unchanged color/depth tolerances.

Each profile passes 170 of 192 geometry/selection roles. The remaining shaded-part
fixtures can have no visible selection change, including native scenes; four
native axis cases change while their Vulkan counterparts do not. They remain
explicit failures requiring renderer/fixture investigation. A yellow-label
assertion previously counted ordinary labels as highlighting and incorrectly
rejected gray whole-object selection. The regression now compares each selected
image with its same-device unselected reference and rejects unchanged images.
The original receipt and recomputed roles preserve the correction's provenance.

Each profile records all 32 release workloads with 20 warmup / 100 sampled frames.
The new workload measures shaded trihedron size changes, recomputation, submission
and readback. An independent same-driver repeat passes all host budgets, while
lavapipe's existing remove/display p95 exceeds its 1.5 ratio budget. The new
trihedron workload passes both repeat budgets. No overall baseline is accepted
while functional/strict gates remain open.

`captures.tar.gz` contains only generated RGBA/depth/images/diffs from the original
and corrected runs. It contains no checkout or build cache. Reproduce with:

```sh
python3 tools/verify_vulkan.py --source ../OCCT \
  --build ../RustyCADKernel/target/occt-all-full \
  --output target/trihedron-gpu/receipt.json \
  --captures target/trihedron-gpu/captures --case-family ais-trihedron \
  --profiles lavapipe host --samples 100 --warmup 20
```

The harness intentionally returns failure while these open gates remain.
