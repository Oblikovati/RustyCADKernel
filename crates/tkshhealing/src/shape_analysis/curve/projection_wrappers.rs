//! Range adjustment and endpoint precedence around the common projection algorithm.
use super::projection_act::project_act;
use super::projection_endpoints::{EndpointCandidates, snapping_tolerance};
use super::projection_segments::local_projection;
use super::{CurveProjection, CurveProjectionResult, ShapeAnalysisCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Point3;

fn require_curve(curve: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeAnalysis_Curve: null curve; expected a live Geom_Curve",
        )
    })
}
fn prepare_curve(
    curve: &Handle<dyn GeomCurve>,
    point: Point3,
    precision: f64,
    range: (f64, f64),
    adjust: bool,
    output: &mut CurveProjection,
) -> Result<(GeomAdaptorCurve, Option<f64>), Failure> {
    let geometry = require_curve(curve)?;
    let (mut first, mut last) = if range.0 < range.1 {
        range
    } else {
        (range.1, range.0)
    };
    let adaptor = GeomAdaptorCurve::with_range(curve, first, last)?;
    if !curve.cast::<dyn GeomBoundedCurve>().is_null() {
        let endpoints = EndpointCandidates::new(&adaptor)?;
        if let Some(distance) = endpoints.snap(point, snapping_tolerance(precision, adjust), output)
        {
            return Ok((adaptor, Some(distance)));
        }
    }
    if !geometry.is_closed()? {
        // PRO11641 / BUC60195: extend by resolution, capped at a tenth of the range.
        let delta = std_min(adaptor.resolution(precision)?, (last - first) * 0.1);
        first -= delta;
        last += delta;
        adaptor.load_range(curve, first, last)?;
    }
    Ok((adaptor, None))
}
impl ShapeAnalysisCurve {
    /// Project over the natural curve range; `ShapeAnalysisCurve.project_full(&c,p,1e-7,true)?`.
    pub fn project_full(
        &self,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        adjust: bool,
    ) -> Result<CurveProjectionResult, Failure> {
        let geometry = require_curve(curve)?;
        self.project(
            curve,
            point,
            precision,
            (geometry.first_parameter(), geometry.last_parameter()),
            adjust,
        )
    }
    /// Project over a range, extending nonclosed curves; `ShapeAnalysisCurve.project(&c,p,1e-7,(0.0,1.0),true)?`.
    pub fn project(
        &self,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        range: (f64, f64),
        adjust: bool,
    ) -> Result<CurveProjectionResult, Failure> {
        let mut output = CurveProjection::default();
        let distance = self.project_into(curve, point, precision, range, adjust, &mut output)?;
        Ok(output.tuple(distance))
    }
    /// Preserve native output mutation; `ShapeAnalysisCurve.project_into(&c,p,1e-7,(0.0,1.0),true,&mut output)?`.
    pub fn project_into(
        &self,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        range: (f64, f64),
        adjust: bool,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        let (adaptor, endpoint) = prepare_curve(curve, point, precision, range, adjust, output)?;
        if let Some(distance) = endpoint {
            return Ok(distance);
        }
        project_act(&adaptor, point, precision, output)
    }
    /// Global extrema and original fallback search; `ShapeAnalysisCurve.project_act(&adaptor,p,1e-7,&mut output)?`.
    pub fn project_act(
        &self,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        project_act(curve, point, precision, output)
    }
    /// Project with endpoint comparison on an adaptor; `ShapeAnalysisCurve.project_adaptor(&adaptor,p,1e-7,true)?`.
    pub fn project_adaptor(
        &self,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
        adjust: bool,
    ) -> Result<CurveProjectionResult, Failure> {
        let mut output = CurveProjection::default();
        let distance = self.project_adaptor_into(curve, point, precision, adjust, &mut output)?;
        Ok(output.tuple(distance))
    }
    /// Preserve caller outputs through adaptor projection; `ShapeAnalysisCurve.project_adaptor_into(&adaptor,p,1e-7,true,&mut output)?`.
    pub fn project_adaptor_into(
        &self,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
        adjust: bool,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        let bounds = (curve.first_parameter()?, curve.last_parameter()?);
        if precision::is_infinite(bounds.0) && precision::is_infinite(bounds.1) {
            return project_act(curve, point, precision, output);
        }
        let endpoints = EndpointCandidates::new(curve)?;
        let distances = endpoints.distances(point);
        if let Some(distance) =
            endpoints.snap_distances(distances, snapping_tolerance(precision, adjust), output)
        {
            return Ok(distance);
        }
        let distance = project_act(curve, point, precision, output)?;
        Ok(endpoints.prefer_projection(distances, distance, output))
    }
    /// Start with the local Newton projection; `ShapeAnalysisCurve.next_project(0.5,&c,p,1e-7,(0.0,1.0),false)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn next_project(
        &self,
        previous: f64,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        range: (f64, f64),
        adjust: bool,
    ) -> Result<CurveProjectionResult, Failure> {
        let mut output = CurveProjection::default();
        let distance = self.next_project_into(
            previous,
            curve,
            point,
            precision,
            range,
            adjust,
            &mut output,
        )?;
        Ok(output.tuple(distance))
    }
    /// Preserve outputs while trying the previous parameter; `ShapeAnalysisCurve.next_project_into(0.5,&c,p,1e-7,(0.0,1.0),false,&mut output)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn next_project_into(
        &self,
        previous: f64,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        range: (f64, f64),
        adjust: bool,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        let (adaptor, endpoint) = prepare_curve(curve, point, precision, range, adjust, output)?;
        if let Some(distance) = endpoint {
            return Ok(distance);
        }
        self.next_project_adaptor_into(previous, &adaptor, point, precision, output)
    }
    /// Local adaptor projection with global fallback; `ShapeAnalysisCurve.next_project_adaptor(0.5,&adaptor,p,1e-7)?`.
    pub fn next_project_adaptor(
        &self,
        previous: f64,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
    ) -> Result<CurveProjectionResult, Failure> {
        let mut output = CurveProjection::default();
        let distance =
            self.next_project_adaptor_into(previous, curve, point, precision, &mut output)?;
        Ok(output.tuple(distance))
    }
    /// Preserve native local-projection outputs; `ShapeAnalysisCurve.next_project_adaptor_into(0.5,&adaptor,p,1e-7,&mut output)?`.
    pub fn next_project_adaptor_into(
        &self,
        previous: f64,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        let bounds = (curve.first_parameter()?, curve.last_parameter()?);
        if let Some(distance) = local_projection(curve, point, previous, bounds, precision, output)?
        {
            return Ok(distance);
        }
        self.project_adaptor_into(curve, point, precision, false, output)
    }
}

impl ShapeAnalysisCurve {
    /// Natural-range projection preserving output state; `ShapeAnalysisCurve.project_full_into(&c,p,1e-7,true,&mut output)?`.
    pub fn project_full_into(
        &self,
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        precision: f64,
        adjust: bool,
        output: &mut CurveProjection,
    ) -> Result<f64, Failure> {
        let geometry = require_curve(curve)?;
        self.project_into(
            curve,
            point,
            precision,
            (geometry.first_parameter(), geometry.last_parameter()),
            adjust,
            output,
        )
    }
    /// Return the native ProjectAct result as a tuple; `ShapeAnalysisCurve.project_act_adaptor(&adaptor,p,1e-7)?`.
    pub fn project_act_adaptor(
        &self,
        curve: &dyn Adaptor3dCurve,
        point: Point3,
        precision: f64,
    ) -> Result<CurveProjectionResult, Failure> {
        let mut output = CurveProjection::default();
        let distance = self.project_act(curve, point, precision, &mut output)?;
        Ok(output.tuple(distance))
    }
}
