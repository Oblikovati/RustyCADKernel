// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_Driver.hxx and .cxx and
// TPrsStd_DriverTable.hxx and .cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TPrsStd_Driver` (the abstract presentation builder of a label) and `TPrsStd_DriverTable`
//! (the drivers by GUID, with the process-wide table of `Get`).
use super::{
    TPrsStdAxisDriver, TPrsStdConstraintDriver, TPrsStdGeometryDriver, TPrsStdNamedShapeDriver,
    TPrsStdPlaneDriver, TPrsStdPointDriver,
};
use rustycad_tkcaf::tdata_xtd::{
    TDataXtdAxis, TDataXtdConstraint, TDataXtdGeometry, TDataXtdPlane, TDataXtdPoint,
};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::DataMap,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, Transient},
};
use rustycad_tklcaf::tdf::{GuidHashPolicy, TdfLabel};
use rustycad_tkv3d::ais::AisInteractiveObject;
use std::sync::OnceLock;

/// `TPrsStd_Driver`: builds or updates the interactive object of a label from the
/// attributes found on it. `Update` is pure virtual natively.
pub trait TPrsStdDriver: Transient {
    /// `Update(L, ais)`: true when `ais` holds the (new or updated) interactive object of the
    /// label; `ais` may come in null or holding the previous object.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure>;
}
impl_transient_interface!(dyn TPrsStdDriver, "TPrsStd_Driver", StandardTransient);

/// The drivers by GUID (`NCollection_DataMap<Standard_GUID, handle<TPrsStd_Driver>>`).
type PresentationDriverMap = DataMap<Guid, Handle<dyn TPrsStdDriver>, GuidHashPolicy>;

/// `TPrsStd_DriverTable`: the presentation drivers by GUID.
///
/// ```
/// use rustycad_tkvcaf::tprs_std::{TPrsStdDriverTable, TPrsStdAISPresentation};
/// use rustycad_tkcaf::tdata_xtd::TDataXtdAxis;
/// let table = TPrsStdDriverTable::get();
/// let mut driver = Default::default();
/// assert!(table.find_driver(&TDataXtdAxis::get_id(), &mut driver));
/// ```
#[derive(Default)]
pub struct TPrsStdDriverTable {
    transient: StandardTransient,
    drivers: SharedState<PresentationDriverMap>,
}
impl_transient!(
    TPrsStdDriverTable,
    "TPrsStd_DriverTable",
    StandardTransient,
    transient
);

/// The file-static handle of `TPrsStd_DriverTable::Get` (never destroyed natively).
static PRESENTATION_DRIVER_TABLE: OnceLock<Handle<TPrsStdDriverTable>> = OnceLock::new();

impl TPrsStdDriverTable {
    /// `Get`: the process-wide table, created with the standard drivers on first use.
    pub fn get() -> Handle<TPrsStdDriverTable> {
        PRESENTATION_DRIVER_TABLE
            .get_or_init(|| Handle::new(Self::new()))
            .clone()
    }
    /// `TPrsStd_DriverTable()`: a table filled with the standard drivers.
    pub fn new() -> Self {
        let table = Self::default();
        table.init_standard_drivers();
        table
    }
    /// `InitStandardDrivers`: binds the six standard drivers, unless the table holds any
    /// driver already.
    pub fn init_standard_drivers(&self) {
        let mut drivers = self.drivers.write();
        if drivers.extent() > 0 {
            return;
        }
        let standard: [(Guid, Handle<dyn TPrsStdDriver>); 6] = [
            (
                TDataXtdAxis::get_id(),
                Handle::new(TPrsStdAxisDriver::new()).cast(),
            ),
            (
                TDataXtdConstraint::get_id(),
                Handle::new(TPrsStdConstraintDriver::new()).cast(),
            ),
            (
                TDataXtdGeometry::get_id(),
                Handle::new(TPrsStdGeometryDriver::new()).cast(),
            ),
            (
                TNamingNamedShape::get_id(),
                Handle::new(TPrsStdNamedShapeDriver::new()).cast(),
            ),
            (
                TDataXtdPlane::get_id(),
                Handle::new(TPrsStdPlaneDriver::new()).cast(),
            ),
            (
                TDataXtdPoint::get_id(),
                Handle::new(TPrsStdPointDriver::new()).cast(),
            ),
        ];
        for (guid, driver) in standard {
            drivers.bind(guid, driver);
        }
    }
    /// `AddDriver(guid, driver)`: false when the GUID is bound already.
    pub fn add_driver(&self, guid: &Guid, driver: &Handle<dyn TPrsStdDriver>) -> bool {
        self.drivers.write().bind(*guid, driver.clone())
    }
    /// `FindDriver(guid, driver)`: true and the driver when the GUID is bound.
    pub fn find_driver(&self, guid: &Guid, driver: &mut Handle<dyn TPrsStdDriver>) -> bool {
        let found = self.drivers.read().find(guid).ok().cloned();
        let Some(found) = found else {
            return false;
        };
        *driver = found;
        true
    }
    /// `RemoveDriver(guid)`: true when the GUID was bound.
    pub fn remove_driver(&self, guid: &Guid) -> bool {
        self.drivers.write().unbind(guid)
    }
    /// `Clear`: removes every driver (`InitStandardDrivers` fills the table again).
    pub fn clear(&self) {
        self.drivers.write().clear(false);
    }
}
