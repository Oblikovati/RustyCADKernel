# Composite alert live-merge callback oracle

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Native source: `Message_CompositeAlerts::AddAlert`,
`src/FoundationClasses/TKernel/Message/Message_CompositeAlerts.cxx`.
The source calls `SupportsMerge()` before it constructs the NCollection list
iterator. `Merge()` callbacks then append to the same live list; the iterator
can see those appended entries while continuing forward.
When the list is empty, short-circuit evaluation skips `DynamicType()` too.

The first case starts with a different alert type. Incoming `SupportsMerge`
appends a same-type alert that declines merging; AddAlert then visits it and
merges the incoming alert into it. The second case appends a same-type target
from the first `Merge` callback; traversal visits the new target and succeeds.
Both calls return `false` because incoming alerts merge instead of appending.
The fourth case has incoming `DynamicType()` remove the original first target;
OCCT creates its iterator afterward and visits only the next retained alert.
This fixture does not remove the iterator's current list node from a callback;
that mutation is outside the defined native iterator domain.

The empty-list probe overrides `DynamicType()` with a counter and observes zero
calls. Its two nonempty cases observe two and three custom-type calls,
respectively.

Pinned source SHA-256:
`5d73be0b216a8d7f5c1c80efd9460cc6d45b15a7078930a369d0eaf4f8b1a3a7`.
Probe SHA-256:
`e09f16f3eb059329a44871f0e2c6f6d1a8401dde44fa0fc6fb501dea47ff17cc`.
The exact expected output SHA-256 is
`4803e72897a94aaba2f41862e9f4aeb2e17ec0ee837f1b146c8e02c9e6cdc890` for the
four-case output.
The probe was built with `c++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`.

The source-bound test and build receipts, including the pre-fix red replay and
full TKernel test totals, are in
`porting/audits/tkernel-composite-alert-live-merge.json`.

Build and replay from the repository root:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_composite_alert_live_merge.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel_composite_alert_live_merge_followup
/tmp/tkernel_composite_alert_live_merge_followup > /tmp/tkernel_composite_alert_live_merge_followup.out
cmp tools/oracle/tkernel_composite_alert_live_merge.expected \
  /tmp/tkernel_composite_alert_live_merge_followup.out
```
