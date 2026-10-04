# Binned split failure state

Pinned `BVH_BinnedBuilder::buildNode` marks its node inner before invoking `SplitPrimitives`; this probe makes the first primitive swap raise `Standard_Failure` and prints the swap count plus root leaf flag. The observed `1 0` confirms that the root stays inner after the failed split. The opt-in Rust shared-snapshot builder returns the error before publishing children, so its root remains a leaf. This defined error-state difference is covered by `parallel_split_failure_documents_partial_tree_state_difference` and prevents a full `Build` parity claim for that path.

Run with the repository's configured OCCT build:

```sh
g++ -std=c++20 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_bvh_partition_failure.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath_bvh_partition_failure
/tmp/tkmath_bvh_partition_failure > /tmp/tkmath_bvh_partition_failure.out
cmp tools/oracle/tkmath_bvh_partition_failure.expected /tmp/tkmath_bvh_partition_failure.out
```
