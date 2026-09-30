// Rust translation of GeomFill_GordonUtilities.pxx (contactPointGrid, curveWeightAt,
// contactWeightGrid and the NetworkPreparation class) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GordonUtilities::NetworkPreparation`: the preparatory steps of GeomFill_Gordon over
//! the working curves and parameter tables it owns.
use super::gordon_contacts::{ContactSet, adaptor_d0, fill_contact_table, max_nearest_contact_gap};
use super::gordon_normalize::{NetworkTables, normalize_contact_table};
use super::gordon_params::{ParamTable, make_unit_curves, null_curve};
use super::gordon_rebuild::{RebuildReport, equalize_parameters};
use super::gordon_reorder::{SourceNetwork, reorder_network};
use super::network_surface::has_rational_curve;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// `NetworkPreparation`: borrows the network data of GeomFill_Gordon and keeps the
/// contact sets and the measured gaps and deviations.
pub(crate) struct NetworkPreparation<'n> {
    my_profiles: &'n mut Vec<Handle<GeomBSplineCurve>>,
    my_guides: &'n mut Vec<Handle<GeomBSplineCurve>>,
    my_profile_params: &'n mut ParamTable,
    my_guide_params: &'n mut ParamTable,
    my_contact_sets: Vec<ContactSet>,
    my_tolerance: f64,
    my_max_contact_gap: f64,
    my_max_reparametrization_deviation: f64,
}

impl<'n> NetworkPreparation<'n> {
    /// The preparation of the network owned by the caller.
    pub(crate) fn new(
        (profiles, guides): (
            &'n mut Vec<Handle<GeomBSplineCurve>>,
            &'n mut Vec<Handle<GeomBSplineCurve>>,
        ),
        (profile_params, guide_params): (&'n mut ParamTable, &'n mut ParamTable),
        the_tolerance: f64,
    ) -> Self {
        Self {
            my_profiles: profiles,
            my_guides: guides,
            my_profile_params: profile_params,
            my_guide_params: guide_params,
            my_contact_sets: Vec::new(),
            my_tolerance: the_tolerance,
            my_max_contact_gap: 0.0,
            my_max_reparametrization_deviation: 0.0,
        }
    }
    /// `ConvertCurvesToUnitInterval()`: every curve non-periodic on `[0, 1]`.
    pub(crate) fn convert_curves_to_unit_interval(&mut self) -> Result<bool, Failure> {
        Ok(make_unit_curves(self.my_profiles)? && make_unit_curves(self.my_guides)?)
    }
    /// `FillIntersectionParameters()`: the contact table and the largest nearest gap.
    pub(crate) fn fill_intersection_parameters(&mut self) -> Result<bool, Failure> {
        let tables = (&mut *self.my_profile_params, &mut *self.my_guide_params);
        let is_done: bool = fill_contact_table(
            self.my_profiles,
            self.my_guides,
            tables,
            &mut self.my_contact_sets,
            self.my_tolerance,
        )?;
        self.my_max_contact_gap = if is_done {
            max_nearest_contact_gap(&self.my_contact_sets)
        } else {
            0.0
        };
        Ok(is_done)
    }
    /// `ReorderNetwork()`: the monotone ordering and orientation of the network.
    pub(crate) fn reorder_network(&mut self) -> Result<bool, Failure> {
        let src = SourceNetwork {
            profiles: self.my_profiles,
            guides: self.my_guides,
            profile_params: self.my_profile_params,
            guide_params: self.my_guide_params,
            contact_sets: &self.my_contact_sets,
        };
        reorder_network(src, self.my_tolerance)
    }
    /// `NormalizeIntersectionParameters(IsUClosed, IsVClosed)`: None when a contact is
    /// too far, otherwise the closed flags of both directions.
    pub(crate) fn normalize_intersection_parameters(
        &mut self,
    ) -> Result<Option<(bool, bool)>, Failure> {
        let mut net = NetworkTables {
            profiles: self.my_profiles,
            guides: self.my_guides,
            profile_params: self.my_profile_params,
            guide_params: self.my_guide_params,
        };
        normalize_contact_table(&mut net, self.my_tolerance)
    }
    /// `EqualizeIntersectionParameters(AllowApproximateRational, DidApproximate)`: the
    /// curves rebuilt on the averaged parameters; returns the success and whether a
    /// rational curve was rebuilt approximately.
    pub(crate) fn equalize_intersection_parameters(
        &mut self,
        the_allow_approximate_rational: bool,
        the_did_approximate: &mut bool,
    ) -> Result<bool, Failure> {
        let mut report = RebuildReport {
            allow_approximate_rational: the_allow_approximate_rational,
            did_approximate: *the_did_approximate,
            max_deviation: 0.0,
        };
        let curves = (&mut self.my_profiles[..], &mut self.my_guides[..]);
        let tables = (&mut *self.my_profile_params, &mut *self.my_guide_params);
        let done: bool = equalize_parameters(curves, tables, &mut report)?;
        *the_did_approximate = report.did_approximate;
        self.my_max_reparametrization_deviation = report.max_deviation;
        Ok(done)
    }
    /// `areClosedCurves(myProfiles, myTolerance)`: every profile meets itself at its
    /// ends within the tolerance (at least Precision::Confusion()).
    pub(crate) fn profiles_closed(&self) -> Result<bool, Failure> {
        if self.my_profiles.is_empty() {
            return Ok(false);
        }
        let a_tolerance: f64 = self.my_tolerance.max(precision::confusion());
        for a_handle in self.my_profiles.iter() {
            let Some(a_curve) = a_handle.get() else {
                return Ok(false);
            };
            let first = a_curve.value(a_curve.first_parameter())?;
            let last = a_curve.value(a_curve.last_parameter())?;
            if first.distance(&last) > a_tolerance {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// `hasRationalCurve(myProfiles, myGuides)`: one working curve is rational.
    pub(crate) fn has_rational_curve(&self) -> bool {
        has_rational_curve(self.my_profiles) || has_rational_curve(self.my_guides)
    }
    /// `MaxContactGap()`.
    pub(crate) fn max_contact_gap(&self) -> f64 {
        self.my_max_contact_gap
    }
    /// `MaxReparametrizationDeviation()`.
    pub(crate) fn max_reparametrization_deviation(&self) -> f64 {
        self.my_max_reparametrization_deviation
    }
    /// `IntersectionPoints()`: the contact points in (guide, profile) grid order.
    pub(crate) fn intersection_points(&self) -> Result<Array2<'static, Point3>, Failure> {
        let table: &ParamTable = self.my_profile_params;
        let mut a_points = grid(table)?;
        for (pi, a_profile) in self.my_profiles.iter().enumerate() {
            for gi in 0..table.cols {
                a_points[(gi as i32 + 1, pi as i32 + 1)] = adaptor_d0(a_profile, table.at(pi, gi))?;
            }
        }
        Ok(a_points)
    }
    /// `IntersectionWeights()`: the curve weights at the contacts, in (guide, profile)
    /// grid order (1 for a polynomial profile).
    pub(crate) fn intersection_weights(&self) -> Result<Array2<'static, f64>, Failure> {
        let table: &ParamTable = self.my_profile_params;
        let mut a_weights = grid(table)?;
        for (pi, a_profile) in self.my_profiles.iter().enumerate() {
            for gi in 0..table.cols {
                a_weights[(gi as i32 + 1, pi as i32 + 1)] =
                    curve_weight_at(a_profile, table.at(pi, gi))?;
            }
        }
        Ok(a_weights)
    }
}

// A (guides x profiles) grid for the table.
fn grid<T: Default + Clone>(table: &ParamTable) -> Result<Array2<'static, T>, Failure> {
    Array2::with_bounds(1, table.cols as i32, 1, table.rows as i32)
        .map_err(|e| e.into_failure("GeomFill_Gordon"))
}

/// `curveWeightAt`: the denominator of a rational curve at the parameter
/// (BSplCLib::Eval, non homogeneous, extrapolated with the degree on both sides).
pub(crate) fn curve_weight_at(
    the_curve: &Handle<GeomBSplineCurve>,
    the_parameter: f64,
) -> Result<f64, Failure> {
    let curve = the_curve.get().ok_or_else(null_curve)?;
    if !curve.is_rational() {
        return Ok(1.0);
    }
    let an_extrap_mode: [i32; 2] = [curve.degree(), curve.degree()];
    let (mut a_point, mut a_weight) = (Point3::default(), 1.0);
    bsplclib::eval_extrapolated_point(
        the_parameter,
        curve.is_periodic(),
        false,
        &an_extrap_mode,
        curve.degree(),
        &curve.knot_sequence(),
        &curve.poles(),
        &curve.weights_array(|w| w.clone()),
        &mut a_point,
        &mut a_weight,
    )?;
    Ok(a_weight)
}
