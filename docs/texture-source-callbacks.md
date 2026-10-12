# Martensite texture-source callbacks

The native `AIS_TexturedShape` source setters can release the last pixmap owner.
Its destructor can immediately inspect `TexturePixMap()`. OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871` keeps the old pointer in the actual
handle slot until `EndScope()` returns, and increments the incoming target only
then. Holding a Rust field write guard across this destructor deadlocked a real
pixmap subclass: the regression process timed out with exit 124.

`IntrusiveReferenceSlot` separates the authoritative pointer member from its
Rust ownership lease. Assignment captures the incoming non-owning storage
observer, retires the old ownership outside every field guard, then acquires
incoming ownership and publishes the new pointer. A borrowed
`AisTexturedShapePixMapRef` observes this actual slot, including its address,
nullness and retiring pointer. `set_texture_pix_map(&object.texture_pix_map())`
is a same-target no-op; it acquires no extra pixmap ownership. An explicit
`clone_handle()` remains an owning convenience rather than the native getter.

Twelve committed native records prove last-owner and externally retained-owner
release, filename/predefined-value ordering, same-slot assignment and mutation
of a different incoming source slot from the retiring destructor. Two native
runs match byte for byte; the Rust-only replay consumes the committed fixture.
The destination retains the originally captured incoming pointer while the
caller's live field alias follows its own replacement.

Native destructor-time pointee access is now proved with 24 additional records:
the same count-zero pixmap still has its genuine 3×2 dimensions and first byte
73. A same-slot nested setter is a no-op. Different nested setters can decrement
the retiring object's count to -1, change the actual slot and filename, and leave
a nested pixmap ownership reference unreleased when the outer assignment
subsequently clears that slot. These defined observations are preserved rather
than excluded merely because the counter is nonpositive.

The safe Rust adapter requires the destructor's genuine `&dyn Transient`
reborrow. `alias.with_retiring_pointee(self, |pixels| pixels.width())` projects
that actual embedded pixmap through RTTI; it acquires no owning handle.
`ais_textured_shape_with_retirement(&owner, self, |scope| ...)` lends the actual
source and pointer slot for nested base-body setters. Compile-fail tests prove
that neither the scope nor references borrowed from the pointee can escape the
callback. Field guards are released before callbacks and target destruction.

Preserving native overwritten ownership requires intentionally retaining that
lost reference. Every `EndScope` boundary forgets only an ownership lease
published by a nested assignment and subsequently displaced; ordinary assignment
still releases its previous target normally. This reproduces the native leak
and count2 result instead of silently decrementing it to count1. It adds no
unsafe code and does not postpone the original destructor. A separate two-level
destructor fixture proves that the inner `EndScope` also preserves a callback-
published third target at count1 without running its destructor; the prior Rust
implementation deadlocked while incorrectly destroying it under the field guard.

The getter, both setters and protected pixmap slot remain partial. Ordinary
unscoped Arc-backed projection of an object already in Rust Drop still refuses
access; unrestricted virtual override routing through the scoped mutation
adapter has not been proved. The original full API scope remains open. No
whole-class, codec, renderer parity or performance-baseline credit is added.
Historical strict raster and color diagnostics remain unchanged.

Run the CPU behavior regressions with `cargo test -p rustycad-tkernel -p
rustycad-tkservice -p rustycad-tkv3d --all-targets`. Native source, repeat outputs,
the initial deadlock log and provenance are retained in
[the callback receipt](../porting/runtime-evidence/texture-source-callbacks-2026-10-12/README.md).

The [retirement projection receipt](../porting/runtime-evidence/texture-retirement-projection-2026-10-12/README.md)
contains the genuine native destructor and nested-source corpus, Rust replay,
lifetime-escape rejection tests and exact scope limitations.
