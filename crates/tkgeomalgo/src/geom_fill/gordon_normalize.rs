// Rust translation of GeomFill_GordonUtilities.pxx (normalizeContactTable,
// isMonotoneLayout, openClosedBoundaryCandidates) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The validation of an ordered contact table: boundary snaps, the closed-seam
//! detection and the monotone layout check.
use super::gordon_contacts::{adaptor_d0, curve_bounds, snapped_parameter};
use super::gordon_params::{ParamTable, parameter_resolution};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::gp::Point3;

/// The ordered network: the curves and the (profile, guide) parameter tables.
pub(crate) struct NetworkTables<'n> {
    pub(crate) profiles: &'n [Handle<GeomBSplineCurve>],
    pub(crate) guides: &'n [Handle<GeomBSplineCurve>],
    pub(crate) profile_params: &'n mut ParamTable,
    pub(crate) guide_params: &'n mut ParamTable,
}

// Every table entry snapped to the ends of its curve.
fn snap_table(net: &mut NetworkTables<'_>, the_tolerance: f64) -> Result<(), Failure> {
    for (a_profile_idx, a_profile) in net.profiles.iter().enumerate() {
        for (a_guide_idx, a_guide) in net.guides.iter().enumerate() {
            let p: f64 = net.profile_params.at(a_profile_idx, a_guide_idx);
            let snapped: f64 = snapped_parameter(a_profile, p, the_tolerance)?;
            net.profile_params.set(a_profile_idx, a_guide_idx, snapped);
            let g: f64 = net.guide_params.at(a_profile_idx, a_guide_idx);
            let snapped: f64 = snapped_parameter(a_guide, g, the_tolerance)?;
            net.guide_params.set(a_profile_idx, a_guide_idx, snapped);
        }
    }
    Ok(())
}

// One pass over the table: None when a contact is farther than `a_tol`, otherwise
// the largest distances across the two possible seams `(U gap, V gap)`.
fn seam_gaps(net: &NetworkTables<'_>, a_tol: f64) -> Result<Option<(f64, f64)>, Failure> {
    let (mut a_u_gap, mut a_v_gap) = (0.0_f64, 0.0_f64);
    let (a_last_profile, a_last_guide) = (net.profiles.len() - 1, net.guides.len() - 1);
    for (pi, a_profile) in net.profiles.iter().enumerate() {
        for (gi, a_guide) in net.guides.iter().enumerate() {
            let a_profile_point: Point3 = adaptor_d0(a_profile, net.profile_params.at(pi, gi))?;
            let a_guide_point: Point3 = adaptor_d0(a_guide, net.guide_params.at(pi, gi))?;
            if a_profile_point.distance(&a_guide_point) > a_tol {
                return Ok(None);
            }
            if gi == 0 || gi == a_last_guide {
                let opposite = adaptor_d0(a_profile, net.profile_params.at(pi, a_last_guide - gi))?;
                a_u_gap = a_u_gap.max(a_profile_point.distance(&opposite));
            }
            if pi == 0 || pi == a_last_profile {
                let opposite = adaptor_d0(a_guide, net.guide_params.at(a_last_profile - pi, gi))?;
                a_v_gap = a_v_gap.max(a_guide_point.distance(&opposite));
            }
        }
    }
    Ok(Some((a_u_gap, a_v_gap)))
}

/// `normalizeContactTable`: the entries snapped to the curve ends, every contact
/// checked within the tolerance (at least Precision::Confusion()), and a direction
/// closed when its seam gaps are within it; the seam columns (rows) of a closed
/// direction are set to the curve ends. Returns `None` for a contact too far,
/// otherwise the `(U closed, V closed)` flags.
pub(crate) fn normalize_contact_table(
    net: &mut NetworkTables<'_>,
    the_tolerance: f64,
) -> Result<Option<(bool, bool)>, Failure> {
    let a_tol: f64 = the_tolerance.max(precision::confusion());
    // Boundary snaps are done before validation so closed seams use exact end parameters.
    snap_table(net, the_tolerance)?;
    let Some((a_u_gap, a_v_gap)) = seam_gaps(net, a_tol)? else {
        return Ok(None);
    };
    let (the_is_u_closed, the_is_v_closed) = (a_u_gap <= a_tol, a_v_gap <= a_tol);
    if the_is_u_closed {
        open_profile_ends(net)?;
    }
    if the_is_v_closed {
        open_guide_ends(net)?;
    }
    Ok(Some((the_is_u_closed, the_is_v_closed)))
}

// The first and last guide columns of the profile table set to the profile ends.
fn open_profile_ends(net: &mut NetworkTables<'_>) -> Result<(), Failure> {
    let a_last_guide: usize = net.guides.len() - 1;
    for (pi, a_profile) in net.profiles.iter().enumerate() {
        let (first, last) = curve_bounds(a_profile)?;
        net.profile_params.set(pi, 0, first);
        net.profile_params.set(pi, a_last_guide, last);
    }
    Ok(())
}

// The first and last profile rows of the guide table set to the guide ends.
fn open_guide_ends(net: &mut NetworkTables<'_>) -> Result<(), Failure> {
    let a_last_prof: usize = net.profiles.len() - 1;
    for (gi, a_guide) in net.guides.iter().enumerate() {
        let (first, last) = curve_bounds(a_guide)?;
        net.guide_params.set(0, gi, first);
        net.guide_params.set(a_last_prof, gi, last);
    }
    Ok(())
}

/// `isMonotoneLayout`: every profile row increases along the guides and every guide
/// column along the profiles, by more than parameterResolution(0, 1).
pub(crate) fn is_monotone_layout(
    the_profile_params: &ParamTable,
    the_guide_params: &ParamTable,
) -> bool {
    let a_tol: f64 = parameter_resolution(0.0, 1.0);
    for row in 0..the_profile_params.rows {
        for col in 1..the_profile_params.cols {
            if the_profile_params.at(row, col) <= the_profile_params.at(row, col - 1) + a_tol {
                return false;
            }
        }
    }
    for col in 0..the_guide_params.cols {
        for row in 1..the_guide_params.rows {
            if the_guide_params.at(row, col) <= the_guide_params.at(row - 1, col) + a_tol {
                return false;
            }
        }
    }
    true
}

/// `openClosedBoundaryCandidates`: when every profile meets itself between its first
/// and last contacts, those contacts become the profile ends; the same for the guides.
pub(crate) fn open_closed_boundary_candidates(
    net: &mut NetworkTables<'_>,
    the_tolerance: f64,
) -> Result<(), Failure> {
    let a_tol: f64 = the_tolerance.max(precision::confusion());
    let (a_last_guide, a_last_prof) = (net.guides.len() - 1, net.profiles.len() - 1);
    let mut is_u_repeated: bool = true;
    for (pi, a_profile) in net.profiles.iter().enumerate() {
        let a_first = adaptor_d0(a_profile, net.profile_params.at(pi, 0))?;
        let a_last = adaptor_d0(a_profile, net.profile_params.at(pi, a_last_guide))?;
        if a_first.distance(&a_last) > a_tol {
            is_u_repeated = false;
            break;
        }
    }
    if is_u_repeated {
        open_profile_ends(net)?;
    }
    let mut is_v_repeated: bool = true;
    for (gi, a_guide) in net.guides.iter().enumerate() {
        let a_first = adaptor_d0(a_guide, net.guide_params.at(0, gi))?;
        let a_last = adaptor_d0(a_guide, net.guide_params.at(a_last_prof, gi))?;
        if a_first.distance(&a_last) > a_tol {
            is_v_repeated = false;
            break;
        }
    }
    if is_v_repeated {
        open_guide_ends(net)?;
    }
    Ok(())
}
