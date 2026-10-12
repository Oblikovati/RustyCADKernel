# Martensite texture-source callback receipt

The pinned native `AIS_TexturedShape`/`Image_PixMap` subclasses exercise real
last-owner destructors and actual const handle slots. Twelve final records
repeat byte-identically and are replayed by Rust-only tests. Historical native
outputs retain their original whitespace; final outputs avoid an empty trailing
filename separator without changing observed values.

`rust-getter-before.log.gz` records the genuine pre-fix deadlock (bounded process
exit 124). The fix releases ownership outside all field guards while leaving the
old pointer visible through `EndScope`. It captures incoming pointer storage
without incrementing the native count before the old destructor returns. Live
caller-slot replacement and same-slot no-op behavior are both proved.

The three source/getter APIs remain partial: retiring pointee projection is
explicitly refused rather than fabricated, and conflicting nested assignment
remains unproved. The source-slot observer itself takes no intrusive ownership.
There is no new renderer-submission policy, native codec claim, whole-family
completion or accepted performance baseline. Existing full strict GPU raster
and color diagnostics remain open.

`provenance.json` pins source/header/binary identities. Check logs and actual-device
replays are copied here after their terminal completion. Native oracle generation
is local-only; ordinary behavior tests consume the committed fixture.

Final consumer checks pass 6,079 tests, including four focused foundation
regressions and two callback corpus tests. Four-crate Clippy (`-D warnings`),
global format and canonical discovery pass. Four uniquely pinned partial rows
pass the production declaration matcher against the committed two-crate scoped
rustdoc capture; this is not a whole-workspace inventory.

The actual AMD/lavapipe selected four-case contract passes unchanged. The opt-in
full strict matrix still exits 101: one AMD case and 20 lavapipe cases differ,
matching the historical documented diagnostics. No gate or threshold changes.
There is no new rendering submission policy or new timing-baseline claim.
