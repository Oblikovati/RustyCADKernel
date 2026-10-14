# Framebuffer extent validation correction

Source `c4d395041` corrects the example image publication helper after the
[restart capture receipt](../gui-framebuffer-restart-2026-10-14/README.md).
Those GPU images remain evidence for `f4f714f7a`; they are not relabeled as
captures of this correction. No new GPU runtime or timing measurements were made.

Two independent regressions failed before the correction. A capture with both
dimensions `u32::MAX` and empty buffers panicked during byte-count multiplication.
A 1×1 capture with four valid RGBA bytes and no depth values returned an error
that omitted the actual depth length and required depth shape. `red.log.gz`
retains both failures.

The helper now checks both multiplications and returns `InvalidData` before
publication when the byte count cannot fit `usize`. Shape errors include actual
RGBA and depth lengths, the extent, and the expected buffer shape. The named
fake store proves both invalid inputs leave storage untouched. Existing valid
color/depth publication and image ownership assertions still pass unchanged.
The image-file save method was shortened without changing its save outcome.

`receipt.json` records this source revision, its source hash, commands and log
hashes. All 12 example helper tests and 8 release-plan tests pass; the affected
examples build, Clippy passes with warnings denied, and the changed source passes
rustfmt and diff checks. The retained historical BadWindow remains unresolved;
fresh performance acceptance still requires a quiet host.
