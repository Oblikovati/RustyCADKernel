//! Keep native saved-next and old metadata through payload destruction and allocator Free.
use super::{
    Sequence, SequenceCursor, SequenceNodeRetirement,
    base_storage::SequenceNodeLiveness,
    owner::{SequenceNodeOwners, take_owner},
};
use crate::standard::{Failure, exceptions::program_error, raise_native_failure};

/// Native clear traversal over actual nodes; `let progress=sequence.begin_clear_progress()`.
/// Visit captures Next before callbacks; detach/free afterward, then reset only at completion.
pub struct SequenceClearProgress {
    scope_identity: usize,
    current: SequenceCursor,
    pending: Option<SequenceCursor>,
    saved_next: SequenceCursor,
}
unsafe fn registry_contains<T>(registry: *const (), address: usize) -> bool {
    // SAFETY: the token's sequence owns this boxed registry for every scoped topology access.
    let owners = unsafe { &*registry.cast::<SequenceNodeOwners<T>>() };
    owners.contains_key(&address)
}
impl<T> Sequence<T> {
    /// Start ordered clearing without changing old metadata; `sequence.begin_clear_progress()`.
    pub fn begin_clear_progress(&mut self) -> SequenceClearProgress {
        if self.topology.access().clear_node_scope.is_some() {
            return raise_native_failure(Err(clear_phase_failure("Begin", "an idle sequence")));
        }
        let scope = self.clear_node_liveness();
        self.topology
            .edit(|base| base.clear_node_scope = Some(scope));
        SequenceClearProgress {
            scope_identity: self.topology.identity(),
            current: self.cursor(),
            pending: None,
            saved_next: SequenceCursor::default(),
        }
    }
    fn clear_node_liveness(&self) -> SequenceNodeLiveness {
        SequenceNodeLiveness {
            registry: std::ptr::from_ref(self.owners.as_ref()).cast(),
            contains: registry_contains::<T>,
        }
    }
    /// Capture current payload and Next before its callbacks; `sequence.visit_clear_node(&mut progress)?`.
    pub fn visit_clear_node<'a>(
        &'a self,
        progress: &mut SequenceClearProgress,
    ) -> Result<Option<&'a T>, Failure> {
        self.check_clear_progress(progress)?;
        if progress.pending.is_some() {
            return Err(clear_phase_failure("Visit", "no pending node retirement"));
        }
        if !progress.current.more() {
            return Ok(None);
        }
        self.capture_clear_payload(progress).map(Some)
    }
    fn capture_clear_payload<'a>(
        &'a self,
        progress: &mut SequenceClearProgress,
    ) -> Result<&'a T, Failure> {
        let value = progress
            .current
            .value(self)
            .ok_or_else(|| super::no_such_node("Clear", progress.current))?;
        progress.saved_next = progress.current;
        progress.saved_next.next(self);
        progress.pending = Some(progress.current);
        Ok(value)
    }
    /// Detach the visited node while preserving old size/endpoints; `sequence.detach_clear_node(&mut progress)?`.
    /// Drop the returned allocation outside the adapter guard, after its payload has retired.
    pub fn detach_clear_node(
        &mut self,
        progress: &mut SequenceClearProgress,
    ) -> Result<SequenceNodeRetirement<T>, Failure> {
        self.check_clear_progress(progress)?;
        let cursor = progress
            .pending
            .ok_or_else(|| clear_phase_failure("Detach", "one visited pending node"))?;
        let node = self
            .owned_node(cursor.current_node())
            .ok_or_else(|| super::no_such_node("Clear", cursor))?;
        let owner = take_owner(&mut self.owners, node);
        progress.pending = None;
        progress.current = progress.saved_next;
        Ok(SequenceNodeRetirement { owner })
    }
    /// Reset after final Free, preserving native lost callback-added ownership; `sequence.finish_clear_progress(progress)?`.
    pub fn finish_clear_progress(
        &mut self,
        progress: SequenceClearProgress,
    ) -> Result<(), Failure> {
        self.check_clear_progress(&progress)?;
        if progress.pending.is_some() || progress.current.more() {
            return Err(clear_phase_failure(
                "Finish",
                "an exhausted traversal with no pending node",
            ));
        }
        self.reset_clear_metadata();
        self.abandon_unvisited_clear_nodes();
        Ok(())
    }
    pub(super) fn reset_clear_metadata(&mut self) {
        self.topology.edit(|base| {
            base.nullify();
            base.clear_node_scope = None;
        });
    }
    fn abandon_unvisited_clear_nodes(&mut self) {
        // Native Clear saves Next before Delete: nodes appended in the last callback are lost.
        for owner in (*std::mem::take(&mut self.owners)).into_values() {
            owner.abandon();
        }
    }
    fn check_clear_progress(&self, progress: &SequenceClearProgress) -> Result<(), Failure> {
        let identity = self.topology.identity();
        if progress.scope_identity != identity || self.topology.access().clear_node_scope.is_none()
        {
            return Err(program_error(&format!(
                "clear token scope {}, sequence {identity}; expected the active canonical sequence",
                progress.scope_identity
            )));
        }
        Ok(())
    }
}
fn clear_phase_failure(member: &str, expected: &str) -> Failure {
    program_error(&format!(
        "controlled sequence Clear::{member} has incompatible progress; expected {expected}"
    ))
}
#[cfg(test)]
#[path = "controlled_clear_tests.rs"]
mod tests;
