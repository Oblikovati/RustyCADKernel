//! Reopen bucket chains between callbacks, retaining native position rather than a key snapshot.
use super::{AisInteractiveContext, AisInteractiveObject};
use rustycad_tkernel::standard::{Failure, Handle, HandleObserver, exceptions::program_error};
pub(super) struct AisObjectWalk {
    bucket: usize,
    table: usize,
    current: Option<HandleObserver<dyn AisInteractiveObject>>,
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
            current: None,
        };
        walk.seek_bucket(context)?;
        Ok(walk)
    }
    pub(super) fn current(&self) -> Option<&Handle<dyn AisInteractiveObject>> {
        self.current.as_deref()
    }
    pub(super) fn next(&mut self, context: &AisInteractiveContext) -> Result<(), Failure> {
        if self.current.is_none() {
            return Ok(());
        }
        let following = self.chain_successor(context)?;
        self.current = following;
        if self.current.is_none() {
            self.bucket += 1;
            self.seek_bucket(context)?;
        }
        Ok(())
    }
    fn chain_successor(
        &self,
        context: &AisInteractiveContext,
    ) -> Result<Option<HandleObserver<dyn AisInteractiveObject>>, Failure> {
        let fields = context.fields.read();
        let table = fields
            .objects
            .buckets()
            .ok_or_else(|| self.invalidated(0))?;
        if table.identity() as usize != self.table {
            return Err(self.invalidated(table.identity() as usize));
        }
        let current = self
            .current
            .as_deref()
            .expect("AIS cursor None; expected a current map node");
        let mut chain = table.chain(self.bucket)?;
        if !chain.any(|(key, _)| key == current) {
            return Err(self.invalidated(self.table));
        }
        Ok(chain.next().map(|(key, _)| key.observe()))
    }
    fn seek_bucket(&mut self, context: &AisInteractiveContext) -> Result<(), Failure> {
        let fields = context.fields.read();
        let Some(table) = fields.objects.buckets() else {
            return Ok(());
        };
        while self.bucket < table.len() {
            if let Some((key, _)) = table.chain(self.bucket)?.next() {
                self.current = Some(key.observe());
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
            self.current.as_ref().map(|value| value.object_address()),
            self.table
        ))
    }
}
