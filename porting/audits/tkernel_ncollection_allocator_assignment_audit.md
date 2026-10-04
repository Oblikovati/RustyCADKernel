# TKernel collection allocator and assignment audit

This audit checked the allocator and assignment behavior of `NCollection_DynamicArray`,
`NCollection_List`, `NCollection_Sequence`, `NCollection_Array1`, `NCollection_Array2`,
`NCollection_Buffer`, and `NCollection_OccAllocator`. It found no confirmed implementation gap
in these paths, so it makes no API or ledger claims.

## Source and implementation identity

- OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
- Rust source revision: `1519589500049e0fb042abdfa8960d627020c600`.
- Pinned native test executable SHA-256:
  `1abf9129562e16490ab9098162a9cf2c1aa40d72000328fb5bb9e6cf71cf3693`.

| File | SHA-256 |
| --- | --- |
| OCCT `NCollection_DynamicArray.hxx` | `a3ed414887819b3b077ced7c0067e4cdeb02e260fa16f6a1b4f4a4275937889f` |
| OCCT `NCollection_List.hxx` | `8c4bef79ead7da3d5c5f4667f905aebb7262628d025530766c9c0e4cdff41d01` |
| OCCT `NCollection_Sequence.hxx` | `a1172b06eec049b5016f2f478a354ce837811a78c0fdac5a76d99c1608509beb` |
| OCCT `NCollection_Array1.hxx` | `5757b7e442ae800da44a8584b9eda60a16624bbbf35efa82b766439c87bed3b6` |
| OCCT `NCollection_Array2.hxx` | `452d5656962a8e521578a04ce16dbffcee4a279fe4f01570d9796d145d32c75a` |
| OCCT `NCollection_Buffer.hxx` | `e8d90ce2745f5ef78f2bfbf502cb278b56e825b485c64c80512a425907080da9` |
| OCCT `NCollection_OccAllocator.hxx` | `b4c9e73b300d8174145e44932c78d50aa0841fc263751b64ab63e3450808a241` |
| Rust `dynamic_array.rs` | `722ebff54e090e5a8554b704d89531c2c02e42c08a471ccf703836a21b516324` |
| Rust `list.rs` | `245661ec3159866bbb32c64bf546f1d6ade153a6485ee277b4db0d4c2753f018` |
| Rust `sequence.rs` | `a45f112ec65767745d2bfd25d3c635a8f759b81404fd45766593cc06a19e42a8` |
| Rust `array1.rs` | `fc380173bf4fe7b4b57bbf9293966f120f23d3b455f855f1d75be52938a05683` |
| Rust `array2.rs` | `6994e95800615f1bfb050e917cce630a1b07038af1c6e230c7952c707db98574` |
| Rust `buffer.rs` | `66e0fcb5df398636995893826def6b1b2c5b077f9665ed08edee5f3ef47920f6` |
| Rust `occ_allocator.rs` | `7bacac0d38444ac335ebca74267da98ef009c1b7de690f3a087c41f20d22be40` |

## Behavior comparison

- `DynamicArray::Assign(source, ownAllocator)` retains the destination allocator when
  `ownAllocator` is true and adopts the source allocator when false. Rust `assign` follows this
  rule and creates copied blocks with the selected allocator. `Clear(false)` destroys live
  values while retaining block capacity; `Clear(true)` releases blocks.
- `List::Assign` and `Sequence::Assign` clear and copy values through the destination allocator;
  neither changes allocator identity. Move assignment transfers source nodes and allocator,
  while the moved-from container retains its allocator and becomes empty.
- `Array1/Array2::Assign` copies bounds/dimensions and values, writes through same-sized borrowed
  storage, and detaches from borrowed storage when dimensions differ. `Move` transfers ownership
  and leaves the source empty.
- `Buffer::SetAllocator` frees storage before changing the allocator. Rust reports allocator
  failure rather than mutating on failure. The null-allocator/nonzero-size native construction
  path dereferences a null handle; Rust's documented safe result does not claim parity for this
  undefined native input.
- `OccAllocator::SetAllocator` replaces the resource for subsequent operations. Rust retains
  each already-created `AllocatedBytes` object's freeing allocator; this is a safe ownership
  detail, not a claim that mutating an allocator between raw allocate/deallocate calls is valid.
- The inspected headers declare `Clear(allocator)` / `SetAllocator`; they do not declare a
  separate collection `ChangeAllocator` method.

## Validation

The pinned native GTest run passed 14 targeted tests:

```text
NCollection_DynamicArrayTest.AssignmentOperator
NCollection_DynamicArrayTest.CustomAllocator
NCollection_ListTest.AssignMethod
NCollection_ListTest.OCC25348_AssignDoesNotChangeAllocator
NCollection_SequenceTest.CopyAndAssignment
NCollection_SequenceTest.AllocatorTest
NCollection_SequenceTest.MoveOperations
NCollection_Array1Test.AssignmentOperatorDifferentSize
NCollection_Array1Test.AssignmentOperatorReusesSameSizeExternalBuffer
NCollection_Array1Test.AssignmentOperatorDetachesDifferentSizeExternalBuffer
NCollection_Array1Test.Move
NCollection_Array2Test.AssignmentOperator
NCollection_Array2Test.MoveAssignment
NCollection_BufferTest.EmptyRange
```

The matching Rust targets passed 188 tests: Array1 (44), Array2 (37), Buffer (1),
DynamicArray (52), List (34), and Sequence (20). Command:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH \
CARGO_TARGET_DIR=/tmp/rustycad-tkernel-change-allocator-audit-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 \
cargo test -p rustycad-tkernel --test ncollection_dynamic_array \
  --test ncollection_sequence --test ncollection_list --test ncollection_array1 \
  --test ncollection_array2 --test ncollection_buffer
```

No implementation was changed by this audit. The safe next action is to select a different
unimplemented TKernel behavior from the current inventory rather than add duplicate allocator
methods or tests for behavior already covered here.
