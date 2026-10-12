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

The getter and both setters remain partial. Rust refuses projection of a
count-zero retiring pointee with a typed failure; it preserves pointer identity
and count observations without inventing a live target. A conflicting nested
assignment to a currently retiring destination also fails explicitly; its full
native behavior has not been proved. This bounded fix adds no whole-class,
codec, renderer parity or performance-baseline claim. Historical strict raster
and color diagnostics remain unchanged.

Run the CPU behavior regressions with `cargo test -p rustycad-tkernel -p
rustycad-tkservice -p rustycad-tkv3d --all-targets`. Native source, repeat outputs,
the initial deadlock log and provenance are retained in
[the callback receipt](../porting/runtime-evidence/texture-source-callbacks-2026-10-12/README.md).
