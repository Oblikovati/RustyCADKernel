// Rust port of DataExchange/TKXSBase/Interface/Interface_FileReaderTool (.hxx, .cxx: the
// fields, the accessors and the library recognition) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_FileReaderTool`: drives the reading of a file: recognises the records,
//! creates and reads the entities, records their reports in the model. The loading
//! (`SetEntities`, `LoadModel`, `LoadedEntity`) is in `file_reader_load`.
use super::{
    InterfaceCheck, InterfaceFileReaderData, InterfaceGeneralLib, InterfaceInterfaceModel,
    InterfaceProtocol, InterfaceReaderLib,
};
use rustycad_tkernel::{
    message::{Message, Messenger},
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
};
use std::sync::Arc;

/// The private fields of `Interface_FileReaderTool`.
pub(super) struct ReaderToolState {
    pub(super) theproto: Handle<dyn InterfaceProtocol>,
    pub(super) thereader: Handle<dyn InterfaceFileReaderData>,
    pub(super) themodel: Handle<dyn InterfaceInterfaceModel>,
    pub(super) themessenger: Option<Arc<Messenger>>,
    pub(super) thetrace: i32,
    pub(super) theerrhand: bool,
    pub(super) thenbrep0: i32,
    pub(super) thenbreps: i32,
    pub(super) thereports: Option<Vec<Handle<StandardTransient>>>,
}

/// The embedded base of a file reader tool; `InterfaceFileReaderToolBase::default()` is the
/// native protected constructor (default messenger, error handling on, trace 0).
pub struct InterfaceFileReaderToolBase {
    pub(super) state: SharedState<ReaderToolState>,
}

impl Default for InterfaceFileReaderToolBase {
    fn default() -> Self {
        Self {
            state: SharedState::new(ReaderToolState {
                theproto: Handle::default(),
                thereader: Handle::default(),
                themodel: Handle::default(),
                themessenger: Some(Message::default_messenger().clone()),
                thetrace: 0,
                theerrhand: true,
                thenbrep0: 0,
                thenbreps: 0,
                thereports: None,
            }),
        }
    }
}

/// `Interface_FileReaderTool`, abstract: `Recognize`, `BeginRead` and `AnalyseRecord` are
/// required, `EndRead` is provided. The non-virtual members are the extension trait
/// [`InterfaceFileReaderToolOps`](super::InterfaceFileReaderToolOps).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceFileReaderTool;
/// fn trace(tool: &dyn InterfaceFileReaderTool) -> i32 {
///     use rustycad_tkxsbase::interface::InterfaceFileReaderToolOps;
///     tool.trace_level()
/// }
/// ```
pub trait InterfaceFileReaderTool {
    /// The embedded private fields.
    fn reader_tool_base(&self) -> &InterfaceFileReaderToolBase;
    /// `Recognize(num, ach, ent)`: creates the empty entity of record `num`; false when the
    /// record is not recognised.
    fn recognize(
        &self,
        num: i32,
        ach: &mut Handle<InterfaceCheck>,
        ent: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure>;
    /// `BeginRead(amodel)`: reads the header.
    fn begin_read(&self, amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<(), Failure>;
    /// `AnalyseRecord(num, anent, acheck)`: fills an entity from its record.
    fn analyse_record(
        &self,
        num: i32,
        anent: &Handle<StandardTransient>,
        acheck: &mut Handle<InterfaceCheck>,
    ) -> Result<bool, Failure>;
    /// `EndRead(amodel)`: nothing by default.
    fn end_read(&self, _amodel: &Handle<dyn InterfaceInterfaceModel>) -> Result<(), Failure> {
        Ok(())
    }
}

/// The non-virtual accessors of `Interface_FileReaderTool`.
pub trait InterfaceFileReaderToolOps: InterfaceFileReaderTool {
    /// `SetData(reader, protocol)`.
    fn set_data(
        &self,
        reader: &Handle<dyn InterfaceFileReaderData>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) {
        let mut state = self.reader_tool_base().state.write();
        state.thereader = reader.clone();
        state.theproto = protocol.clone();
    }
    /// `Protocol`.
    fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.reader_tool_base().state.read().theproto.clone()
    }
    /// `Data`.
    fn data(&self) -> Handle<dyn InterfaceFileReaderData> {
        self.reader_tool_base().state.read().thereader.clone()
    }
    /// `SetModel(amodel)`.
    fn set_model(&self, amodel: &Handle<dyn InterfaceInterfaceModel>) {
        self.reader_tool_base().state.write().themodel = amodel.clone();
    }
    /// `Model`.
    fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.reader_tool_base().state.read().themodel.clone()
    }
    /// `SetMessenger(messenger)`: the default messenger for `None`.
    fn set_messenger(&self, messenger: Option<Arc<Messenger>>) {
        let messenger = messenger.unwrap_or_else(|| Message::default_messenger().clone());
        self.reader_tool_base().state.write().themessenger = Some(messenger);
    }
    /// `Messenger`.
    fn messenger(&self) -> Option<Arc<Messenger>> {
        self.reader_tool_base().state.read().themessenger.clone()
    }
    /// `SetTraceLevel(tracelev)`.
    fn set_trace_level(&self, tracelev: i32) {
        self.reader_tool_base().state.write().thetrace = tracelev;
    }
    /// `TraceLevel`.
    fn trace_level(&self) -> i32 {
        self.reader_tool_base().state.read().thetrace
    }
    /// `SetErrorHandle(err)`.
    fn set_error_handle(&self, err: bool) {
        self.reader_tool_base().state.write().theerrhand = err;
    }
    /// `ErrorHandle`.
    fn error_handle(&self) -> bool {
        self.reader_tool_base().state.read().theerrhand
    }
    /// `RecognizeByLib(num, glib, rlib, ach, ent)`: the first reader module giving a case
    /// number to record `num`, then the general module of the same protocol type for an
    /// empty entity (or the reader's `NewRead`).
    fn recognize_by_lib(
        &self,
        num: i32,
        glib: &mut InterfaceGeneralLib,
        rlib: &mut InterfaceReaderLib,
        ach: &mut Handle<InterfaceCheck>,
        ent: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        let reader = self.data();
        let Some((rmod, cn, typrot)) = reader_case(rlib, &reader, num)? else {
            return Ok(false);
        };
        glib.start();
        while glib.more() {
            let proto = glib.protocol()?;
            let same = proto
                .dynamic_type()
                .is_some_and(|t| std::ptr::eq(t, typrot));
            if same {
                if glib.module()?.new_void(cn, ent) {
                    return Ok(true);
                }
                return rmod.new_read(cn, &reader, num, ach, ent);
            }
            glib.next();
        }
        Ok(false)
    }
    /// `UnknownEntity`: from the protocol.
    fn unknown_entity(&self) -> Result<Handle<StandardTransient>, Failure> {
        Ok(live_protocol(&self.protocol())?.unknown_entity())
    }
    /// `NewModel`: from the protocol.
    fn new_model(&self) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        Ok(live_protocol(&self.protocol())?.new_model())
    }
    /// `Clear`: forgets the protocol, the data, the model and the reports.
    fn clear(&self) {
        let mut state = self.reader_tool_base().state.write();
        state.theproto.nullify();
        state.thereader.nullify();
        state.themodel.nullify();
        state.thereports = None;
    }
}

impl<T: InterfaceFileReaderTool + ?Sized> InterfaceFileReaderToolOps for T {}

/// The reader module, case number and protocol type of a recognised record.
type ReaderCase = (
    Handle<dyn super::InterfaceReaderModule>,
    i32,
    &'static rustycad_tkernel::standard::RuntimeType,
);

/// The reader module, case number and protocol type of the first reader module that
/// recognises record `num`.
fn reader_case(
    rlib: &mut InterfaceReaderLib,
    reader: &Handle<dyn InterfaceFileReaderData>,
    num: i32,
) -> Result<Option<ReaderCase>, Failure> {
    rlib.start();
    while rlib.more() {
        let rmod = rlib.module()?;
        if let Some(module) = rmod.get() {
            let cn = module.case_num(reader, num);
            if cn > 0 {
                let proto = rlib.protocol()?;
                return Ok(proto.dynamic_type().map(|t| (rmod.clone(), cn, t)));
            }
        }
        rlib.next();
    }
    Ok(None)
}

/// The protocol, dereferenced natively without a check.
pub(super) fn live_protocol(
    proto: &Handle<dyn InterfaceProtocol>,
) -> Result<&dyn InterfaceProtocol, Failure> {
    proto.get().ok_or_else(|| {
        exceptions::null_object("Interface_FileReaderTool: no protocol; expected SetData first")
    })
}
