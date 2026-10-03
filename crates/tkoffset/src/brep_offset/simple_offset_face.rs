// Translation of BRepOffset_SimpleOffset::FillFaceData at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BRepOffsetStatus, collapse_singularities, surface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// New surface, location, tolerance and reversal flags for one face in simple offset.
#[derive(Clone)]
pub struct SimpleOffsetFaceData {
    /// The surface created from the transformed source face.
    pub offset_surface: Handle<dyn GeomSurface>,
    /// The new face location; FillFaceData stores an identity location.
    pub location: Location,
    /// The source face tolerance retained by FillFaceData.
    pub tolerance: f64,
    /// FillFaceData does not reverse wires.
    pub reverse_wires: bool,
    /// FillFaceData does not reverse the face.
    pub reverse_face: bool,
    /// Status returned by BRepOffset::Surface; the native FillFaceData ignores it.
    pub status: BRepOffsetStatus,
}

/// Runs the private `FillFaceData` operation for a face, including orientation sign,
/// singularity preprocessing and `BRepOffset::Surface(..., allowC0 = true)`.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkoffset::brep_offset::build_simple_offset_face_data;
/// let data = build_simple_offset_face_data(&Face::default(), 1.0, 1.0e-7);
/// assert!(data.is_err());
/// ```
pub fn build_simple_offset_face_data(
    face: &Face,
    offset: f64,
    singularity_tolerance: f64,
) -> Result<SimpleOffsetFaceData, Failure> {
    let tolerance = BRepTool::face_tolerance(face)?;
    let source = BRepTool::surface(face)?;
    let prepared = collapse_singularities(&source, face, singularity_tolerance)?;
    let signed_offset = signed_face_offset(face, offset);
    let (offset_surface, status) = surface(&prepared, signed_offset, true)?;
    Ok(simple_offset_face_result(offset_surface, tolerance, status))
}

fn signed_face_offset(face: &Face, offset: f64) -> f64 {
    if face.orientation() == Orientation::Reversed {
        -offset
    } else {
        offset
    }
}

fn simple_offset_face_result(
    offset_surface: Handle<dyn GeomSurface>,
    tolerance: f64,
    status: BRepOffsetStatus,
) -> SimpleOffsetFaceData {
    SimpleOffsetFaceData {
        offset_surface,
        location: Location::default(),
        tolerance,
        reverse_wires: false,
        reverse_face: false,
        status,
    }
}
