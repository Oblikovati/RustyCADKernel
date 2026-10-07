// Rust translation of ChFi3d_ComputeHatchingDomains (ChFi3d_Builder_SpKP.cxx,
// ChFi3d_Builder_0.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The hatching domains of a face of a fillet, kept segments included when the
//! hatching only runs along the boundary.
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom2d_hatch::Hatcher;

/// `ChFi3d_ComputeHatchingDomains(theHatcher, theHatchingIndex)`: the domains of the
/// hatching; when there is none but a segment of the hatching lies on the boundary
/// (longer than the 2d confusion), the domains are computed again with the segments
/// kept. Whether the computation is done.
///
/// ```
/// use rustycad_tkfillet::chfi3d::compute_hatching_domains;
/// use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
/// let mut hatcher = Hatcher::new(&Intersector::new(1e-7, 1e-7), 1e-7, 1e-7, false, false);
/// assert!(compute_hatching_domains(&mut hatcher, 1).is_err());
/// ```
pub fn compute_hatching_domains(
    hatcher: &mut Hatcher,
    hatching_index: i32,
) -> Result<bool, Failure> {
    hatcher.compute_domains_of(hatching_index)?;
    if !hatcher.is_done(hatching_index)? || hatcher.nb_domains(hatching_index)? != 0 {
        return hatcher.is_done(hatching_index);
    }
    if has_boundary_segment(hatcher, hatching_index)? {
        hatcher.set_keep_segments(true);
        hatcher.compute_domains_of(hatching_index)?;
    }
    hatcher.is_done(hatching_index)
}

/// Whether the points of the hatching open and close a segment (outermost pair, the
/// nested ones counted) longer than the 2d confusion.
fn has_boundary_segment(hatcher: &Hatcher, hatching_index: i32) -> Result<bool, Failure> {
    let (mut depth, mut start) = (0, 0.0);
    for point_index in 1..=hatcher.nb_points(hatching_index)? {
        let point = hatcher.point(hatching_index, point_index)?;
        if point.segment_beginning() {
            if depth == 0 {
                start = point.parameter();
            }
            depth += 1;
        }
        if point.segment_end() && depth > 0 {
            depth -= 1;
            if depth == 0 && (point.parameter() - start).abs() > hatcher.confusion2d() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
