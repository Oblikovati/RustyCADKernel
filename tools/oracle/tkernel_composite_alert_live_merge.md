# Composite alert live-merge callback oracle

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Native source: `Message_CompositeAlerts::AddAlert`,
`src/FoundationClasses/TKernel/Message/Message_CompositeAlerts.cxx`.
The source calls `SupportsMerge()` before it constructs the NCollection list
iterator. `Merge()` callbacks then append to the same live list; the iterator
can see those appended entries while continuing forward.

The first case starts with a different alert type. Incoming `SupportsMerge`
appends a same-type alert that declines merging; AddAlert then visits it and
merges the incoming alert into it. The second case appends a same-type target
from the first `Merge` callback; traversal visits the new target and succeeds.
Both calls return `false` because incoming alerts merge instead of appending.
This fixture does not remove the iterator's current list node from a callback;
that mutation is outside the defined native iterator domain.

Pinned source SHA-256:
`5d73be0b216a8d7f5c1c80efd9460cc6d45b15a7078930a369d0eaf4f8b1a3a7`.
Probe SHA-256:
`b424ef5f91ce234cc36da7c192361723fe199fd6516ce6a564e67118e4b1e077`.

Build and replay from the repository root:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_composite_alert_live_merge.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel_composite_alert_live_merge
/tmp/tkernel_composite_alert_live_merge > /tmp/tkernel_composite_alert_live_merge.out
cmp tools/oracle/tkernel_composite_alert_live_merge.expected \
  /tmp/tkernel_composite_alert_live_merge.out
```
