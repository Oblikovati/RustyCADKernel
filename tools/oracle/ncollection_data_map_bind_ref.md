# Borrowed DataMap Bind

The native `Bind(const TheKeyType&, const TheItemType&)` hashes and searches the original key once. On a miss, the node constructs its value before copying its key. On a hit, it copy-constructs a temporary from the source value, then assigns that temporary into the existing value slot. The bool reports whether insertion occurred.

This Rust adapter hashes and searches the original key once. On a miss it allocates node storage before cloning, then clones the item before the key. On a hit it clones and replaces only the value, preserving the source and slot address. A supplemental allocator probe verifies allocation/copy order and cleanup when a clone panics. User-defined C++ move-assignment/destructor hooks, self-alias calls, and exact combined-node allocation remain outside the generic Rust contract.

```sh
OCCT_BUILD_ROOT=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
c++ -std=c++17 -O2 \
  -I "$OCCT_BUILD_ROOT/include/opencascade" \
  tools/oracle/ncollection_data_map_bind_ref.cxx \
  -L "$OCCT_BUILD_ROOT/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD_ROOT/lin64/gcc/libi" \
  -lTKernel -o /tmp/ncollection_data_map_bind_ref
/tmp/ncollection_data_map_bind_ref | diff -u tools/oracle/ncollection_data_map_bind_ref.expected -
cargo test -p rustycad-tkernel --test ncollection_data_map_bind_ref
```
