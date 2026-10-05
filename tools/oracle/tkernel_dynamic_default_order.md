# Dynamic-array default-constructor order

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The supplemental oracle uses a first gap default constructor that throws. The
requested value constructor must never run, and the array must stay empty. The
initial Rust regression failed because its constructing closure ran first.
The port now constructs the gap defaults before invoking that closure.

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_dynamic_default_order.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -o /tmp/dynamic-default-oracle
LD_LIBRARY_PATH=target/occt-modeling-full/lin64/gcc/libi \
  /tmp/dynamic-default-oracle > /tmp/dynamic-default.actual
diff -u tools/oracle/tkernel_dynamic_default_order.expected /tmp/dynamic-default.actual
cargo test -p rustycad-tkernel --test dynamic_array_default_constructor_order \
  --test ncollection_dynamic_array --test dynamic_array_emplace_allocation_order --locked
```

The original two `EmplaceValue` mappings remain partial for replacement
semantics outside the exercised types. A follow-up now reserves all destination
blocks first, stages gap defaults with the prior logical length, and commits the
new length only after the requested constructor returns. On unwind it forgets
the staged values because the pinned native implementation leaves those
placement-constructed objects outside `UsedSize` and does not destroy them.
See `tkernel_dynamic_emplace_value_order.md` for native evidence and remaining
exception-domain limits. No original legacy declaration was added.
