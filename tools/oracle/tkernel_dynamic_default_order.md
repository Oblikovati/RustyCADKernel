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

Both `EmplaceValue` mappings remain partial. Native reserves all requested
storage before constructing any gaps and publishes the new logical length only
after the requested constructor succeeds. Rust allocates progressively and
increments length after each completed default. Thus later constructor failure,
allocation side effects, replacement and destructor behavior are not all proven
equivalent. This regression adds no original legacy declaration.
