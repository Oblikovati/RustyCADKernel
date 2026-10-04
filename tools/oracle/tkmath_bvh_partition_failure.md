# Binned split failure state

Pinned `BVH_BinnedBuilder::buildNode` marks its node inner before invoking `SplitPrimitives`; this probe makes the first primitive swap raise `Standard_Failure` and prints the swap count plus root leaf flag. The observed `1 0` confirms that the root stays inner after the failed split. Rust now marks the shared tree node inner at the same point before partitioning, and `binned_split_failure_keeps_native_inner_parent_state` checks that state. Generic custom `BvhQueueBuilder` builds remain serialized and retain their separate failure behavior.

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
