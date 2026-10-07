// Rust port of DataExchange/TKXSBase/Interface/Interface_GeneralLib (.hxx, _0.cxx) and
// its node classes Interface_GlobalNodeOfGeneralLib and Interface_NodeOfGeneralLib
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_GeneralLib`: the LibCtl library of general modules
//! (xsbase-design.md section 2).
use super::{InterfaceGeneralModule, InterfaceProtocol};
use crate::lib_ctl::{LibCtlGlobalNode, LibCtlKind, LibCtlLibrary, LibCtlNode, LibCtlRegistry};
use std::sync::OnceLock;

/// The instance of the LibCtl template for `Interface_GeneralModule`.
pub struct InterfaceGeneralLibKind;

impl LibCtlKind for InterfaceGeneralLibKind {
    type Module = dyn InterfaceGeneralModule;
    type Protocol = dyn InterfaceProtocol;
    fn registry() -> &'static LibCtlRegistry<Self> {
        static REGISTRY: OnceLock<LibCtlRegistry<InterfaceGeneralLibKind>> = OnceLock::new();
        REGISTRY.get_or_init(LibCtlRegistry::new)
    }
}

/// `Interface_GeneralLib`;
/// `InterfaceGeneralLib::new(&protocol).select(&entity)` gives the module and case number.
pub type InterfaceGeneralLib = LibCtlLibrary<InterfaceGeneralLibKind>;
/// `Interface_GlobalNodeOfGeneralLib`.
pub type InterfaceGlobalNodeOfGeneralLib = LibCtlGlobalNode<InterfaceGeneralLibKind>;
/// `Interface_NodeOfGeneralLib`.
pub type InterfaceNodeOfGeneralLib = LibCtlNode<InterfaceGeneralLibKind>;
