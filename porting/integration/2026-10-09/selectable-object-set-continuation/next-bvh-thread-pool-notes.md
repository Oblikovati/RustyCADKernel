# Next port: SelectMgr_BVHThreadPool

Source: `src/Visualization/TKV3d/SelectMgr/SelectMgr_BVHThreadPool.{hxx,cxx}` at
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Both sensitive and selectable BVH sets
are now implemented; this pool is a remaining dependency for ViewerSelector.

Native behavior observed directly:

- Constructor clamps the worker count to at least 1, creates a one-based Array1,
  initializes wake=false, idle=true and started=false, and captures the current
  floating-point-signal setting for each worker. Threads start lazily.
- AddEntity first calls virtual ToBuildBVH; false means no queue or start side
  effects. True appends the handle without deduplication, sets wake, resets idle,
  and starts every worker if not already started. Null input is native undefined.
- A worker waits for wake and checks stop BEFORE taking a task. Under the queue
  mutex it either resets wake/sets idle for an empty queue, or removes First.
  It then acquires its own BVH mutex and invokes virtual BVH. The queue lock is
  never held during BVH.
- Worker failures are caught and sent to the default messenger: Standard_Failure
  type and message, std::exception dynamic type and message, or unknown-exception
  text. A failure does not terminate processing of later queued entities.
- WaitThreads first waits for the idle event, then constructs a Sentry that locks
  all worker BVH mutexes in array order. Idle alone is not proof that builds ended.
- StopThreads is a no-op before start. Otherwise it sets stop, wakes, waits for
  every worker, and resets stop/started. It does NOT clear the pending queue or
  reinitialize events. Adding a buildable entity after stop restarts workers.
- Destructor calls StopThreads. Do not keep the public pool alive in its own
  worker ownership cycle; retained entities must still survive their queued jobs.
- Public Threads exposes mutable Array1<BVHThread>, and BVHThread inherits
  OSD_Thread. Copy and Assign share the underlying native thread handle/function/
  priority and copy the raw pool pointer and FPE setting, but each wrapper has a
  distinct mutex (Assign leaves the destination mutex alone).
- Sentry owns a pool handle; null is allowed. Constructor locks, destructor
  unlocks, and explicit Lock/Unlock exist. Copy/assignment are deliberately
  declared but undefined. Native std::mutex is nonrecursive. Do not substitute
  the project's recursive StandardMutex.

Existing seams:

- `crates/tkernel/src/osd/parallel/thread.rs`: OsdThread implements native copies,
  detach, wait, timeout and priorities. ThreadFunction is currently fn(usize)->usize;
  a safe owned callback/context seam may be needed to bridge the pool without raw
  pointer reconstruction or a hidden global registry. Preserve inherited public
  thread operations, function replacement and worker-copy semantics.
- `standard/condition.rs`: StandardCondition is already a manual-reset event.
- `osd/timing/signals.rs`: SignalState and injected SignalHost expose the signal
  configuration and per-thread setup; capture the constructor's FPE setting.
- `message/defaults.rs` and messenger.rs expose failure reporting and named
  printers suitable for observing logs in tests.
- Select3DSensitiveEntity has virtual bvh() -> Result<(), Failure> and to_build_bvh().

Use deterministic latch/barrier-based fake entities to prove simultaneous builds,
per-worker exclusion, sentry behavior, wait vs queue-empty ordering, duplicate jobs,
stop/restart, retained lifetimes and continued processing after failures. Compare
native public thread/copy behavior where execution is defined. Do not replace the
pool with synchronous processing or omit the mutable thread interface just to pass
small tests. Keep the full 76,041 API scope.

Preserve the original unfinished TKernel linked-slot refactor at
`/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/.claude/worktrees/agent-aeed1901b085eca95`,
branch `worktree-agent-aeed1901b085eca95`, tip `d618906825cab733d312701b51f795dd3d99fe61`.
Those dirty changes remain unvalidated. All original Claude worktrees remain needed
until integration can finish; current Git metadata remains read-only.
