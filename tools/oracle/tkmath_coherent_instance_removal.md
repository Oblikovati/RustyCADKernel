# Poly coherent removal across equivalent allocator instances

`Poly_CoherentTriPtr::Remove` forwards the supplied allocator's `Free` call; the native
probe allocates each list node from one derived Base/Heap allocator and removes it through
a different instance of the same backend. Counters confirm the freeing allocator receives
the call. Rust retains the allocation's owner for safe deallocation but accepts distinct
common/common and heap/heap policies as compatible. Incremental and accumulating arenas
remain identity-checked.

Reproduce against the pinned native toolkit build:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-tkernel-full/include/opencascade \
  tools/oracle/tkmath_coherent_instance_removal.cxx \
  target/occt-tkernel-full/lin64/gcc/libi/libTKMath.so \
  target/occt-tkernel-full/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi \
  -o /tmp/tkmath-coherent-instance-removal
/tmp/tkmath-coherent-instance-removal
```

Expected output is in `tkmath_coherent_instance_removal.expected`.
