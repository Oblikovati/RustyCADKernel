# Dynamic-array emplacement allocation order

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`NCollection_DynamicArray::EmplaceAppend` expands its element block before
invoking the element constructor. The regression supplies an incremental
allocator to both the array and a constructor that makes a nested allocation.
Both allocations fit in one arena block; the stored element must precede the
nested allocation. The old Rust implementation failed this comparison because
it invoked the closure before allocating array storage.

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_dynamic_emplace_order.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -o /tmp/dynamic-emplace-oracle
LD_LIBRARY_PATH=target/occt-modeling-full/lin64/gcc/libi \
  /tmp/dynamic-emplace-oracle > /tmp/dynamic-emplace.actual
diff -u tools/oracle/tkernel_dynamic_emplace_order.expected /tmp/dynamic-emplace.actual
cargo test -p rustycad-tkernel --test dynamic_array_emplace_allocation_order \
  --test ncollection_dynamic_array --locked
```

`porting/audits/dynamic-array-emplace-allocation-order.json` binds the sources,
native executable/library, initial failure, focused tests, full package run and
strict Clippy. This adds a supplemental regression, not an original declaration.

Source inspection also found a separate gap in both `EmplaceValue` overloads:
Rust constructs the requested value before default-constructing gaps; OCCT does
those default constructions first. Their ledger rows are now partial. Throwing
and replacement-constructor semantics remain outside this slice.
