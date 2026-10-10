# Line diagnostics, 2026-10-10

All performance runs use the existing release `renderer_performance 100 20`
protocol: 100 samples, 20 warmups, validation disabled, device-selected profiles.
Each pair runs three times per profile, reversing order on repetition 1. Every
sample and binary SHA-256 is retained. These exploratory measurements do not
replace the six-workload acceptance baseline.

- `performance-before`: main at 36a2b56df versus the initial line implementation.
- `performance-width`: initial lines versus dynamic line width only for line draws.
  Both retain UNORM attachments and 80-byte vertex/fragment push ranges.
- `performance-push`: width-only fix versus restoring triangle constants to
  64 bytes / vertex-only. Both retain UNORM attachments. Lines keep 80 bytes and
  both shader stages. This isolates push-constant state from the later sRGB fix.

Surface medians on lavapipe were 3.71–3.82 ms on main and 4.87–5.32 ms initially.
Removing line-width commands alone did not help. In the push-range comparison,
width-only medians were 5.24/6.22/5.57 ms; vertex-only triangle ranges measured
3.71/4.70/4.02 ms. Release-package compilation overlapped portions of this protocol;
CPU/IO pressure snapshots accompany each run. Native host tails are especially
noisy. No failed run is discarded, and this is not a claim of a clean timing gate.

`color-regression-red.txt` is the new midtone GPU assertion with UNORM attachments:
observed `[128,64,32]`, expected sRGB `[188,137,99]`. `srgb-extra-captures.json`
retains the targeted varying-color/depth comparison after changing attachment
format. The top-level `vulkan-line-colors-before-2026-10-10.json` retains the
previous comparison. All six host gradient cases become exact; software retains
three cases with one-byte differences in two/four pixels. Sloped-depth host
failures are preserved for further diagnosis.

`midtone-assertion-before.json` preserves the first full run: its standalone
lavapipe midtone assertion expected 188 exactly and observed 187. The corrected
encoding check allows one byte of hardware quantization. Strict native capture
comparison is unchanged. That run also retained a host many-draws p95 failure.

The final full matrix, including sRGB and both pipeline changes, is recorded
separately in `vulkan-lines-worktree-2026-10-10.json` and its integration receipt.

The worktree receipt predates constant reuse. Its established host timing budgets
pass; lavapipe surface median is 4.359 ms against a 4.296 ms limit. This red result
motivated a further change: reuse identical push constants until the pipeline or
bytes change. `performance-reuse.json` compares both variants with sRGB enabled.
All three software surface medians improve (4.192/4.397/4.444 to
3.974/3.980/4.153 ms), as do line medians (3.835/4.029/3.958 to
2.932/3.235/3.061 ms). Host medians are broadly unchanged. Tests cover transform
mutation and pipeline/layout transitions. The main receipt verifies the final
implementation and preserves any remaining budget failures.
