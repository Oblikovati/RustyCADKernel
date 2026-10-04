# Standard_Type owning descriptor probe

This supplemental probe distinguishes class-local static descriptor handles from the
global `Standard_Type::Register` registry. The native class macros keep one static
`occ::handle<Standard_Type>` per class; each descriptor also owns a handle to its
parent. The registry itself stores raw pointers and `Register` returns a raw pointer,
so registration alone does not add an owning reference to the new descriptor.

The output records baseline counts, a copied descriptor handle, a concurrent first
registration, and direct raw registration. The concurrent case uses twelve threads
released from one gate and checks stable counts after joining.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Native `libTKernel.so` SHA-256:
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.
Probe source SHA-256: `9fd0b305c282f0ff630913b2f4832e688a7c6d6b9be7e648481b80ae4ef16e40`.
Probe binary SHA-256: `d3b24e0bc904c268a4ef0e61b50224d3a530427562be52660b3f096236b09df8`.

Build and replay:

```sh
g++ -std=c++17 -pthread \
  -I<occt-build>/include/opencascade \
  tools/oracle/tkernel_runtime_type_owners_probe.cxx \
  -L<occt-build>/lin64/gcc/libi -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKernel -o tkernel_runtime_type_owners_probe
./tkernel_runtime_type_owners_probe
```

Expected output is `tkernel_runtime_type_owners.expected`. This proves the Linux
reference-count and registration behavior of these cases only. Rust descriptors are
leaked to retain stable registry addresses, so this does not claim native descriptor
destruction or explicit-registration lifetime parity.
