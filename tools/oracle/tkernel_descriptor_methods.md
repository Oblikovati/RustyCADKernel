# Standard descriptor method replay

The compiler inventory identifies the macro-generated RTTI members for
`Standard_Type` and `Standard_Persistent` as six distinct declarations. The probe
uses `static_assert` to retain their native return types and records names,
parent descriptors, dynamic descriptor selection, and handle copy/drop counts.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Pinned `libTKernel.so` SHA-256:
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.

```sh
c++ -std=c++17 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_descriptor_methods_probe.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel_descriptor_methods_probe
/tmp/tkernel_descriptor_methods_probe > /tmp/tkernel_descriptor_methods_probe.out
cmp tools/oracle/tkernel_descriptor_methods.expected /tmp/tkernel_descriptor_methods_probe.out
```

Native accessors return `const occ::handle<Standard_Type>&`, a borrowed reference
to the class-held owner. Rust's `RuntimeTyped::static_type` and
`Transient::dynamic_type` borrow the static descriptor directly; the corresponding
`*_handle` methods make an explicit owning copy. Rust does not expose the borrowed
C++ handle wrapper type. Raw registry entries remain leaked, so explicit
`Register`/destructor lifetime parity is still partial.
