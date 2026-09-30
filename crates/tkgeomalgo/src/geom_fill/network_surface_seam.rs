// Rust translation of GeomFill_NetworkSurface.cxx (makeProfileSeamGuide,
// completeClosedGuideSeam, makeNetworkSurface, canSetPeriodic, applyPeriodicity,
// reparametrizeSurface) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The closed U seam of a network, the construction of its surface and the final
//! periodicity and reparametrization.
use super::gordon_params::null_curve;
use super::network_surface::{
    ClosedSeams, NetworkSurfaceStatus, PreparedNetwork, has_rational_curve, prepare_curve_family,
};
use super::network_surface_align::align_surfaces;
use super::network_surface_rational::make_corrected_profile_skin;
use super::network_surface_skin::{
    SkinFamily, SkinningBasis, array, at, grid, make_guide_skin, make_profile_skin,
    make_reference_surface, pole_weight,
};
use crate::geom_api::Interpolate;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// `makeProfileSeamGuide`: the interpolation (GeomAPI_Interpolate, not periodic,
/// tolerance Precision::Confusion()) of the start points of the profiles on the
/// profile parameters; None when it is not done.
fn make_profile_seam_guide(
    n: &PreparedNetwork,
) -> Result<Option<Handle<GeomBSplineCurve>>, Failure> {
    let mut points: Array1<'static, Point3> = array(n.profiles.len())?;
    let mut parameters: Array1<'static, f64> = array(n.profile_parameters.len())?;
    for (i, a_profile) in n.profiles.iter().enumerate() {
        let profile = a_profile.get().ok_or_else(null_curve)?;
        points[i as i32 + 1] = profile.value(profile.first_parameter())?;
        parameters[i as i32 + 1] = at(&n.profile_parameters, i);
    }
    let points = Handle::new(HArray1::from_owned(points));
    let parameters = Handle::new(HArray1::from_owned(parameters));
    let mut an_interpolator =
        Interpolate::with_parameters(&points, &parameters, false, precision::confusion())?;
    an_interpolator.perform()?;
    if !an_interpolator.is_done() {
        return Ok(None);
    }
    Ok(Some(an_interpolator.curve()?))
}

// The profile range and whether the guides miss its start and its end.
fn seam_needs(n: &PreparedNetwork) -> Result<((f64, f64), bool, bool), Failure> {
    let profile = n.profiles[0].get().ok_or_else(null_curve)?;
    let (start, end) = (profile.first_parameter(), profile.last_parameter());
    let first: f64 = n.guide_parameters[n.guide_parameters.lower()];
    let last: f64 = n.guide_parameters[n.guide_parameters.upper()];
    Ok((
        (start, end),
        first > start + precision::p_confusion(),
        last < end - precision::p_confusion(),
    ))
}

/// `completeClosedGuideSeam`: the guides completed with a seam guide at the profile
/// start (the interpolation of the profile starts) and a repeat of the first guide at
/// the profile end, with their parameters and contact rows; None when the seam guide
/// cannot be interpolated. The caller runs it only when a seam is missing.
fn complete_closed_guide_seam(n: &PreparedNetwork) -> Result<Option<PreparedNetwork>, Failure> {
    let ((start, end), needs_start, needs_end) = seam_needs(n)?;
    let seam: Option<Handle<GeomBSplineCurve>> = if needs_start {
        let Some(seam) = make_profile_seam_guide(n)? else {
            return Ok(None);
        };
        Some(seam)
    } else {
        None
    };
    let offset: usize = usize::from(needs_start);
    let nb_guides: usize = n.guides.len() + offset + usize::from(needs_end);
    let mut guides: Vec<Handle<GeomBSplineCurve>> = Vec::with_capacity(nb_guides);
    let mut parameters: Array1<'static, f64> = array(nb_guides)?;
    if let Some(seam) = &seam {
        guides.push(seam.clone());
        parameters[1] = start;
    }
    for (gi, guide) in n.guides.iter().enumerate() {
        guides.push(guide.clone());
        parameters[(gi + offset) as i32 + 1] = at(&n.guide_parameters, gi);
    }
    if needs_end {
        guides.push(seam.clone().unwrap_or_else(|| n.guides[0].clone()));
        parameters[nb_guides as i32] = end;
    }
    let (points, weights) = closed_contact_rows(n, (nb_guides, offset), (needs_start, needs_end))?;
    Ok(Some(PreparedNetwork {
        profiles: n.profiles.clone(),
        guides,
        profile_parameters: n.profile_parameters.clone(),
        guide_parameters: parameters,
        intersection_points: points,
        intersection_weights: weights,
    }))
}

// The contact rows of the completed guides: the profile starts (weight of the first
// pole) for a start seam, the input rows, then row 0 again for an end seam.
fn closed_contact_rows(
    n: &PreparedNetwork,
    (nb_guides, offset): (usize, usize),
    (needs_start, needs_end): (bool, bool),
) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
    let (input_points, input_weights) = (&n.intersection_points, &n.intersection_weights);
    let nb_cols: i32 = input_points.nb_columns() as i32;
    let mut points: Array2<'static, Point3> = grid(nb_guides as i32, nb_cols)?;
    let mut weights: Array2<'static, f64> =
        grid(nb_guides as i32, input_weights.nb_columns() as i32)?;
    if needs_start {
        for pi in 0..nb_cols {
            let profile = n.profiles[pi as usize].get().ok_or_else(null_curve)?;
            points[(1, pi + 1)] = profile.value(profile.first_parameter())?;
            weights[(1, pi + 1)] = pole_weight(profile, 1)?;
        }
    }
    let (row0, col0) = (input_points.lower_row(), input_points.lower_col());
    let (wrow0, wcol0) = (input_weights.lower_row(), input_weights.lower_col());
    for gi in 0..input_points.nb_rows() as i32 {
        for pi in 0..nb_cols {
            let row: i32 = gi + offset as i32 + 1;
            points[(row, pi + 1)] = input_points[(row0 + gi, col0 + pi)];
            weights[(row, pi + 1)] = input_weights[(wrow0 + gi, wcol0 + pi)];
        }
    }
    if needs_end {
        for pi in 1..=nb_cols {
            points[(nb_guides as i32, pi)] = points[(1, pi)];
            weights[(nb_guides as i32, pi)] = weights[(1, pi)];
        }
    }
    Ok((points, weights))
}

/// `makeNetworkSurface`: the U seam completed when the network is closed in U and the
/// guides miss a profile end, the profile and guide skins, the reference surface of
/// the contact grid, their alignment and the corrected profile skin. Returns the
/// surface, or None with the failing status.
pub(crate) fn make_network_surface(
    n: &PreparedNetwork,
    the_is_u_closed: bool,
) -> Result<(Option<Handle<GeomBSplineSurface>>, NetworkSurfaceStatus), Failure> {
    let (_, needs_start, needs_end) = seam_needs(n)?;
    let should_complete_u_seam: bool = the_is_u_closed && (needs_start || needs_end);
    let completed: PreparedNetwork;
    let network: &PreparedNetwork = if should_complete_u_seam {
        let Some(mut closed) = complete_closed_guide_seam(n)? else {
            return Ok((None, NetworkSurfaceStatus::SkinningFailed));
        };
        if !prepare_curve_family(&mut closed.guides)? {
            return Ok((None, NetworkSurfaceStatus::CurveCompatibilityFailed));
        }
        completed = closed;
        &completed
    } else {
        n
    };
    let u_basis = SkinningBasis::init(&network.guide_parameters)?;
    let v_basis = SkinningBasis::init(&n.profile_parameters)?;
    build_from_bases(n, network, (&u_basis, &v_basis))
}

// The skins, the reference, the alignment and the corrected skin on both bases.
fn build_from_bases(
    n: &PreparedNetwork,
    network: &PreparedNetwork,
    (u_basis, v_basis): (&SkinningBasis, &SkinningBasis),
) -> Result<(Option<Handle<GeomBSplineSurface>>, NetworkSurfaceStatus), Failure> {
    let profiles = SkinFamily {
        curves: &n.profiles,
        parameters: &n.profile_parameters,
        basis: v_basis,
    };
    let Some(profile_surface) = make_profile_skin(&profiles)? else {
        return Ok((None, NetworkSurfaceStatus::SkinningFailed));
    };
    let guides = SkinFamily {
        curves: &network.guides,
        parameters: &network.guide_parameters,
        basis: u_basis,
    };
    let Some(guide_surface) = make_guide_skin(&guides)? else {
        return Ok((None, NetworkSurfaceStatus::SkinningFailed));
    };
    // The reference surface starts from the same contact grid validated during preparation.
    let is_reference_polynomial: bool =
        has_rational_curve(&n.profiles) != has_rational_curve(&network.guides);
    let grid_data = (&network.intersection_points, &network.intersection_weights);
    let parameters = (&network.guide_parameters, &n.profile_parameters);
    let Some(reference) = make_reference_surface(
        grid_data,
        (u_basis, v_basis),
        parameters,
        is_reference_polynomial,
    )?
    else {
        return Ok((None, NetworkSurfaceStatus::ReferenceSurfaceFailed));
    };
    if !align_surfaces([&profile_surface, &guide_surface, &reference])? {
        return Ok((None, NetworkSurfaceStatus::KnotAlignmentFailed));
    }
    let null = || {
        Failure::with_type(
            "Standard_NullObject",
            "GeomFill_NetworkSurface - null skin",
            "",
        )
    };
    let skins = [
        profile_surface.get().ok_or_else(null)?,
        guide_surface.get().ok_or_else(null)?,
        reference.get().ok_or_else(null)?,
    ];
    match make_corrected_profile_skin(skins)? {
        Ok(surface) => Ok((Some(surface), NetworkSurfaceStatus::Done)),
        Err(status) => Ok((None, status)),
    }
}

/// `canSetPeriodic`: the first and last pole rows (columns) coincide within the
/// tolerance, across U (`the_along_u`) or V.
fn can_set_periodic(
    the_surface: &GeomBSplineSurface,
    the_along_u: bool,
    the_tolerance: f64,
) -> Result<bool, Failure> {
    let (nu, nv) = (the_surface.nb_u_poles(), the_surface.nb_v_poles());
    let a_nb_checks: i32 = if the_along_u { nv } else { nu };
    for k in 1..=a_nb_checks {
        let (first, last) = if the_along_u {
            (the_surface.pole(1, k)?, the_surface.pole(nu, k)?)
        } else {
            (the_surface.pole(k, 1)?, the_surface.pole(k, nv)?)
        };
        if first.distance(&last) > the_tolerance {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `applyPeriodicity`: each closed direction made periodic when its end poles coincide
/// within the tolerance (at least Precision::Confusion()); false otherwise.
pub(crate) fn apply_periodicity(
    the_surface: &Handle<GeomBSplineSurface>,
    (the_is_u_closed, the_is_v_closed): ClosedSeams,
    the_tolerance: f64,
) -> Result<bool, Failure> {
    let surface = the_surface.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "GeomFill_NetworkSurface - null surface",
            "",
        )
    })?;
    let a_tolerance: f64 = the_tolerance.max(precision::confusion());
    if the_is_u_closed {
        if !can_set_periodic(surface, true, a_tolerance)? {
            return Ok(false);
        }
        surface.set_u_periodic()?;
    }
    if the_is_v_closed {
        if !can_set_periodic(surface, false, a_tolerance)? {
            return Ok(false);
        }
        surface.set_v_periodic()?;
    }
    Ok(true)
}

/// `reparametrizeSurface`: both knot vectors mapped onto `[0, 1]`.
pub(crate) fn reparametrize_surface(
    the_surface: &Handle<GeomBSplineSurface>,
) -> Result<(), Failure> {
    let surface = the_surface.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "GeomFill_NetworkSurface - null surface",
            "",
        )
    })?;
    let mut a_u_knots: Array1<'static, f64> = surface.u_knots();
    let mut a_v_knots: Array1<'static, f64> = surface.v_knots();
    bsplclib::reparametrize(0.0, 1.0, &mut a_u_knots)?;
    bsplclib::reparametrize(0.0, 1.0, &mut a_v_knots)?;
    surface.set_u_knots(&a_u_knots)?;
    surface.set_v_knots(&a_v_knots)
}
