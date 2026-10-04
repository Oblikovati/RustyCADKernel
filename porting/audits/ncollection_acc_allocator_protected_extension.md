# NCollection_AccAllocator protected extension slice

## Native declarations

Pinned `NCollection_AccAllocator.hxx` SHA256:
`5fe1b00032b8b1493a18d1a8c778410f929cb25a010757c15028d02fa490c585`.

| Source declaration | Selector / identity | Access |
| --- | --- | --- |
| `findBlock(void*, Key&) noexcept` | `c:@S@NCollection_AccAllocator@F@findBlock#*v#&$@S@NCollection_AccAllocator@S@Key#` | protected, lines 183–184 |
| `allocateNewBlock(size_t)` | `c:@S@NCollection_AccAllocator@F@allocateNewBlock#l#` | protected, lines 186–187 |
| `Block(void*, size_t, Block*) noexcept` | `c:@S@NCollection_AccAllocator@S@Block@F@Block#*v#l#*$@S@NCollection_AccAllocator@S@Block#` | protected nested type, public constructor, lines 143–171 |

The block constructor is currently unclaimed in the captured crosswalk. Its exact unique Rust target is `rustycad_tkernel::ncollection::AccBlock::with_previous` in `crates/tkernel/src/ncollection/allocator/block.rs` (source SHA256 `45f4c9e2f13a7cac015a7627fc02652ca9918d01bd56707a9378851e5efdb9b6`). It preserves the block’s previous-link value as `Option<usize>` and models addresses as integer identities so the record does not imply ownership or permit dereference. This is a safe representation mapping, not a claim of native pointer identity.

## Rust extension surface

`BaseAllocator::accumulating_extension` provides a lock-scoped `AccAllocatorExtension`. Its lookup returns the native output key, including the previous-key fallback on misses, and an immutable `AccBlockState` snapshot. It can create a block through the shared engine and allocate from that block only when the rounded request fits its free space. A key collision is detected before binding: Rust frees the new payload and returns `Standard_ProgramError` instead of replacing a live map entry. Returned pointers retain payload provenance. Callers must drop the mutex guard before reentering the allocator.

The API deliberately does not expose mutable block pointers, `myBlockSize`, `mypLastBlock`, or the native `NCollection_DataMap`. It therefore does not claim arbitrary C++ subclass, RTTI, or raw mutable-field parity.

## Oracle evidence

`tools/oracle/tkernel_ncollection_allocators_probe.cxx` derives `AccProbe` from the native allocator and directly exercises both protected methods plus `Block::Allocate`. Request `ncoll_allocators_acc_extension` is mirrored in `crates/tkernel/examples/oracle_probe/ncollection/allocators.rs` and its corpus. The pinned native output is in `tools/oracle/tkernel_acc_allocator_extension.expected`; it checks lookup keys, free bytes, allocation counts, offsets, and chain unlink after free.
