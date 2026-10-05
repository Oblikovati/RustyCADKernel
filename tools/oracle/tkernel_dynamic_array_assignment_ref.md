# DynamicArray assignment and copy-failure reference

The probe checks native `Assign` and both `operator=` overloads return the target reference,
copy-assignment allocator policy, move source state, and copy exceptions at the first and later
block boundaries. `copyDate()` publishes `myUsedSize` after each entire block, so a caught copy
exception in a block leaves that block outside the logical size. The Rust implementation stages
each block before publishing it and drops partially cloned scratch values safely; it matches the
observable size and completed prefix without reproducing OCCT's leaked partially constructed
allocation. Rust panic payloads and C++ exception types are not claimed equivalent.

Build and run against the pinned modeling-full TKernel library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_dynamic_array_assignment_ref.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_dynamic_array_assignment_ref
/tmp/tkernel_dynamic_array_assignment_ref
```

Expected output is committed beside the probe. The corresponding Rust test checks returned
receiver identity, allocator selection, moved-from state, and caught clone panics at both block
boundaries.
