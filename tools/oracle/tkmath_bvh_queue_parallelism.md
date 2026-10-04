# `BVH_QueueBuilder` worker overlap

The pinned OCCT queue builder executes independent queued node splits on its configured worker threads. This fixture creates a root split into two two-primitive nodes; a two-party barrier in those child callbacks proves that both workers enter `buildNode` before either can leave. The output is maximum simultaneous callbacks and final tree length.

Run against the pinned OCCT build:

```sh
g++ -std=c++20 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_bvh_queue_parallelism.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath_bvh_queue_parallelism
timeout 10s /tmp/tkmath_bvh_queue_parallelism
```

Expected output: `2 7`. In the current Rust `queue_build`, the shared mutex around the whole `build_node` and `add_children` callback serializes these same two child splits. The supplemental Rust test records that current limit; it is not a native parity pass. The general mutable-only Rust builder/set contract remains serial until an opt-in synchronized shared-set API can safely support parallel execution.
