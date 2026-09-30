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
// Rust translation of GeomFill_Gordon.hxx/.cxx (the class, its enumerations and its
// report) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Gordon`: the Gordon surface of a network of profile and guide curves.
use super::gordon_params::ParamTable;

use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve};
use rustycad_tkgeombase::geom_convert;
use rustycad_tkmath::convert::ParameterisationType;

/// `GeomFill_Gordon::ResultStatus`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GordonStatus {
    /// Perform() has not been called since initialization.
    #[default]
    NotStarted,
    /// Surface has been constructed.
    Done,
    /// Input network has too few profile or guide curves.
    InvalidInput,
    /// Curves could not be converted/reparametrized to B-splines.
    ConversionFailed,
    /// Full profile/guide intersection table could not be built.
    IntersectionFailed,
    /// Network curves could not be ordered consistently.
    OrderingFailed,
    /// Intersections could not be equalized in parameter space.
    ReparametrizationFailed,
    /// Prepared network failed geometric compatibility checks.
    CompatibilityFailed,
    /// Prepared curve families are not B-spline compatible.
    CurveCompatibilityFailed,
    /// Rational curves require unsupported exact reparametrization.
    RationalReparametrizationFailed,
    /// Intermediate profile/guide skinning has failed.
    SkinningFailed,
    /// Intersection-grid reference surface could not be built.
    ReferenceSurfaceFailed,
    /// Intermediate surfaces could not be aligned.
    KnotAlignmentFailed,
    /// Exact rational product degree exceeds OCCT's B-spline limit.
    RationalDegreeOverflow,
    /// Exact rational numerator/denominator construction has failed.
    RationalConstructionFailed,
    /// Closed seam could not be converted to periodic form.
    PeriodicityFailed,
    /// Optional approximate fallback has failed.
    ApproximationFailed,
    /// Final B-spline surface construction has failed.
    ConstructionFailed,
}

/// `GeomFill_Gordon::ApproximationMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GordonApproximationMode {
    /// Report exact construction failure (default).
    #[default]
    ExactOnly,
    /// Try a sampled B-spline fallback without exact interpolation.
    AllowApproximateFallback,
}

/// `GeomFill_Gordon::BuildStage`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GordonBuildStage {
    /// Perform() has not started.
    #[default]
    NotStarted,
    /// Input curves are being converted to working B-splines.
    InputConversion,
    /// Profile/guide contacts are being collected.
    ContactDiscovery,
    /// Contacts and curves are being ordered into a monotone network.
    NetworkOrdering,
    /// Curves are being rebuilt to shared network parameters.
    Reparametrization,
    /// Exact B-spline network surface is being constructed.
    ExactConstruction,
    /// Result is being checked against prepared curves.
    Validation,
    /// Optional sampled fallback is being built.
    Approximation,
}

/// `GeomFill_Gordon::BuildReport`: the status, the failing stage and the measured
/// gaps and deviations of the last Perform.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GordonBuildReport {
    /// `Status`.
    pub status: GordonStatus,
    /// `FailedStage`.
    pub failed_stage: GordonBuildStage,
    /// `IsApproximate`.
    pub is_approximate: bool,
    /// `MaxContactGap`.
    pub max_contact_gap: f64,
    /// `MaxReparametrizationDeviation`.
    pub max_reparametrization_deviation: f64,
    /// `MaxProfileDeviation`.
    pub max_profile_deviation: f64,
    /// `MaxGuideDeviation`.
    pub max_guide_deviation: f64,
    /// `MaxApproximationDeviation`.
    pub max_approximation_deviation: f64,
}

/// `GeomFill_Gordon`: the input curves are converted to B-splines at Init; Perform
/// works on deep copies: it makes them non-periodic on `[0, 1]`, finds the contacts of
/// each profile/guide pair, orders and orients the network, rebuilds the curves so that
/// all contacts share averaged parameters, and builds the surface with
/// GeomFill_NetworkSurface (or, when allowed, a sampled approximation). The parallel
/// mode computes the same contacts, so the port runs it sequentially.
///
/// ```
/// use rustycad_tkgeomalgo::geom_fill::{GeomFillGordon, GordonStatus};
/// let mut gordon = GeomFillGordon::new();
/// gordon.perform().unwrap();
/// assert_eq!(gordon.status(), GordonStatus::InvalidInput);
/// ```
#[derive(Default)]
pub struct GeomFillGordon {
    pub(super) my_input_profiles: Vec<Handle<GeomBSplineCurve>>,
    pub(super) my_input_guides: Vec<Handle<GeomBSplineCurve>>,
    pub(super) my_profiles: Vec<Handle<GeomBSplineCurve>>,
    pub(super) my_guides: Vec<Handle<GeomBSplineCurve>>,
    pub(super) my_profile_params: ParamTable,
    pub(super) my_guide_params: ParamTable,
    pub(super) my_surface: Handle<GeomBSplineSurface>,
    pub(super) my_tolerance: f64,
    pub(super) my_is_u_closed: bool,
    pub(super) my_is_v_closed: bool,
    pub(super) my_to_use_parallel: bool,
    pub(super) my_approximation_mode: GordonApproximationMode,
    pub(super) my_report: GordonBuildReport,
}

impl GeomFillGordon {
    /// `GeomFill_Gordon()`: not started. `GeomFillGordon::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Init(Profiles, Guides, Tolerance)`: the report and the surface reset; with two
    /// profiles and two guides at least, each curve converted by
    /// GeomConvert::CurveToBSplineCurve (whose exceptions are returned).
    /// `gordon.init(&profiles, &guides, 1e-4)?`.
    pub fn init(
        &mut self,
        the_profiles: &Array1<'_, Handle<dyn GeomCurve>>,
        the_guides: &Array1<'_, Handle<dyn GeomCurve>>,
        the_tolerance: f64,
    ) -> Result<(), Failure> {
        self.my_tolerance = the_tolerance;
        (self.my_is_u_closed, self.my_is_v_closed) = (false, false);
        self.my_report = GordonBuildReport::default();
        self.my_surface = Handle::default();
        let (nb_prof, nb_guid) = (the_profiles.len(), the_guides.len());
        if nb_prof < 2 || nb_guid < 2 {
            // Store counts but don't try to convert - Perform() will check and return.
            self.my_input_profiles = vec![Handle::default(); nb_prof];
            self.my_input_guides = vec![Handle::default(); nb_guid];
            return Ok(());
        }
        // Convert input arrays to immutable B-spline seeds. Perform() deep-copies these
        // seeds before applying destructive preparation steps.
        self.my_input_profiles = Vec::with_capacity(nb_prof);
        self.my_input_guides = Vec::with_capacity(nb_guid);
        for curve in the_profiles.iter() {
            self.my_input_profiles
                .push(geom_convert::curve_to_bspline_curve(
                    curve,
                    ParameterisationType::TgtThetaOver2,
                )?);
        }
        for curve in the_guides.iter() {
            self.my_input_guides
                .push(geom_convert::curve_to_bspline_curve(
                    curve,
                    ParameterisationType::TgtThetaOver2,
                )?);
        }
        self.my_profile_params = ParamTable::new(nb_prof, nb_guid);
        self.my_guide_params = ParamTable::new(nb_prof, nb_guid);
        Ok(())
    }
    /// `Perform()`: the construction; a status reports each failure, and the exceptions
    /// that the native Perform lets through are returned. `gordon.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        super::gordon_perform::perform(self)
    }
    /// `SetParallelMode(ToUseParallel)`. `gordon.set_parallel_mode(true)`.
    pub fn set_parallel_mode(&mut self, the_to_use_parallel: bool) {
        self.my_to_use_parallel = the_to_use_parallel;
    }
    /// `SetApproximationMode(Mode)`.
    /// `gordon.set_approximation_mode(GordonApproximationMode::AllowApproximateFallback)`.
    pub fn set_approximation_mode(&mut self, the_mode: GordonApproximationMode) {
        self.my_approximation_mode = the_mode;
    }
    /// `GetApproximationMode()`. `gordon.get_approximation_mode()`.
    pub fn get_approximation_mode(&self) -> GordonApproximationMode {
        self.my_approximation_mode
    }
    /// `IsParallelMode()`. `gordon.is_parallel_mode()`.
    pub fn is_parallel_mode(&self) -> bool {
        self.my_to_use_parallel
    }
    /// `IsDone()`. `gordon.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_report.status == GordonStatus::Done
    }
    /// `IsApproximate()`. `gordon.is_approximate()`.
    pub fn is_approximate(&self) -> bool {
        self.my_report.is_approximate
    }
    /// `Status()`. `gordon.status()`.
    pub fn status(&self) -> GordonStatus {
        self.my_report.status
    }
    /// `Report()`. `gordon.report().max_contact_gap`.
    pub fn report(&self) -> &GordonBuildReport {
        &self.my_report
    }
    /// `Surface()`: StdFail_NotDone unless done. `gordon.surface()?`.
    pub fn surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        if !self.is_done() {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                &format!(
                    "GeomFill_Gordon::Surface - status {:?}; expected Done",
                    self.my_report.status
                ),
                "",
            ));
        }
        Ok(self.my_surface.clone())
    }
}
