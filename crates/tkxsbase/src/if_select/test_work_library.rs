//! A fake work library for the tests of the model copier, the transformers and the work
//! session: it writes nothing, records the files it was asked to write and fails on the
//! names containing "fail".
use super::{IFSelectContextWrite, IFSelectWorkLibrary, IFSelectWorkLibraryBase};
use crate::interface::test_norm::fake_model;
use crate::interface::{InterfaceInterfaceModel, InterfaceProtocol};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use std::io::Write;

/// The fake work library.
#[derive(Default)]
pub(crate) struct FakeWorkLibrary {
    transient: StandardTransient,
    base: IFSelectWorkLibraryBase,
    pub(crate) written: SharedState<Vec<String>>,
}
impl_transient!(FakeWorkLibrary, "Fake_WorkLibrary", StandardTransient, transient, interfaces: [dyn IFSelectWorkLibrary]);

impl IFSelectWorkLibrary for FakeWorkLibrary {
    fn work_library_base(&self) -> &IFSelectWorkLibraryBase {
        &self.base
    }
    fn read_file(
        &self,
        name: &str,
        model: &mut Handle<dyn InterfaceInterfaceModel>,
        _protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<i32, Failure> {
        if name.contains("fail") {
            return Ok(-1);
        }
        *model = fake_model();
        Ok(0)
    }
    fn write_file(&self, ctx: &mut IFSelectContextWrite) -> Result<bool, Failure> {
        let name = String::from_utf8_lossy(ctx.file_name().as_bytes()).into_owned();
        let ok = !name.contains("fail");
        self.written.write().push(name);
        Ok(ok)
    }
    fn dump_entity(
        &self,
        _model: &Handle<dyn InterfaceInterfaceModel>,
        _protocol: &Handle<dyn InterfaceProtocol>,
        _entity: &Handle<StandardTransient>,
        s: &mut dyn Write,
        level: i32,
    ) -> Result<(), Failure> {
        let _ = write!(s, "level {level}");
        Ok(())
    }
}

/// A new fake work library as the abstract handle.
pub(crate) fn fake_work_library() -> Handle<dyn IFSelectWorkLibrary> {
    Handle::new(FakeWorkLibrary::default()).cast()
}
