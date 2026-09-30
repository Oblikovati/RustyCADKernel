// Rust translation of the Standard_NotImplemented members of ModelingData/TKG3d/GeomEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The failure every GeomEval curve and surface raises from the members that would
//! change or reparameterize it (`Reverse`, `UReverse`, `ReversedParameter`,
//! `Transform`, `UIso`...).
use crate::geom::errors;
use rustycad_tkernel::standard::Failure;

/// Standard_NotImplemented of a refused member, named as natively (`Class::Member`),
/// with the offending arguments in `detail`;
/// `unsupported("GeomEval_SineWaveCurve::ReversedParameter", "(U = 0.5)")`.
pub(crate) fn unsupported(member: &str, detail: &str) -> Failure {
    errors::not_implemented(&format!(
        "{member}{detail}: GeomEval geometry keeps its defining parameterization; expected no reversal, transformation or iso extraction"
    ))
}
