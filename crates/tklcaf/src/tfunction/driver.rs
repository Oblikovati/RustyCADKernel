// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_ExecutionStatus.hxx,
// TFunction_Driver (.hxx, .lxx and .cxx) and TFunction_DriverTable (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The function drivers and their process-wide table (ocaf-design.md section 8).
use super::TFunctionLogbook;
use crate::live;
use crate::tdf::{GuidHashPolicy, Tdf, TdfLabel};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::DataMap,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, Transient},
    tcollection::ExtendedString,
};
use std::io;
use std::sync::OnceLock;

/// `TFunction_ExecutionStatus`: the execution state of a function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TFunctionExecutionStatus {
    /// `TFunction_ES_WrongDefinition`: the function is not defined properly.
    #[default]
    WrongDefinition,
    /// `TFunction_ES_NotExecuted`: the function waits for its execution.
    NotExecuted,
    /// `TFunction_ES_Executing`: the function is being executed.
    Executing,
    /// `TFunction_ES_Succeeded`: the execution succeeded.
    Succeeded,
    /// `TFunction_ES_Failed`: the execution failed.
    Failed,
}

/// The fields of `TFunction_Driver` (`myLabel`).
#[derive(Default)]
pub struct TFunctionDriverBase {
    label: SharedState<TdfLabel>,
}

/// `TFunction_Driver`: the computation of a function. `Execute` is pure virtual; the
/// other members are provided with the native bodies.
pub trait TFunctionDriver: Transient {
    /// The embedded base (the protected constructor `TFunction_Driver()`).
    fn driver_base(&self) -> &TFunctionDriverBase;
    /// `Init(L)`: the label of the function.
    fn init(&self, label: &TdfLabel) {
        *self.driver_base().label.write() = label.clone();
    }
    /// `Label`.
    fn label(&self) -> TdfLabel {
        self.driver_base().label.read().clone()
    }
    /// `Validate(log)`: every result label becomes valid in the logbook.
    fn validate(&self, log: &Handle<TFunctionLogbook>) -> Result<(), Failure> {
        let mut results = Vec::new();
        self.results(&mut results);
        for result in &results {
            live(log, "TFunction_Driver::Validate")?.set_valid(result, true)?;
        }
        Ok(())
    }
    /// `MustExecute(log)`: an argument label is modified in the logbook.
    fn must_execute(&self, log: &Handle<TFunctionLogbook>) -> Result<bool, Failure> {
        let mut arguments = Vec::new();
        self.arguments(&mut arguments);
        for argument in &arguments {
            if live(log, "TFunction_Driver::MustExecute")?.is_modified(argument, false)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// `Execute(log)` (pure virtual): computes the function; 0 on success.
    fn execute(&self, log: &mut Handle<TFunctionLogbook>) -> Result<i32, Failure>;
    /// `Arguments(args)`: none by default.
    fn arguments(&self, _arguments: &mut Vec<TdfLabel>) {}
    /// `Results(res)`: none by default.
    fn results(&self, _results: &mut Vec<TdfLabel>) {}
}
impl_transient_interface!(dyn TFunctionDriver, "TFunction_Driver", StandardTransient);

/// One table of drivers by GUID.
type DriverMap = DataMap<Guid, Handle<dyn TFunctionDriver>, GuidHashPolicy>;

/// The fields of `TFunction_DriverTable`: the drivers of thread 0 and, from index 1, the
/// per-thread tables (`myThreadDrivers`, null until a thread driver is added).
#[derive(Default)]
struct DriverTableState {
    drivers: DriverMap,
    thread_drivers: Option<Vec<DriverMap>>,
}

/// `TFunction_DriverTable`: the drivers by GUID; `TFunctionDriverTable::get()` is the
/// process-wide table.
///
/// ```
/// use rustycad_tklcaf::tfunction::TFunctionDriverTable;
/// use rustycad_tklcaf::guid_from_text;
/// let id = guid_from_text("0f2a9a10-0000-4000-8000-000000000001");
/// assert!(!TFunctionDriverTable::get().has_driver(&id, 0));
/// ```
#[derive(Default)]
pub struct TFunctionDriverTable {
    transient: StandardTransient,
    state: SharedState<DriverTableState>,
}
impl_transient!(
    TFunctionDriverTable,
    "TFunction_DriverTable",
    StandardTransient,
    transient
);

/// The process-wide table of `TFunction_DriverTable::Get` (a file-static handle natively).
static DRIVER_TABLE: OnceLock<Handle<TFunctionDriverTable>> = OnceLock::new();

impl TFunctionDriverTable {
    /// `Get`: the process-wide table, created on first use.
    pub fn get() -> Handle<TFunctionDriverTable> {
        DRIVER_TABLE
            .get_or_init(|| Handle::new(Self::new()))
            .clone()
    }
    /// `TFunction_DriverTable()`: an empty table.
    pub fn new() -> Self {
        Self::default()
    }
    /// `AddDriver(guid, driver, thread)`: binds the driver of thread 0, or of a positive
    /// thread (growing the per-thread tables); false when the GUID is already bound or the
    /// thread is negative.
    pub fn add_driver(
        &self,
        guid: &Guid,
        driver: &Handle<dyn TFunctionDriver>,
        thread: i32,
    ) -> bool {
        let mut state = self.state.write();
        if thread == 0 {
            return state.drivers.bind(*guid, driver.clone());
        }
        let Ok(index) = usize::try_from(thread) else {
            return false;
        };
        let tables = state.thread_drivers.get_or_insert_with(Vec::new);
        if tables.len() < index {
            tables.resize_with(index, DriverMap::default);
        }
        tables[index - 1].bind(*guid, driver.clone())
    }
    /// The table of a thread, when it exists.
    fn with_table<R>(&self, thread: i32, action: impl FnOnce(&DriverMap) -> R) -> Option<R> {
        let state = self.state.read();
        if thread == 0 {
            return Some(action(&state.drivers));
        }
        let index = usize::try_from(thread).ok().filter(|index| *index > 0)?;
        state.thread_drivers.as_ref()?.get(index - 1).map(action)
    }
    /// `HasDriver(guid, thread)`.
    pub fn has_driver(&self, guid: &Guid, thread: i32) -> bool {
        self.with_table(thread, |table| table.is_bound(guid))
            .unwrap_or(false)
    }
    /// `FindDriver(guid, driver, thread)`: true and the driver when bound.
    pub fn find_driver(
        &self,
        guid: &Guid,
        driver: &mut Handle<dyn TFunctionDriver>,
        thread: i32,
    ) -> bool {
        let found = self
            .with_table(thread, |table| table.find(guid).ok().cloned())
            .flatten();
        let Some(found) = found else {
            return false;
        };
        *driver = found;
        true
    }
    /// `Dump` (and `operator<<`): one `"<guid>\t<ProgID>\n"` line per driver of thread 0.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let guids: Vec<Guid> = self
            .state
            .read()
            .drivers
            .items()
            .map(|(guid, _)| *guid)
            .collect();
        for guid in guids {
            let mut text = String::new();
            guid.shallow_dump(&mut text).map_err(io::Error::other)?;
            let mut prog_id = ExtendedString::default();
            Tdf::prog_id_from_guid(&guid, &mut prog_id);
            writeln!(stream, "{text}\t{}", prog_id.to_utf8())?;
        }
        Ok(())
    }
    /// `RemoveDriver(guid, thread)`: true when thread 0 unbinds it; a thread driver is
    /// removed but false is returned, as natively.
    pub fn remove_driver(&self, guid: &Guid, thread: i32) -> bool {
        let mut state = self.state.write();
        if thread == 0 {
            return state.drivers.unbind(guid);
        }
        let index = usize::try_from(thread).ok().filter(|index| *index > 0);
        let tables = state.thread_drivers.as_mut();
        if let Some(table) = index
            .zip(tables)
            .and_then(|(index, tables)| tables.get_mut(index - 1))
        {
            table.unbind(guid);
        }
        false
    }
    /// `Clear`: forgets every driver and the per-thread tables.
    pub fn clear(&self) {
        let mut state = self.state.write();
        state.drivers.clear(false);
        state.thread_drivers = None;
    }
}
