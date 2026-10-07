// Rust port of DataExchange/TKXSBase/Transfer/Transfer_FinderProcess (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_FinderProcess`: the process that writes objects (by finders) into an
//! interface model.
use super::{
    TransferFinder, TransferFinderKind, TransferProcess, TransferProcessForFinder,
    TransferProcessOps, TransferTransientMapper, bind_new, transient_process::print_result_counts,
};
use crate::interface::InterfaceInterfaceModel;
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, SharedState, StandardTransient},
};
use std::io::{self, Write};
use std::ops::Deref;

/// `Transfer_FinderProcess`: embeds its `Transfer_ProcessForFinder` (registered as its
/// RTTI parent) and dereferences to it.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferFinderProcess;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let fp = TransferFinderProcess::new(100);
/// let mapper = fp.transient_mapper(&Handle::new(StandardTransient::default()));
/// assert!(mapper.is_some() && fp.nb_mapped() == 0);
/// ```
pub struct TransferFinderProcess {
    base: TransferProcessForFinder,
    themodel: SharedState<Handle<dyn InterfaceInterfaceModel>>,
}
impl_transient!(TransferFinderProcess, "Transfer_FinderProcess", TransferProcessForFinder, base, interfaces: [dyn TransferProcessOps<TransferFinderKind>]);

impl Deref for TransferFinderProcess {
    type Target = TransferProcessForFinder;
    fn deref(&self) -> &TransferProcessForFinder {
        &self.base
    }
}

impl TransferFinderProcess {
    /// `Transfer_FinderProcess(nb = 10000)`.
    pub fn new(nb: i32) -> Handle<Self> {
        bind_new(Self {
            base: TransferProcessForFinder::unbound(TransferProcess::new(nb)),
            themodel: SharedState::new(Handle::default()),
        })
    }
    /// `SetModel(model)`.
    pub fn set_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        *self.themodel.write() = model.clone();
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themodel.read().clone()
    }
    /// `NextMappedWithAttribute(name, num0)`: the first mapped finder after `num0` with the
    /// attribute `name` (0 when none).
    pub fn next_mapped_with_attribute(&self, name: &str, num0: i32) -> i32 {
        let nb = self.nb_mapped();
        let first = num0.saturating_add(1).max(1);
        (first..=nb)
            .find(|num| {
                self.mapped(*num)
                    .ok()
                    .and_then(|f| f.get().map(|live| live.attribute(name).is_some()))
                    .unwrap_or(false)
            })
            .unwrap_or(0)
    }
    /// `TransientMapper(obj)`: the mapped mapper of `obj`, else a new (unmapped) one.
    pub fn transient_mapper(
        &self,
        obj: &Handle<StandardTransient>,
    ) -> Handle<TransferTransientMapper> {
        let mapper = Handle::new(TransferTransientMapper::new(obj));
        let index = self.map_index(&mapper.cast());
        if index == 0 {
            return mapper;
        }
        self.mapped(index).map(|f| f.cast()).unwrap_or_default()
    }
    /// `PrintStats(mode, S)`: the counts of results, errors and warnings for mode 1.
    /// Native quirk kept: "Errors on :" has no blank before the count blanks.
    pub fn print_stats(&self, mode: i32, s: &mut dyn Write) -> io::Result<()> {
        s.write_all(b"\n*******************************************************************\n")?;
        if mode == 1 {
            s.write_all(b"********                 Basic Statistics                  ********\n")?;
            print_result_counts(self.process(), s, "")?;
            s.write_all(b"*******************************************************************")?;
        }
        s.write_all(b"\n")
    }
}

impl TransferProcessOps<TransferFinderKind> for TransferFinderProcess {
    fn process(&self) -> &TransferProcess<TransferFinderKind> {
        &self.base
    }
    /// `PrintTrace(start, S)`: " Type:" and the value type name of the finder.
    fn print_trace(&self, start: &Handle<dyn TransferFinder>, s: &mut dyn Write) -> io::Result<()> {
        match start.get() {
            Some(finder) => write!(s, " Type:{}", finder.value_type_name()),
            None => Ok(()),
        }
    }
}
