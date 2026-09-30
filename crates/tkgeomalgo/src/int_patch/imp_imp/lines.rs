// Rust translation of the `occ::down_cast<IntPatch_ALine/GLine/RLine>(lin)->...` vertex
// edits repeated through ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex members of the analytic, conic and restriction lines, reached from a
//! `Handle<IntPatch_Line>` by its arc type.
use super::super::IntPatchIType;
use super::super::aline::IntPatchALine;
use super::super::failures::null_object;
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLine;
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use rustycad_tkernel::standard::{Failure, Handle};

/// A line of the result whose vertices the algorithm edits.
pub(super) enum VertexLine {
    /// `IntPatch_ALine`.
    Analytic(Handle<IntPatchALine>),
    /// `IntPatch_GLine`.
    Conic(Handle<IntPatchGLine>),
    /// `IntPatch_RLine`.
    Restriction(Handle<IntPatchRLine>),
}

/// The line as the class `T`, or the Standard_NullObject of the null handle that the
/// native `occ::down_cast` returns for another class (then dereferenced).
fn cast_to<T: IntPatchLine + rustycad_tkernel::standard::RuntimeTyped>(
    line: &Handle<dyn IntPatchLine>,
    class: &str,
) -> Result<Handle<T>, Failure> {
    let cast: Handle<T> = line.cast();
    if cast.is_null() {
        return Err(null_object("IntPatch_ImpImpIntersection", class));
    }
    Ok(cast)
}

impl VertexLine {
    /// `TheType == IntPatch_Analytic ? ALine : GLine`: the two-way down cast of
    /// PutPointsOnLine, MultiplePoint, PointOnSecondDom and ProcessRLine.
    pub(super) fn analytic_or_conic(line: &Handle<dyn IntPatchLine>) -> Result<Self, Failure> {
        if line.arc_type() == IntPatchIType::Analytic {
            return Ok(Self::Analytic(cast_to(line, "IntPatch_ALine")?));
        }
        Ok(Self::Conic(cast_to(line, "IntPatch_GLine")?))
    }

    /// The three-way down cast of ProcessSegments (analytic, restriction, conic).
    pub(super) fn of(line: &Handle<dyn IntPatchLine>) -> Result<Self, Failure> {
        if line.arc_type() == IntPatchIType::Restriction {
            return Ok(Self::Restriction(cast_to(line, "IntPatch_RLine")?));
        }
        Self::analytic_or_conic(line)
    }

    /// `NbVertex()`.
    pub(super) fn nb_vertex(&self) -> i32 {
        match self {
            Self::Analytic(l) => l.nb_vertex(),
            Self::Conic(l) => l.nb_vertex(),
            Self::Restriction(l) => l.nb_vertex(),
        }
    }

    /// `Vertex(Index)`, a copy.
    pub(super) fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        match self {
            Self::Analytic(l) => l.vertex(index),
            Self::Conic(l) => l.vertex(index),
            Self::Restriction(l) => l.vertex(index),
        }
    }

    /// `Replace(Index, Pnt)` (the conic line appends the point, as natively).
    pub(super) fn replace(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        match self {
            Self::Analytic(l) => l.replace(index, pnt),
            Self::Conic(l) => {
                l.replace(index, pnt);
                Ok(())
            }
            Self::Restriction(l) => l.replace(index, pnt),
        }
    }

    /// `AddVertex(Pnt)`.
    pub(super) fn add_vertex(&self, pnt: &IntPatchPoint) -> Result<(), Failure> {
        match self {
            Self::Analytic(l) => {
                l.add_vertex(pnt);
                Ok(())
            }
            Self::Conic(l) => l.add_vertex(pnt),
            Self::Restriction(l) => {
                l.add_vertex(pnt, false);
                Ok(())
            }
        }
    }
}
