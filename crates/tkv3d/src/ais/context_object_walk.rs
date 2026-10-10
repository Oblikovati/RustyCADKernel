//! Reopen bucket chains between callbacks, retaining native position rather than a key snapshot.
use super::{AisInteractiveContext, AisInteractiveObject};
use rustycad_tkernel::standard::{Failure, Handle, HandleObserver, exceptions::program_error};
pub(super) struct AisObjectWalk {
    bucket: usize,
    table: usize,
    current: HandleObserver<dyn AisInteractiveObject>,
}
impl AisObjectWalk {
    pub(super) fn new(context: &AisInteractiveContext) -> Result<Self, Failure> {
        let table = context
            .fields
            .read()
            .objects
            .buckets()
            .map_or(0, |b| b.identity() as usize);
        let mut walk = Self {
            bucket: 0,
            table,
            current: Handle::<dyn AisInteractiveObject>::default().observe(),
        };
        walk.seek_bucket(context)?;
        Ok(walk)
    }
    pub(super) fn current(&self) -> Option<&Handle<dyn AisInteractiveObject>> {
        (!self.current.is_null()).then_some(&self.current)
    }
    pub(super) fn next(&mut self, context: &AisInteractiveContext) -> Result<(), Failure> {
        if self.current.is_null() {
            return Ok(());
        }
        let following = self.chain_successor(context)?;
        self.current = following;
        if self.current.is_null() {
            self.bucket += 1;
            self.seek_bucket(context)?;
        }
        Ok(())
    }
    fn chain_successor(
        &self,
        context: &AisInteractiveContext,
    ) -> Result<HandleObserver<dyn AisInteractiveObject>, Failure> {
        let fields = context.fields.read();
        let table = fields
            .objects
            .buckets()
            .ok_or_else(|| self.invalidated(0))?;
        if table.identity() as usize != self.table {
            return Err(self.invalidated(table.identity() as usize));
        }
        let mut chain = table.chain(self.bucket)?;
        if !chain.any(|(key, _)| key == &*self.current) {
            return Err(self.invalidated(self.table));
        }
        Ok(chain.next().map_or_else(
            || Handle::<dyn AisInteractiveObject>::default().observe(),
            |(key, _)| key.observe(),
        ))
    }
    fn seek_bucket(&mut self, context: &AisInteractiveContext) -> Result<(), Failure> {
        let fields = context.fields.read();
        let Some(table) = fields.objects.buckets() else {
            return Ok(());
        };
        while self.bucket < table.len() {
            if let Some((key, _)) = table.chain(self.bucket)?.next() {
                self.current = key.observe();
                break;
            }
            self.bucket += 1;
        }
        Ok(())
    }
    fn invalidated(&self, actual: usize) -> Failure {
        // Upstream iterators are invalid after rehash or deleting the current node (OCCT 3d097a0).
        program_error(&format!(
            "AIS traversal table {actual:#x}, object {:?}; expected live node in table {:#x}",
            self.current.object_address(),
            self.table
        ))
    }
}
