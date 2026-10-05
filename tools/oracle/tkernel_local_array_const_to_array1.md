# LocalArray const `ToArray1` alias behavior

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The `const` overload at `NCollection_LocalArray.hxx:304` returns a mutable
`NCollection_Array1<theItem>` over the same `myPtr`. The standalone probe
writes through that view while the source is const-qualified and observes the
change through the original buffer (`12 77`). Rust `to_array1_view(&self)` is
a safe copy-on-write view; writing it leaves the LocalArray unchanged. The
ledger row is therefore partial. Rust keeps the safe borrowing contract and
does not emulate native const-cast mutation.

Source hashes:

- `NCollection_LocalArray.hxx`: `516ac5faff95f20fa5638c2cf8819c0730f6940289316a6a662fbbb6981f141f`
- `NCollection_LocalArray_Test.cxx`: `d24eb586570d9f970f864f655ec071b07351a66517059b86d0fcaddf1100a864`

Build with the repository's pinned Linux OCCT installation:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/OCCT/src/FoundationClasses/TKernel/NCollection \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_local_array_const_to_array1.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel-local-array-const-to-array1
/tmp/tkernel-local-array-const-to-array1
```

The expected native output is in the adjacent `.expected` file.
