# ExtendedString UTF-16 C-string predicates

The one-argument pointer overloads `IsEqual`, `IsDifferent`, `IsLess`, `IsGreater`, `StartsWith`, and `EndsWith` call `extStringLen` and then their pointer-and-length cores. They therefore compare only through the first NUL. A null pointer produces length zero: equality is true only for an empty receiver, inequality is its inverse, ordering compares against empty, and empty prefixes/suffixes match.

The native fixture compares receiver `A` with needle `A\0B`, then receiver `AB` against the same truncated needle. It also verifies the pointer operator aliases, full three-unit explicit-length behavior, `B\0A` suffix matching, and null-pointer behavior on both empty and nonempty receivers.

Build and run against the pinned modeling-full native library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_c_units_comparison.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_c_units_comparison
/tmp/tkernel_extended_string_c_units_comparison
```

Its output is committed in `tkernel_extended_string_c_units_comparison.expected`; the declaration IDs, source hashes, unique Rustdoc paths, and evidence hashes are in `porting/audits/extended-string-c-units-comparison.json`.
