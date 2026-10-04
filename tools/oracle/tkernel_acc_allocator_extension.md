# AccAllocator protected extension oracle

`AccProbe` in `tkernel_ncollection_allocators_probe.cxx` derives from the pinned
`NCollection_AccAllocator` and calls protected `findBlock` and `allocateNewBlock`.
It also allocates through the returned `Block`, then frees that allocation through
`NCollection_AccAllocator::Free`. The output avoids process-specific addresses and
checks block state, lookup keys, and chain removal.

Reproduce against the pinned build:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-tkernel-full/include/opencascade \
  tools/oracle/tkernel_probe.cxx \
  target/occt-tkernel-full/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi \
  -o /tmp/tkernel-probe
printf 'ncoll_allocators_acc_extension\n' | /tmp/tkernel-probe
```

Expected output is in `tkernel_acc_allocator_extension.expected`. The Rust twin is
`ncoll_allocators_acc_extension` in `crates/tkernel/examples/oracle_probe/ncollection/allocators.rs`.
The extension API returns block snapshots and allocator-owned pointers; it does not expose
mutable native descriptor pointers or the protected allocator fields.
