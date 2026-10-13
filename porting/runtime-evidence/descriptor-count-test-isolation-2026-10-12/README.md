# Martensite descriptor count test isolation

The two existing RTTI ownership tests share the actual process-global
StandardPersistent descriptor. Each captures a baseline, adds one real owner,
and checks baseline+1/restoration. Concurrent execution correctly changes that
shared count between snapshots and produces false failures. Production atomic
counter behavior is unchanged.

The original full gate failed both owner tests. Default-parallel focused repeat
failed at run52; removing all three borrow-only tests still failed at run5.
Running each owner test independently passed100/100. This minimizes the cause to
the two simultaneous ownership assertions; lazy-registration prewarm and changed
chain code are not load-bearing. Exact red logs and binary hashes are retained.

A named test-only DescriptorCountScope accepts its mutex through a constructor.
Only the two shared-count assertion windows acquire it. All other same-binary
RTTI ownership checks use distinct private descriptors; StandardPersistent/Type
borrow-only tests and storage object handles do not own these descriptor targets.
A concurrency regression uses actual runtime descriptors and synchronized
worker signals to verify that the first owner's observation window stays held
and both explicit owners see exactly baseline+1. There are no sleeps or reduced
test threads. The existing exact counter assertions remain unchanged.

Final default-parallel Kernel all-target execution passes5,016 tests/29groups;
Clippy-Dwarnings, cargo fmt, explicit included-file rustfmt and diff checks pass.
A frozen final binary passes1,000 focused groups/6,000 tests with equal binary
bookends. The new probe's initial literal count1 was corrected to baseline+1
because its actual canonical class cache already owns the descriptor; that
fixture-expectation failure is preserved separately from the original race.

Diagnosis followed the diagnosing-bugs skill: red-capable repeated test loop,
minimized two-owner trigger, isolated controls, named scoped fix and original
loop replay. No temporary production instrumentation, native harness dependency,
API mapping or class-completion credit is added. The unchanged counter code and
realistic interference controls prevented an unnecessary production rewrite.
