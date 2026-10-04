# Fresh BaseAllocator and HeapAllocator instances

## Native constructor declarations

| Declaration | Compiler identity | Access | Header SHA-256 |
| --- | --- | --- | --- |
| `NCollection_BaseAllocator()` | `4c2ed1877a2009771090593a9014e53e2a7b87af4f88bf88a523d10ca2c48d64` | protected | `dfe168fc91f72b4e20391d015aca19da751d505f79795e4af3d0174cc71d321e` |
| `NCollection_HeapAllocator()` | `ee8f9d9474d56031e3ca793e667ef38386a50100e45cd6e0c30be363cbc4d1ac` | protected | `5a478d96d94cb53a89b1ea737a2ce1b8a0c392f2c57060efdfeb87e7d6466f5b` |

The compiler inventory resolves each exact selector uniquely. OCCT documents that more than one
base allocator can be created despite recommending `CommonBaseAllocator`; protected constructors
make that possible through derived classes. Rust's `fresh_common` and `fresh_heap` create distinct
policy identities and preserve allocation/free behavior. The wrappers do not provide C++ subclassing
or runtime-type identity.

## Identity and Poly removal evidence

`tkernel_acc_allocator_instances` compares each process singleton with repeat calls, derived
instances with their corresponding singleton, and two distinct derived instances. Expected
output is `1111111` in `tools/oracle/tkernel_acc_allocator_instances.expected`.

Pinned `Poly_CoherentTriPtr::Remove` unlinks a node and calls `Free` on the supplied allocator,
without comparing allocator handles. `tkmath_coherent_instance_removal.cxx` allocates a ring node
from one derived base/heap allocator and removes it through another same-backend instance. The
native free counters report `common 1:0/0:1 heap 1:0/0:1`. Rust accepts those two compatible backend
pairs and releases through the allocation's retained owner. Arena instances remain identity-bound:
tests verify a wrong incremental allocator is rejected, the owner still has a live allocation, and
removal through the owner permits reset.

`Poly_CoherentTriPtr.cxx` SHA-256:
`94c8114cd73f61b4df39ab335ac530b0a4f1bd00390d553c0f772894bd4f7eba`.
