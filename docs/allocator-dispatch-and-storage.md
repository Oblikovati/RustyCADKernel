# Martensite allocator dispatch and backing storage

`AllocatorSource` is the genuine polymorphic interface of the embedded
`AllocatorObject`. Its descriptor aliases `NCollection_BaseAllocator`; concrete
and derived handles preserve the original object, parent and intrusive count.
Register a handled descendant with `BaseAllocator::from_handle(&owner)` before
using its compatibility `resource()` façade. Registration stores a weak alias;
repeating it does not replace the object or create a self-cycle.

The three raw entrypoints invoke the real derived overrides. Base Optimal is
independent of Allocate. Heap, Acc and Aligned Optimal call virtual Allocate;
Inc Allocate calls virtual Optimal. The protected base constructor is proved at count0 on the stack, count1
after handle construction, and count2 with an alias to the same object.
Static CommonBaseAllocator and
GlobalHeapAllocator return references to actual persistent handle slots.
`BaseAllocator::common()` and `global_heap()` acquire explicit owning copies.

Typed storage uses `storage_lease()`, which acquires no intrusive ownership.
Its backing memory and original raw allocation provenance survive independently
of the derived allocator object. The live original source still receives Free;
when a collection changes resource, `original.deallocate_using(pointer, layout,
&current)` restores the original pointer and invokes current virtual Free.
Payload destruction and collection metadata are the caller's responsibility.
The clipping worker owns those Sequence lifecycle policies.

The pinned six-record native Heap probe changes allocator during Allocate,
releases the caller's last ownership while a node remains, and later frees it
through the current compatible allocator. A retained allocator observer failed
this real case: it delayed the destructor. The new weak source plus independent
backing lease matches native retirement at size1/count0, before current Free.
The rejected Rust transcript and byte-repeatable native assets are retained.

Custom memory providers implement the unsafe `AllocatorPhysicalStorage`
contract and override `AllocatorSource::physical_storage()`. This lease owns
actual backing memory, not the derived allocator. The default provider is valid
for allocations from the embedded policy. An injected independent heap test
proves actual object Drop before surviving payload release; it is Rust safety
coverage, not a native custom-pool parity claim.

Typed over-alignment and independent backing lifetime are Rust safety behavior.
Native Inc storage becomes invalid after its allocator destruction; reading it
then is not native oracle success. Owning sequences retain their actual allocator
while nodes are live. Allocator family completion and unrestricted destructor
reentrancy are not claimed from these bounded probes. The four existing
AIS_TexturedShape source retirement bindings remain partial and unchanged.

Temporary dispatch observers remain an open native lifetime obligation. The
retirement fixture releases the last caller ownership after Allocate returns;
it does not prove ownership loss during an active virtual callback. Those
contracts remain in family scope and are not inferred complete from this matrix.
