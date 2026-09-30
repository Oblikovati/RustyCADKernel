//! Native sampling distribution shared with face classification (TUBE.stp, 2002-06-05).
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d, geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::geom2d_int::Geom2dCurveTool;
use rustycad_tkmath::gp::Point2;

impl ShapeAnalysisCurve {
    /// Append the native p-curve sampling distribution; `ShapeAnalysisCurve::get_sample_points_2d(&curve,0.0,1.0,&mut points)?`.
    pub fn get_sample_points_2d(
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        points: &mut Sequence<Point2>,
    ) -> Result<bool, Failure> {
        let adaptor = Geom2dAdaptorCurve::with_range(curve, first, last)?;
        let mut count = Geom2dCurveTool::nb_samples(&adaptor)?;
        if count > 2 {
            count *= 4;
        }
        let step = (last - first) / f64::from(count - 1);
        for i in 0..count - 1 {
            points.append(adaptor.value(first + step * f64::from(i))?);
        }
        points.append(adaptor.value(last)?);
        Ok(true)
    }
}
