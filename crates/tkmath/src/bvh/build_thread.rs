// Rust translation of FoundationClasses/TKMath/BVH/BVH_BuildThread.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BuildTool` and `BVH_BuildThread`: workers draining a build queue.
use super::build_queue::BvhBuildQueue;
use rustycad_tkernel::standard::Failure;
use std::thread::{Scope, ScopedJoinHandle};

/// Tool object to call BVH builder subroutines; shared by all build threads,
/// so it synchronizes internally. Example: `tool.perform(0)?`.
pub trait BvhBuildTool: Sync {
    /// Performs splitting of the given BVH node.
    fn perform(&self, node: i32) -> Result<(), Failure>;
}

/// Wrapper for a BVH build thread; `OSD_Thread` becomes a scoped OS thread.
/// Example: `BvhBuildThread::new(&tool, &queue).execute()?`.
pub struct BvhBuildThread<'a> {
    build_tool: &'a dyn BvhBuildTool,
    build_queue: &'a BvhBuildQueue,
}
impl<'a> BvhBuildThread<'a> {
    /// Creates new BVH build thread; `BvhBuildThread::new(&tool, &queue)`.
    pub fn new(build_tool: &'a dyn BvhBuildTool, build_queue: &'a BvhBuildQueue) -> Self {
        Self {
            build_tool,
            build_queue,
        }
    }

    /// Processes queued nodes until the queue is empty and no thread is busy
    /// (other threads may still enqueue children, so an idle thread spins).
    /// A failing tool leaves the busy count before the error is returned.
    /// Example: `thread.execute()?` runs the loop on the calling thread.
    pub fn execute(&self) -> Result<(), Failure> {
        let mut was_busy = false;
        loop {
            let node = self.build_queue.fetch(&mut was_busy);
            if node != -1 {
                if let Err(failure) = self.build_tool.perform(node) {
                    self.build_queue.abandon(was_busy);
                    return Err(failure);
                }
                continue;
            }
            if !self.build_queue.has_busy_threads() {
                return Ok(());
            }
            std::hint::spin_loop();
        }
    }

    /// Starts execution on a new OS thread of `scope` (`Run`).
    /// Example: `let handle = thread.run(scope);`.
    pub fn run<'scope, 'env>(
        &'scope self,
        scope: &'scope Scope<'scope, 'env>,
    ) -> ScopedJoinHandle<'scope, Result<(), Failure>>
    where
        'a: 'scope,
    {
        scope.spawn(move || self.execute())
    }

    /// Waits till the thread finishes execution (`Wait`); a panic becomes a failure.
    /// Example: `BvhBuildThread::wait(handle)?`.
    pub fn wait(handle: ScopedJoinHandle<'_, Result<(), Failure>>) -> Result<(), Failure> {
        handle
            .join()
            .unwrap_or_else(|_| Err(Failure::new("BVH build thread panicked")))
    }
}
