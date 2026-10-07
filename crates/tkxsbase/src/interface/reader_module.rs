// Rust port of DataExchange/TKXSBase/Interface/Interface_ReaderModule (.hxx, .cxx),
// Interface_ReaderLib (.hxx, _0.cxx) and its node classes
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_ReaderModule`: the reading of the entities of a norm from file records,
//! by case number; `Interface_ReaderLib`: its LibCtl library.
use super::{InterfaceCheck, InterfaceFileReaderData, InterfaceProtocol};
use crate::lib_ctl::{LibCtlGlobalNode, LibCtlKind, LibCtlLibrary, LibCtlNode, LibCtlRegistry};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, StandardTransient, Transient},
};
use std::sync::OnceLock;

/// `Interface_ReaderModule`, abstract. `Read` returns `Result` because the readers of
/// the norms raise on malformed parameters.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceReaderModule;
/// fn case(module: &dyn InterfaceReaderModule, data: &rustycad_tkernel::standard::Handle<dyn rustycad_tkxsbase::interface::InterfaceFileReaderData>) -> i32 {
///     module.case_num(data, 1)
/// }
/// ```
pub trait InterfaceReaderModule: Transient {
    /// `CaseNum(data, num)`: the case number of record `num`, 0 when not recognised.
    fn case_num(&self, data: &Handle<dyn InterfaceFileReaderData>, num: i32) -> i32;
    /// `Read(casenum, data, num, ach, ent)`: fills `ent` from record `num`.
    fn read(
        &self,
        casenum: i32,
        data: &Handle<dyn InterfaceFileReaderData>,
        num: i32,
        ach: &mut Handle<InterfaceCheck>,
        ent: &Handle<StandardTransient>,
    ) -> Result<(), Failure>;
    /// `NewRead(casenum, data, num, ach, ent)`: creates and reads an entity; false
    /// (not done) by default.
    fn new_read(
        &self,
        _casenum: i32,
        _data: &Handle<dyn InterfaceFileReaderData>,
        _num: i32,
        _ach: &mut Handle<InterfaceCheck>,
        _ent: &mut Handle<StandardTransient>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
}
impl_transient_interface!(
    dyn InterfaceReaderModule,
    "Interface_ReaderModule",
    StandardTransient
);

/// The instance of the LibCtl template for `Interface_ReaderModule`.
pub struct InterfaceReaderLibKind;

impl LibCtlKind for InterfaceReaderLibKind {
    type Module = dyn InterfaceReaderModule;
    type Protocol = dyn InterfaceProtocol;
    fn registry() -> &'static LibCtlRegistry<Self> {
        static REGISTRY: OnceLock<LibCtlRegistry<InterfaceReaderLibKind>> = OnceLock::new();
        REGISTRY.get_or_init(LibCtlRegistry::new)
    }
}

/// `Interface_ReaderLib`.
pub type InterfaceReaderLib = LibCtlLibrary<InterfaceReaderLibKind>;
/// `Interface_GlobalNodeOfReaderLib`.
pub type InterfaceGlobalNodeOfReaderLib = LibCtlGlobalNode<InterfaceReaderLibKind>;
/// `Interface_NodeOfReaderLib`.
pub type InterfaceNodeOfReaderLib = LibCtlNode<InterfaceReaderLibKind>;
