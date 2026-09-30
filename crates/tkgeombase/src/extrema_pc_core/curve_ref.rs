//! The adaptor held by the offset and general-curve evaluators.
//!
//! Natively they keep a `const Adaptor_Curve*` to an adaptor that must outlive them;
//! `ExtremaPC(2d)_Curve` points it at an adaptor it owns (`myAdaptorOwned`) when built
//! from a Geom curve. The Rust evaluators hold either the borrowed adaptor or a handle
//! sharing the owned one, so no evaluator refers into its own owner.
use super::space::PcSpace;
use rustycad_tkernel::standard::Handle;

/// A borrowed or shared curve adaptor; `PcCurveRef::Borrowed(&adaptor)`.
pub enum PcCurveRef<'a, S: PcSpace> {
    /// An adaptor owned by the caller, which must outlive the evaluator.
    Borrowed(&'a S::Curve),
    /// An adaptor shared through a handle (the dispatcher's owned adaptor).
    Shared(Handle<S::Curve>),
}

impl<S: PcSpace> PcCurveRef<'_, S> {
    /// The adaptor, `None` for a null shared handle; `curve_ref.get()`.
    pub fn get(&self) -> Option<&S::Curve> {
        match self {
            Self::Borrowed(curve) => Some(*curve),
            Self::Shared(handle) => handle.get(),
        }
    }
}
