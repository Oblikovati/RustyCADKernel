//! The native ParametersCollection template contract, implemented for both callers.
use crate::imesh_data::ParametersListArrayAdaptor;
use rustycad_tkernel::{
    ncollection::HArray1OfReal,
    standard::{Failure, Handle},
};
/// Parameter source retaining its original bounds; `collection.value(collection.lower())?`.
pub trait EdgeParameterCollection: Send + Sync {
    /// First parameter index; `collection.lower()`.
    fn lower(&self) -> i32;
    /// Last parameter index; `collection.upper()`.
    fn upper(&self) -> i32;
    /// Shared parameter access; `collection.value(index)?`.
    fn value(&self, index: i32) -> Result<f64, Failure>;
}
impl EdgeParameterCollection for Handle<HArray1OfReal> {
    fn lower(&self) -> i32 {
        self.get()
            .expect("null parameters; expected live array")
            .lower()
    }
    fn upper(&self) -> i32 {
        self.get()
            .expect("null parameters; expected live array")
            .upper()
    }
    fn value(&self, index: i32) -> Result<f64, Failure> {
        self.get()
            .ok_or_else(super::uninitialized_parameters)?
            .value(index)
    }
}
impl EdgeParameterCollection for Handle<ParametersListArrayAdaptor> {
    fn lower(&self) -> i32 {
        self.get()
            .expect("null parameters; expected live adaptor")
            .lower()
    }
    fn upper(&self) -> i32 {
        self.get()
            .expect("null parameters; expected live adaptor")
            .upper()
    }
    fn value(&self, index: i32) -> Result<f64, Failure> {
        self.get()
            .ok_or_else(super::uninitialized_parameters)?
            .value(index)
    }
}
