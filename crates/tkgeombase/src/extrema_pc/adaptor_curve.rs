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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_OffsetCurve.hxx/.cxx and
// ExtremaPC_OtherCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_OffsetCurve` and `ExtremaPC_OtherCurve`: numerical extrema through a 3D
//! adaptor over a curve-aware partition built at construction. The offset evaluator
//! first tries the exact 2D offset curve of a planar offset (ExtremaPC::ProjectOffset).
use super::spline::not_done;
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status,
    add_endpoint_extrema, project_offset,
};
use crate::extrema_pc_core::{PcCurveRef, Space3d, is_valid_tolerance};
use crate::extrema_pc2d;
use crate::plane_projection as projlib;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::geom2d::Geom2dOffsetCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomOffsetCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkmath::math::Vector;

/// The adaptor reference of the evaluators.
pub type CurveRef3d<'a> = PcCurveRef<'a, Space3d>;

/// The adaptor, evaluated by AddEndpointExtrema apart from the evaluator's result.
struct AdaptorShape<'a>(CurveRef3d<'a>);

impl CurveEvaluator<Point3> for AdaptorShape<'_> {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
        self.0.get().ok_or_else(null_adaptor)?.value(u)
    }
}

// Natively a null adaptor pointer is dereferenced.
fn null_adaptor() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ExtremaPC: Value on a null adaptor; expected a curve",
        "",
    )
}

/// The shared body of the 3D offset and general-curve evaluators.
struct AdaptorExtrema<'a> {
    shape: AdaptorShape<'a>,
    domain: Domain1D,
    evaluator: GridEvaluator,
}

impl<'a> AdaptorExtrema<'a> {
    // The constructors: the adaptor bounds when no domain is given, then buildParams.
    fn new(curve: CurveRef3d<'a>, domain: Option<Domain1D>) -> Result<Self, Failure> {
        let domain: Domain1D = match domain {
            Some(domain) => domain,
            None => {
                let adaptor = curve.get().ok_or_else(null_adaptor)?;
                Domain1D::new(adaptor.first_parameter()?, adaptor.last_parameter()?)
            }
        };
        let mut result = Self {
            shape: AdaptorShape(curve),
            domain,
            evaluator: GridEvaluator::default(),
        };
        if let Some(adaptor) = result.shape.0.get()
            && result.domain.is_valid()
            && result.domain.is_finite(1.0e100)
        {
            let params: Vector<'static> =
                GridEvaluator::build_curve_aware_params(adaptor, &result.domain)?;
            result.evaluator.set_params(&params);
        }
        Ok(result)
    }
    // `myEvaluator.Perform(*myCurve, ...)`, NotDone without an adaptor.
    fn grid_perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        let Some(curve) = self.shape.0.get() else {
            return Ok(not_done(self.evaluator.result()));
        };
        self.evaluator
            .perform(curve, &point, &self.domain, tol, mode)
    }
    // The endpoint pass of PerformWithEndpoints after the interior extrema.
    fn add_endpoints(
        &mut self,
        point: Point3,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        let status: Status = self.evaluator.result().status;
        if let Some(curve) = self.shape.0.get()
            && (status == Status::OK || status == Status::NoSolution)
        {
            let is_closed: bool = GridEvaluator::is_closed_domain(curve, &self.domain);
            let result: &mut ExtremaResult = self.evaluator.result();
            add_endpoint_extrema(result, &point, &self.domain, &self.shape, mode, is_closed)?;
            if !result.extrema.is_empty() {
                result.status = Status::OK;
            }
        }
        Ok(self.evaluator.result())
    }
    // `performPlanar` of the offset evaluator: false when the adaptor is not a
    // GeomAdaptor_Curve on an exactly planar offset (every failure there is caught).
    fn perform_planar_offset(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
        ends: bool,
    ) -> Result<bool, Failure> {
        let mut plane = Plane3::default();
        let Ok(Some(offset2d)) = self.planar_offset(&mut plane) else {
            return Ok(false);
        };
        let adaptor2d =
            Geom2dAdaptorCurve::with_range(&offset2d.cast(), self.domain.min, self.domain.max)?;
        let domain2d = Domain1D::new(self.domain.min, self.domain.max);
        let mut evaluator2d = extrema_pc2d::OffsetCurve::with_domain(&adaptor2d, domain2d);
        let mode2d: SearchMode = super::to_search_mode_2d(mode);
        let point2d = projlib::project_point(&plane, point);
        let result2d = if ends {
            evaluator2d.perform_with_endpoints(point2d, tol, mode2d)?
        } else {
            evaluator2d.perform(point2d, tol, mode2d)?
        };
        super::convert_planar_result(
            result2d,
            &point,
            &plane,
            &self.shape,
            self.evaluator.result(),
        )
    }
    // The try block of performPlanar: the 2D offset curve, if any.
    fn planar_offset(
        &self,
        plane: &mut Plane3,
    ) -> Result<Option<Handle<Geom2dOffsetCurve>>, Failure> {
        let Some(curve) = self.shape.0.get() else {
            return Ok(None);
        };
        let upcast: &dyn Transient = curve;
        let Some(adaptor) = GeomAdaptorCurve::project_from(upcast) else {
            return Ok(None);
        };
        let offset: Handle<GeomOffsetCurve> = adaptor.curve().cast();
        let mut offset2d: Handle<Geom2dOffsetCurve> = Handle::default();
        if !project_offset(&offset, plane, &mut offset2d)? {
            return Ok(None);
        }
        Ok(Some(offset2d))
    }
}

/// Point-offset-curve extrema through a 3D adaptor (`ExtremaPC_OffsetCurve`): a planar
/// offset of a GeomAdaptor_Curve is solved by `ExtremaPC2d_OffsetCurve` on its exact 2D
/// offset; otherwise by the grid evaluator. The adaptor must outlive the evaluator;
/// `OffsetCurve::with_domain(&adaptor, domain)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct OffsetCurve<'a>(AdaptorExtrema<'a>);

/// Point-curve extrema through any 3D adaptor (`ExtremaPC_OtherCurve`), by the grid
/// evaluator; `OtherCurve::with_domain(&adaptor, domain)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct OtherCurve<'a>(AdaptorExtrema<'a>);

// The members both evaluators share, with the native class names.
macro_rules! adaptor_members {
    ($name:ident, $native:literal) => {
        impl<'a> $name<'a> {
            #[doc = concat!("Evaluator over the adaptor bounds (`", $native, "(theCurve)`); `new(&adaptor)?`.")]
            pub fn new(curve: &'a dyn Adaptor3dCurve) -> Result<Self, Failure> {
                AdaptorExtrema::new(PcCurveRef::Borrowed(curve), None).map(Self)
            }
            #[doc = concat!("Evaluator over `domain` (`", $native, "(theCurve, theDomain)`); `with_domain(&adaptor, domain)?`.")]
            pub fn with_domain(curve: &'a dyn Adaptor3dCurve, domain: Domain1D) -> Result<Self, Failure> {
                AdaptorExtrema::new(PcCurveRef::Borrowed(curve), Some(domain)).map(Self)
            }
            /// Evaluator over a borrowed or shared adaptor, with the adaptor bounds when
            /// `domain` is `None`; `from_curve_ref(PcCurveRef::Shared(handle), None)?`.
            pub fn from_curve_ref(curve: CurveRef3d<'a>, domain: Option<Domain1D>) -> Result<Self, Failure> {
                AdaptorExtrema::new(curve, domain).map(Self)
            }
            /// The adaptor point at `u` (`Value`); `extrema.value(0.5)?`.
            pub fn value(&self, u: f64) -> Result<Point3, Failure> {
                self.0.shape.evaluate(u)
            }
            /// True for a finite domain (`IsBounded`); `extrema.is_bounded()`.
            pub fn is_bounded(&self) -> bool {
                self.0.domain.is_finite(1.0e100)
            }
            /// The domain (`Domain`); `extrema.domain()`.
            pub fn domain(&self) -> &Domain1D {
                &self.0.domain
            }
        }

        impl CurveEvaluator<Point3> for $name<'_> {
            type Error = Failure;
            fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
                self.value(u)
            }
        }
    };
}

adaptor_members!(OffsetCurve, "ExtremaPC_OffsetCurve");
adaptor_members!(OtherCurve, "ExtremaPC_OtherCurve");

impl OffsetCurve<'_> {
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.0.shape.0.get().is_none() {
            return Ok(not_done(self.0.evaluator.result()));
        }
        if is_valid_tolerance(tol)
            && self.0.domain.is_valid()
            && self.0.perform_planar_offset(point, tol, mode, false)?
        {
            return Ok(self.0.evaluator.result());
        }
        self.0.grid_perform(point, tol, mode)
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.0.shape.0.get().is_none() {
            return Ok(not_done(self.0.evaluator.result()));
        }
        if is_valid_tolerance(tol)
            && self.0.domain.is_valid()
            && self.0.perform_planar_offset(point, tol, mode, true)?
        {
            return Ok(self.0.evaluator.result());
        }
        self.0.grid_perform(point, tol, mode)?;
        self.0.add_endpoints(point, mode)
    }
}

impl OtherCurve<'_> {
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.0.grid_perform(point, tol, mode)
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.0.shape.0.get().is_none() {
            return Ok(not_done(self.0.evaluator.result()));
        }
        self.0.grid_perform(point, tol, mode)?;
        self.0.add_endpoints(point, mode)
    }
}
