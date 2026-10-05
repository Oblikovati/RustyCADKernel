# Standard_GUID assignment reference result

The native `operator=(const Standard_GUID&)` and `operator=(const Standard_UUID&)` return a reference to the receiving GUID. The existing `Assign` methods return `void`; the new Rust `assign_returning` and `assign_uuid_returning` adapters preserve the operator result while delegating field copying to those established methods.

This probe verifies both returned references identify the receiver, then exercises a chained UUID assignment and checks the resulting GUIDs. Build against the pinned modeling-full TKernel library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_standard_guid_assignment_ref.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_standard_guid_assignment_ref
/tmp/tkernel_standard_guid_assignment_ref
```

The expected output is committed beside the probe. The Rust test uses pointer identity on the returned `&mut Guid` and verifies the same chained field updates. This validates the return contract without changing the existing void `Assign` mappings.
