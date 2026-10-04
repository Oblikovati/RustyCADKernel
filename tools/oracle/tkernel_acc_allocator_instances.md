# Fresh BaseAllocator and HeapAllocator identity oracle

The native probe derives public test classes from the protected
`NCollection_BaseAllocator` and `NCollection_HeapAllocator` constructors. It compares
those independent instances with each other and with the `CommonBaseAllocator` and
`GlobalHeapAllocator` singletons, then allocates and frees through both derived instances.

Reproduce against the pinned build:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-tkernel-full/include/opencascade \
  tools/oracle/tkernel_probe.cxx \
  target/occt-tkernel-full/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi \
  -o /tmp/tkernel-probe
printf 'ncoll_allocators_instance_identity\n' | /tmp/tkernel-probe
```

Expected output is `1111111`, recorded in `tkernel_acc_allocator_instances.expected`.
The Rust twin proves fresh allocation-policy identity and freeing. It does not claim
native subclassing or dynamic RTTI identity.
