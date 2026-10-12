# Martensite borrowed textured-shape filenames

`object.texture_file()` captures the real NUL-terminated `AsciiString` allocation.
`file.as_ptr()` exposes that captured address; `file.read()?` makes a bounded
observation of its current C prefix. The observer retains neither the object
nor its byte buffer, and holds no lock across a caller callback or source update.

```rust
let file = object.texture_file();
object.set_texture_file_name(&"short.png".into())?;
let observed = file.read()?;
```

Native OCCT `3d097a0` copy assignment keeps the destination allocation for an
equal or shorter nonempty input. Filename assignment now uses the existing
`AsciiString::assign_from` body instead of replacing the field with a clone.
The old observer consequently sees updated bytes at its original address.
Embedded NULs remain in the stored string; observation ends at the first NUL.

The observer follows the allocation through genuine `Move` and `Swap` ownership
transfers, including edits under the new owner. It never substitutes the current
source-field buffer for its captured allocation. Clearing the last owner or
reallocating that buffer invalidates the native pointer; safe observation then
returns `Standard_NoSuchObject`. The native fixture never reads a dangling
pointer. A captured native static-empty terminator remains valid independently
of later source allocations, while an owned buffer truncated to zero retains
its distinct allocation identity.

Private allocation storage has one unique owner and weak observers. Mutations
and copied observations use its own short read/write gate. Ordinary borrowed
byte slices remain tied to the unique owner's Rust lifetime. The small unsafe
boundary is confined to `ascii_string/allocation.rs`; observers only return
copied bytes. Retirement marks a released allocation logically dead even if a
temporary internal storage lease delays physical destruction.

Write guards assert that a Vec operation preserves its initial address and
capacity; growth must first retire the allocation owner. A scoped-thread/barrier
regression checks copied weak observations racing bounded same-size updates,
without sleeps or a claim about native concurrent access. Growth within existing
padding is Rust safety behavior: native `reallocate()` calls
`Standard::Reallocate` even when the padded size is unchanged. Those potentially
invalidated old native C pointers are outside the 15 defined-lifetime fixture
cases and confer no wider foundation parity credit.

The committed native fixture records 15 byte-identical repeated rows, including
the actual intrusive object count of one throughout observation and transfer.
Rust regressions replay these rows and separately check released-owner refusal,
every allocation growth path, clone/move/swap identity and temporary leases.
The earlier field-relative prototype and its moved-buffer limitation are
retained as rejected scope evidence. These helpers grant no unrelated native
`AsciiString` API credit and do not complete the TexturedShape rendering family.
