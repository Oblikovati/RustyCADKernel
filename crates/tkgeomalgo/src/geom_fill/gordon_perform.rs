// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of GeomFill_Gordon.cxx (Perform with mapNetworkStatus,
// canApproximateAfterExactFailure, setStatus, copyWorkingCurves) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Perform of GeomFill_Gordon.
use super::gordon::{
    GeomFillGordon, GordonApproximationMode, GordonBuildReport, GordonBuildStage, GordonStatus,
};
use super::gordon_fallback::{approximate_prepared_network, max_family_deviation};
use super::gordon_network::NetworkPreparation;
use super::gordon_params::ParamTable;
use super::network_surface::{GeomFillNetworkSurface, NetworkSurfaceStatus};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomGeometry};
use rustycad_tkmath::gp::Point3;

/// `mapNetworkStatus`: the Gordon status of a network-construction status.
fn map_network_status(the_status: NetworkSurfaceStatus) -> GordonStatus {
    match the_status {
        NetworkSurfaceStatus::Done => GordonStatus::Done,
        NetworkSurfaceStatus::InvalidInput => GordonStatus::CompatibilityFailed,
        NetworkSurfaceStatus::CurveCompatibilityFailed => GordonStatus::CurveCompatibilityFailed,
        NetworkSurfaceStatus::SkinningFailed => GordonStatus::SkinningFailed,
        NetworkSurfaceStatus::ReferenceSurfaceFailed => GordonStatus::ReferenceSurfaceFailed,
        NetworkSurfaceStatus::KnotAlignmentFailed => GordonStatus::KnotAlignmentFailed,
        NetworkSurfaceStatus::RationalDegreeOverflow => GordonStatus::RationalDegreeOverflow,
        NetworkSurfaceStatus::RationalConstructionFailed => {
            GordonStatus::RationalConstructionFailed
        }
        NetworkSurfaceStatus::PeriodicityFailed => GordonStatus::PeriodicityFailed,
        NetworkSurfaceStatus::NotStarted | NetworkSurfaceStatus::ConstructionFailed => {
            GordonStatus::ConstructionFailed
        }
    }
}

/// `canApproximateAfterExactFailure`: the exact failures that a sampled fallback of the
/// validated network can recover.
fn can_approximate_after_exact_failure(the_status: GordonStatus) -> bool {
    matches!(
        the_status,
        GordonStatus::CurveCompatibilityFailed
            | GordonStatus::SkinningFailed
            | GordonStatus::ReferenceSurfaceFailed
            | GordonStatus::KnotAlignmentFailed
            | GordonStatus::RationalDegreeOverflow
            | GordonStatus::RationalConstructionFailed
            | GordonStatus::ConstructionFailed
    )
}

/// `setStatus`: the status, and its stage unless done.
fn set_status(
    the_report: &mut GordonBuildReport,
    the_new_status: GordonStatus,
    the_stage: GordonBuildStage,
) {
    the_report.status = the_new_status;
    the_report.failed_stage = if the_new_status == GordonStatus::Done {
        GordonBuildStage::NotStarted
    } else {
        the_stage
    };
}

/// `copyWorkingCurves`: deep copies of the input curves (None for a null curve or an
/// exception, as the native try/catch).
fn copy_working_curves(
    the_input: &[Handle<GeomBSplineCurve>],
) -> Option<Vec<Handle<GeomBSplineCurve>>> {
    let mut working: Vec<Handle<GeomBSplineCurve>> = Vec::with_capacity(the_input.len());
    for a_curve in the_input {
        let copy: Handle<GeomBSplineCurve> = a_curve.get()?.copy().ok()?.cast();
        if copy.is_null() {
            return None;
        }
        working.push(copy);
    }
    Some(working)
}

/// The prepared contact grid of a successful preparation.
struct PreparedGrid {
    points: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
    did_approximate: bool,
}

// A failure recorded in the report: the preparation stops without a grid.
fn stop(
    report: &mut GordonBuildReport,
    status: GordonStatus,
    stage: GordonBuildStage,
) -> Result<Option<PreparedGrid>, Failure> {
    set_status(report, status, stage);
    Ok(None)
}

/// The network preparation of `Perform()`, from the unit conversion to the second
/// normalization; None after a recorded failure.
fn prepare(g: &mut GeomFillGordon) -> Result<Option<PreparedGrid>, Failure> {
    let tolerance: f64 = g.my_tolerance;
    let allow: bool = g.my_approximation_mode == GordonApproximationMode::AllowApproximateFallback;
    let GeomFillGordon {
        my_profiles,
        my_guides,
        my_profile_params,
        my_guide_params,
        my_report,
        my_is_u_closed,
        my_is_v_closed,
        ..
    } = g;
    let mut n = NetworkPreparation::new(
        (my_profiles, my_guides),
        (my_profile_params, my_guide_params),
        tolerance,
    );
    if !n.convert_curves_to_unit_interval()? {
        return stop(
            my_report,
            GordonStatus::ConversionFailed,
            GordonBuildStage::InputConversion,
        );
    }
    if !n.fill_intersection_parameters()? {
        return stop(
            my_report,
            GordonStatus::IntersectionFailed,
            GordonBuildStage::ContactDiscovery,
        );
    }
    my_report.max_contact_gap = n.max_contact_gap();
    if !n.reorder_network()? {
        return stop(
            my_report,
            GordonStatus::OrderingFailed,
            GordonBuildStage::NetworkOrdering,
        );
    }
    let Some((u, v)) = n.normalize_intersection_parameters()? else {
        return stop(
            my_report,
            GordonStatus::CompatibilityFailed,
            GordonBuildStage::NetworkOrdering,
        );
    };
    (*my_is_u_closed, *my_is_v_closed) = (u || n.profiles_closed()?, v);
    let mut did_approximate: bool = false;
    if !n.equalize_intersection_parameters(allow, &mut did_approximate)? {
        let status = if n.has_rational_curve() {
            GordonStatus::RationalReparametrizationFailed
        } else {
            GordonStatus::ReparametrizationFailed
        };
        return stop(my_report, status, GordonBuildStage::Reparametrization);
    }
    my_report.max_reparametrization_deviation = n.max_reparametrization_deviation();
    let Some((u, v)) = n.normalize_intersection_parameters()? else {
        return stop(
            my_report,
            GordonStatus::CompatibilityFailed,
            GordonBuildStage::Reparametrization,
        );
    };
    (*my_is_u_closed, *my_is_v_closed) = (u || n.profiles_closed()?, v);
    Ok(Some(PreparedGrid {
        points: n.intersection_points()?,
        weights: n.intersection_weights()?,
        did_approximate,
    }))
}

/// `Perform()`: the report and the surface reset; InvalidInput for fewer than two
/// profiles or guides; deep copies of the input as working curves; the preparation,
/// the exact construction, and the fallback when allowed.
pub(super) fn perform(g: &mut GeomFillGordon) -> Result<(), Failure> {
    g.my_report = GordonBuildReport::default();
    g.my_surface = Handle::default();
    (g.my_is_u_closed, g.my_is_v_closed) = (false, false);
    if g.my_input_profiles.len() < 2 || g.my_input_guides.len() < 2 {
        set_status(
            &mut g.my_report,
            GordonStatus::InvalidInput,
            GordonBuildStage::InputConversion,
        );
        return Ok(());
    }
    let (nb_p, nb_g) = (g.my_input_profiles.len(), g.my_input_guides.len());
    g.my_profiles = vec![Handle::default(); nb_p];
    g.my_guides = vec![Handle::default(); nb_g];
    let (Some(profiles), Some(guides)) = (
        copy_working_curves(&g.my_input_profiles),
        copy_working_curves(&g.my_input_guides),
    ) else {
        set_status(
            &mut g.my_report,
            GordonStatus::ConversionFailed,
            GordonBuildStage::InputConversion,
        );
        return Ok(());
    };
    (g.my_profiles, g.my_guides) = (profiles, guides);
    g.my_profile_params = ParamTable::new(nb_p, nb_g);
    g.my_guide_params = ParamTable::new(nb_p, nb_g);
    let Some(grid) = prepare(g)? else {
        return Ok(());
    };
    construct(g, &grid)
}

/// The exact construction by GeomFill_NetworkSurface on the averaged parameters, the
/// validation deviations, or the fallback.
fn construct(g: &mut GeomFillGordon, grid: &PreparedGrid) -> Result<(), Failure> {
    let guide_values: Vec<f64> = g.my_profile_params.column_means();
    let profile_values: Vec<f64> = g.my_guide_params.row_means();
    let mut network = GeomFillNetworkSurface::new();
    network.init(
        (&handles(&g.my_profiles)?, &handles(&g.my_guides)?),
        (&one_based(&profile_values)?, &one_based(&guide_values)?),
        (&grid.points, &grid.weights),
        g.my_tolerance,
        (g.my_is_u_closed, g.my_is_v_closed),
    );
    network.perform();
    let values = (&profile_values[..], &guide_values[..]);
    if !network.is_done() {
        return after_exact_failure(
            g,
            map_network_status(network.status()),
            &grid.points,
            values,
        );
    }
    g.my_surface = network.surface()?;
    let r = &mut g.my_report;
    r.max_profile_deviation = max_family_deviation(&g.my_surface, &g.my_profiles, values.0, true)?;
    r.max_guide_deviation = max_family_deviation(&g.my_surface, &g.my_guides, values.1, false)?;
    r.is_approximate = grid.did_approximate;
    r.max_approximation_deviation = if r.is_approximate {
        r.max_profile_deviation.max(r.max_guide_deviation)
    } else {
        0.0
    };
    set_status(r, GordonStatus::Done, GordonBuildStage::Validation);
    Ok(())
}

/// The sampled fallback after an exact failure, when allowed and recoverable.
fn after_exact_failure(
    g: &mut GeomFillGordon,
    an_exact_status: GordonStatus,
    points: &Array2<'static, Point3>,
    (profile_values, guide_values): (&[f64], &[f64]),
) -> Result<(), Failure> {
    let allowed = g.my_approximation_mode == GordonApproximationMode::AllowApproximateFallback;
    if !(allowed && can_approximate_after_exact_failure(an_exact_status)) {
        set_status(
            &mut g.my_report,
            an_exact_status,
            GordonBuildStage::ExactConstruction,
        );
        return Ok(());
    }
    let curves = (&g.my_profiles[..], &g.my_guides[..]);
    let Some(surface) = approximate_prepared_network(curves, points, g.my_tolerance)? else {
        set_status(
            &mut g.my_report,
            GordonStatus::ApproximationFailed,
            GordonBuildStage::Approximation,
        );
        return Ok(());
    };
    g.my_surface = surface;
    g.my_report.is_approximate = true;
    let profile_deviation =
        max_family_deviation(&g.my_surface, &g.my_profiles, profile_values, true)?;
    let guide_deviation = max_family_deviation(&g.my_surface, &g.my_guides, guide_values, false)?;
    g.my_report.max_approximation_deviation = profile_deviation.max(guide_deviation);
    set_status(
        &mut g.my_report,
        GordonStatus::Done,
        GordonBuildStage::Approximation,
    );
    Ok(())
}

// The curves as a native 1-based array.
fn handles(
    curves: &[Handle<GeomBSplineCurve>],
) -> Result<Array1<'static, Handle<GeomBSplineCurve>>, Failure> {
    let mut array: Array1<'static, Handle<GeomBSplineCurve>> =
        Array1::with_bounds(1, curves.len() as i32)
            .map_err(|e| e.into_failure("GeomFill_Gordon"))?;
    for (i, c) in curves.iter().enumerate() {
        array[i as i32 + 1] = c.clone();
    }
    Ok(array)
}

// A native 1-based array of reals.
fn one_based(values: &[f64]) -> Result<Array1<'static, f64>, Failure> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32)
        .map_err(|e| e.into_failure("GeomFill_Gordon"))?;
    for (i, v) in values.iter().enumerate() {
        array[i as i32 + 1] = *v;
    }
    Ok(array)
}
