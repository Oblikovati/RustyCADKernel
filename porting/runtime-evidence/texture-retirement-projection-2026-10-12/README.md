# Martensite destructor projection and nested source transport

OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` exposes the real count-zero
Image_PixMap through TexturePixMap during its descendant's destructor. The
24-record native matrix proves pointer identity, genuine 3×2 dimensions and
byte73, same-slot no-op, negative-count nested setters and native lost-reference
counts/filename ordering. Two native runs repeat byte-identically. A separate
10-record genuine two-level destructor matrix proves that every EndScope clears
a callback-published third target without releasing its ownership: it stays at
count1 and its destructor never runs. Both corpora have committed Rust replay.

The initial ordinary Arc observer cannot project an object already in Rust
Drop: the real regression fails with typed refusal. The new safe scope receives
the destructor's genuine &dynTransient reborrow, checks original pointer
identity, and projects the actual embedded pixmap through RTTI. It creates no
Arc observer or owning pixmap snapshot; lifetime compile-fail tests reject both
escaped scope and pointee reference. No new unsafe code or Delete hook is added.

The coordinator's deeper EndScope case was reproduced before its fix: incorrectly
releasing a callback-published third target under a slot guard deadlocked the
actual getter callback (timeout exit124). Shared clear_end_scope_pointer now
preserves native lost ownership at every boundary. It deliberately forgets only
the ownership displaced by an EndScope callback; ordinary replacements still
release the original target outside guards, proved by the retained callback
regressions and the no-nested-assignment rows.

Original four bindings remain partial. Explicit scope transport calls native
base source bodies; unrestricted virtual override routing/unscoped Arc-backed
retiring projection remain open. Their full behavior is retained in scope; no
class completion, native codec, renderer parity or baseline credit is added.
Raw historical strict rendering diagnostics stay intact. This change does not
alter rendering submission policy.

Final checks:8,836 four-toolkit consumer tests, five-crate Clippy -Dwarnings,
global formatter, canonical discovery plus13 discovery regressions pass. Two
lifetime compile-fail tests and three native corpus replay tests pass. The
original12-record ordinary source callback corpus also stays green. Four pinned
original partial bindings preflight against the current two-crate scoped
rustdoc capture; no whole-workspace inventory is claimed.

The pinned validation-layer wrapper runs actual AMD/lavapipe selected4 replay
successfully. Full strict diagnostic remains exit101 with1AMD and20software
cases, matching historical raw color/line-coverage differences. Thresholds,
baselines and accepted workload gates are unchanged.
