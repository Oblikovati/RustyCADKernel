# DynamicArray `EmplaceValue` ordering

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`NCollection_DynamicArray.hxx` first calls `ensureStorageForIndex`, then
constructs each gap, then constructs the requested item. It writes
`myUsedSize = theIndex + 1` only after the requested constructor returns. A
throwing requested constructor therefore leaves the old logical length. The
native destructor only destroys items below that length; already constructed
gap objects remain undestroyed. The Rust stage reserves all destination blocks
before either constructor, commits `used` last, and forgets staged values on
unwind to preserve that observable destructor behavior without unsafe code.

Rust's safe `IndexMut` can initialize slots beyond `used`, unlike native access
to unconstructed storage. A regression covers that state: successful
EmplaceValue overwrites only the requested prefix and preserves later
initialized slots; a constructor panic leaves those slots unchanged. This is a
Rust storage-safety guarantee and is not counted as native parity evidence.

The caught-constructor path intentionally leaks resources held by staged gap
values, as the pinned native implementation does. Re-emplacing into those
indices after a caught panic is outside the verified parity domain: native
placement construction encounters live objects whose constructors were not
paired with destructors. Allocation failure and throwing gap-default
constructors retain native length/order behavior, but no exhaustive C++ object
lifetime guarantee is claimed for arbitrary throwing/destructor types.

The native probe checks a target at the first index of a new block (so its
block must already be allocated before the requested constructor's nested
allocation), a multi-block target where every block must exist before the
first gap constructor's nested allocation, and a requested-constructor
exception after three gap defaults.

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_dynamic_emplace_value_order.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -o /tmp/tkernel-dynamic-emplace-value-order
LD_LIBRARY_PATH=target/occt-modeling-full/lin64/gcc/libi \
  /tmp/tkernel-dynamic-emplace-value-order
```

The recorded output is `tkernel_dynamic_emplace_value_order.expected`.
