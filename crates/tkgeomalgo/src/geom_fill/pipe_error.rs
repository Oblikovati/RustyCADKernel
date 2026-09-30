// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_PipeError.hxx and
// GeomFill_Trihedron.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status of a sweep law (`GeomFill_PipeError`) and the kinds of trihedron law
//! a pipe can build (`GeomFill_Trihedron`).

/// The status of a location or trihedron law and of `GeomFill_Pipe`
/// (`GeomFill_PipeError`); `GeomFillPipeError::PipeOk as i32 == 0`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum GeomFillPipeError {
    /// `GeomFill_PipeOk`: the law is valid.
    #[default]
    PipeOk,
    /// `GeomFill_PipeNotOk`: the computation failed.
    PipeNotOk,
    /// `GeomFill_PlaneNotIntersectGuide`: a normal plane of the path misses the guide.
    PlaneNotIntersectGuide,
    /// `GeomFill_ImpossibleContact`: the section cannot touch the guide.
    ImpossibleContact,
}

/// The trihedron law of a pipe (`GeomFill_Trihedron`);
/// `GeomFillTrihedron::IsFrenet as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum GeomFillTrihedron {
    /// `GeomFill_IsCorrectedFrenet`.
    #[default]
    IsCorrectedFrenet,
    /// `GeomFill_IsFixed`.
    IsFixed,
    /// `GeomFill_IsFrenet`.
    IsFrenet,
    /// `GeomFill_IsConstantNormal`.
    IsConstantNormal,
    /// `GeomFill_IsDarboux`.
    IsDarboux,
    /// `GeomFill_IsGuideAC`.
    IsGuideAC,
    /// `GeomFill_IsGuidePlan`.
    IsGuidePlan,
    /// `GeomFill_IsGuideACWithContact`.
    IsGuideACWithContact,
    /// `GeomFill_IsGuidePlanWithContact`.
    IsGuidePlanWithContact,
    /// `GeomFill_IsDiscreteTrihedron`.
    IsDiscreteTrihedron,
}
