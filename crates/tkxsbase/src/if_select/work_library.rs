// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkLibrary (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_WorkLibrary`: the reading, writing, copying and dumping services of a norm
//! for the work session.
use super::IFSelectContextWrite;
use crate::interface::{
    InterfaceCopyTool, InterfaceEntityIterator, InterfaceInterfaceModel, InterfaceProtocol,
};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
    tcollection::AsciiString,
};
use std::io::{Read, Write};

/// The fields of `IFSelect_WorkLibrary`: the default dump level and the level helps.
#[derive(Default)]
pub struct IFSelectWorkLibraryBase {
    thelevdef: SharedState<i32>,
    thelevhlp: SharedState<Option<Vec<AsciiString>>>,
}

impl IFSelectWorkLibraryBase {
    /// `IFSelect_WorkLibrary()`: default level 0, no levels.
    ///
    /// ```
    /// use rustycad_tkxsbase::if_select::IFSelectWorkLibraryBase;
    /// let _base = IFSelectWorkLibraryBase::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

/// `IFSelect_WorkLibrary`, abstract: `ReadFile`, `WriteFile` and `DumpEntity` (by
/// level) are required; the protected constructor is the Default of
/// [`IFSelectWorkLibraryBase`].
pub trait IFSelectWorkLibrary: Transient {
    /// The embedded fields.
    fn work_library_base(&self) -> &IFSelectWorkLibraryBase;
    /// `ReadFile(name, model, protocol)`: 0 when read, else an error status.
    fn read_file(
        &self,
        name: &str,
        model: &mut Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<i32, Failure>;
    /// `ReadStream(name, istream, model, protocol)`: 1 (not supported) by default.
    fn read_stream(
        &self,
        _name: &str,
        _istream: &mut dyn Read,
        _model: &mut Handle<dyn InterfaceInterfaceModel>,
        _protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<i32, Failure> {
        Ok(1)
    }
    /// `WriteFile(ctx)`: true when written.
    fn write_file(&self, ctx: &mut IFSelectContextWrite) -> Result<bool, Failure>;
    /// `WriteStream(ctx, ostream)`: false (not supported) by default.
    fn write_stream(
        &self,
        _ctx: &mut IFSelectContextWrite,
        _ostream: &mut dyn Write,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `CopyModel(original, newmodel, list, TC)`: transfers the entities of `list`, then
    /// fills the new model; true.
    fn copy_model(
        &self,
        _original: &Handle<dyn InterfaceInterfaceModel>,
        newmodel: &Handle<dyn InterfaceInterfaceModel>,
        list: &InterfaceEntityIterator,
        tc: &mut InterfaceCopyTool,
    ) -> Result<bool, Failure> {
        for ent in list.content() {
            tc.transfer_entity(&ent)?;
        }
        tc.fill_model(newmodel)?;
        Ok(true)
    }
    /// `DumpEntity(model, protocol, entity, S, level)`.
    fn dump_entity(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        entity: &Handle<StandardTransient>,
        s: &mut dyn Write,
        level: i32,
    ) -> Result<(), Failure>;
    /// `DumpEntity(model, protocol, entity, S)`: at the default level (0 without levels).
    fn dump_entity_default(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        entity: &Handle<StandardTransient>,
        s: &mut dyn Write,
    ) -> Result<(), Failure> {
        let base = self.work_library_base();
        let level = if base.thelevhlp.read().is_none() {
            0
        } else {
            *base.thelevdef.read()
        };
        self.dump_entity(model, protocol, entity, s, level)
    }
    /// `SetDumpLevels(def, max)`: the default level and the levels 0..max (none for a
    /// negative max).
    fn set_dump_levels(&self, def: i32, max: i32) {
        let base = self.work_library_base();
        *base.thelevdef.write() = def;
        *base.thelevhlp.write() =
            (max >= 0).then(|| vec![AsciiString::default(); max as usize + 1]);
    }
    /// `DumpLevels(def, max)`: the default and the highest level ((0, -1) without levels).
    fn dump_levels(&self) -> (i32, i32) {
        let base = self.work_library_base();
        match base.thelevhlp.read().as_ref() {
            Some(levels) => (*base.thelevdef.read(), levels.len() as i32 - 1),
            None => (0, -1),
        }
    }
    /// `SetDumpHelp(level, help)`: the help of a level (nothing out of the levels).
    fn set_dump_help(&self, level: i32, help: &str) {
        let mut levels = self.work_library_base().thelevhlp.write();
        if let Some(slot) = levels
            .as_mut()
            .and_then(|l| usize::try_from(level).ok().and_then(|i| l.get_mut(i)))
        {
            *slot = AsciiString::new(help);
        }
    }
    /// `DumpHelp(level)`: empty out of the levels.
    fn dump_help(&self, level: i32) -> AsciiString {
        let levels = self.work_library_base().thelevhlp.read();
        levels
            .as_ref()
            .and_then(|l| usize::try_from(level).ok().and_then(|i| l.get(i)))
            .cloned()
            .unwrap_or_default()
    }
}
impl_transient_interface!(
    dyn IFSelectWorkLibrary,
    "IFSelect_WorkLibrary",
    StandardTransient
);
