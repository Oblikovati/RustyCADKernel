// Rust translation of the static PrintRegularity and ReadRegularity and of the
// representation accessors shared by the writers of
// ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The regularity codes of the ASCII format and the representation accessors shared
//! by the dump and write members.
use crate::brep::update_curves::as_gcurve;
use crate::brep::{BRepCurveOnClosedSurface, BRepCurveOnSurface, BRepCurveRepresentation};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;

/// `char buffer[255]` of `ReadRegularity`.
const REGULARITY_BUFFER: usize = 255;

/// `PrintRegularity(C, OS)`: `C0`, `G1`, `C1`, `G2`, `C2`, `C3` or `CN`.
pub(super) fn print_regularity(continuity: Continuity, stream: &mut IoStream) {
    let text: &[u8] = match continuity {
        Continuity::C0 => b"C0",
        Continuity::G1 => b"G1",
        Continuity::C1 => b"C1",
        Continuity::G2 => b"G2",
        Continuity::C2 => b"C2",
        Continuity::C3 => b"C3",
        Continuity::CN => b"CN",
    };
    stream.insert_bytes(text);
}

/// `ReadRegularity(IS)`: one word whose first two characters name a continuity; any
/// other word reads C0. The native buffer keeps its previous bytes when no word is
/// extracted; here that reads C0 as well.
pub(super) fn read_regularity(stream: &mut IoStream) -> Continuity {
    let word = stream.extract_word(REGULARITY_BUFFER);
    let first = word.first().copied().unwrap_or(0);
    let second = word.get(1).copied().unwrap_or(0);
    match (first, second) {
        (b'C', b'0') => Continuity::C0,
        (b'C', b'1') => Continuity::C1,
        (b'C', b'2') => Continuity::C2,
        (b'C', b'3') => Continuity::C3,
        (b'C', b'N') => Continuity::CN,
        (b'G', b'1') => Continuity::G1,
        (b'G', b'2') => Continuity::G2,
        _ => Continuity::C0,
    }
}

/// `GC->Range(first, last)` of a curve representation that is a BRep_GCurve.
pub(super) fn gcurve_range(
    representation: &Handle<dyn BRepCurveRepresentation>,
) -> Result<(f64, f64), Failure> {
    let gcurve = as_gcurve(representation);
    let Some(gcurve) = gcurve.get() else {
        return Err(crate::brep::errors::null_object(
            "BRepTools_ShapeSet: a curve representation is not a BRep_GCurve",
        ));
    };
    Ok(gcurve.range())
}

/// `UVPoints` of the p-curve representation (the first p-curve of a seam).
pub(super) fn uv_points(representation: &Handle<dyn BRepCurveRepresentation>) -> (Point2, Point2) {
    let seam = representation.cast::<BRepCurveOnClosedSurface>();
    if let Some(seam) = seam.get() {
        return seam.as_curve_on_surface().uv_points();
    }
    let pcurve = representation.cast::<BRepCurveOnSurface>();
    pcurve.get().map(|p| p.uv_points()).unwrap_or_default()
}

/// `UVPoints2` of a seam representation.
pub(super) fn uv_points2(representation: &Handle<dyn BRepCurveRepresentation>) -> (Point2, Point2) {
    let seam = representation.cast::<BRepCurveOnClosedSurface>();
    seam.get().map(|seam| seam.uv_points2()).unwrap_or_default()
}
