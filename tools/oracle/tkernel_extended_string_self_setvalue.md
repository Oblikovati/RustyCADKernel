# ExtendedString self-buffer SetValue

The pointer `SetValue` overload has a defined in-place self-alias case when the replacement fits inside the existing `Length()`. The native implementation writes from left to right, so overlapping writes may change later reads. The Rust offset adapter reproduces that order without creating an aliased Rust reference.

The probe covers overlap in both directions, a pointer at the terminator, and invalid index validation with an empty source. It produces no output for growing self-alias `SetValue` or self-alias `Insert`: both native paths reallocate before reading the old pointer, so those cases are outside the defined contract. Rust refuses a growing self-alias SetValue with `Standard_NotImplemented`; this is a safe-boundary signal, not a claimed native exception.

Run the original native class suite and the probe against the pinned modeling-full artifacts:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_self_setvalue.cxx \
  target/occt-modeling-full/lin64/gcc/libi/libTKernel.so.8.1.0 \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_self_setvalue
/tmp/tkernel_extended_string_self_setvalue
 target/occt-modeling-full/lin64/gcc/bini/OpenCascadeGTest \
  --gtest_filter='TCollection_ExtendedStringTest.*'
```

The machine-readable audit records source and binary hashes, exact compiler selectors, Rustdoc item identities, and the focused and full TKernel test receipts. The supplemental cases stay outside the source-mapped legacy test target.
