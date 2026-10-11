//! A named fake that rejects abstract overlap calls, exposing only tested base defaults.
use rustycad_tkernel::standard::StandardTransient;
use rustycad_tkv3d::select_mgr::*;

use super::abstract_intersector;

#[derive(Default)]
pub struct BaseDefaultsIntersector {
    transient: StandardTransient,
    base: SelectMgrBaseIntersectorBase,
}
rustycad_tkernel::impl_transient!(BaseDefaultsIntersector, "BaseDefaultsIntersector", StandardTransient, transient,
    interfaces: [dyn SelectMgrBaseIntersector]);

impl SelectMgrBaseIntersector for BaseDefaultsIntersector {
    fn intersector_base(&self) -> &SelectMgrBaseIntersectorBase {
        &self.base
    }
    abstract_intersector::reject_abstract_geometry!();
}
