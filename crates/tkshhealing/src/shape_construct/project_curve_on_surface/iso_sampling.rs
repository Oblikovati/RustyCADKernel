use super::isoparametric::{IsoRecognition, iso_uv};
use super::sampling::CurveSamples;
use super::*;

pub(super) struct IsoSampling {
    pub recognition: IsoRecognition,
    pub analytic: bool,
    value: f64,
    parameters: [f64; 2],
}
impl IsoSampling {
    pub fn new(
        analysis: &ShapeAnalysisSurface,
        mut recognition: IsoRecognition,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let surface = analysis.surface();
        let analytic = surface.dynamic_type().unwrap().name() != "Geom_BezierSurface"
            && surface.dynamic_type().unwrap().name() != "Geom_BSplineSurface";
        if recognition.recognized && !recognition.same_parameters && !analytic {
            adjust_iso_endpoints(analysis, &mut recognition, tolerance)?;
        }
        let constant = if recognition.u_iso { 1 } else { 2 };
        let value = recognition.endpoints[0].coord(constant)?;
        let parameters = [
            recognition.endpoints[0].coord(3 - constant)?,
            recognition.endpoints[1].coord(3 - constant)?,
        ];
        Ok(Self {
            recognition,
            analytic,
            value,
            parameters,
        })
    }

    pub fn parameterized_point(&self, samples: &CurveSamples, index: i32) -> Point2 {
        let parameter = if self.recognition.same_parameters {
            if self.parameters[1] > self.parameters[0] {
                samples.parameters[index]
            } else {
                self.recognition.range.0 + self.recognition.range.1 - samples.parameters[index]
            }
        } else if index == 1 {
            self.parameters[0]
        } else if index == samples.points.length() {
            self.parameters[1]
        } else {
            self.recognition.projected[index]
        };
        iso_uv(self.recognition.u_iso, self.value, parameter)
    }
}

fn adjust_iso_endpoints(
    analysis: &ShapeAnalysisSurface,
    iso: &mut IsoRecognition,
    tolerance: f64,
) -> Result<(), Failure> {
    let bounds = analysis.surface().bounds();
    let coordinate = if iso.u_iso { 2 } else { 1 };
    let range = if iso.u_iso {
        (bounds.2, bounds.3)
    } else {
        (bounds.0, bounds.1)
    };
    let closed = if iso.u_iso {
        analysis.is_v_closed(tolerance)?
    } else {
        analysis.is_u_closed(tolerance)?
    };
    let near = [iso.projected[2], iso.projected[iso.projected.length() - 1]];
    if closed {
        for (index, near) in near.into_iter().enumerate() {
            adjust_closed_endpoint(&mut iso.endpoints[index], coordinate, range, near)?;
        }
        return Ok(());
    }
    orient_iso_endpoints(iso, coordinate, near)
}
fn orient_iso_endpoints(
    iso: &mut IsoRecognition,
    coordinate: i32,
    near: [f64; 2],
) -> Result<(), Failure> {
    let first = iso.endpoints[0].coord(coordinate)?;
    let last = iso.endpoints[1].coord(coordinate)?;
    if (near[0] - first).abs() > (near[0] - last).abs()
        && (near[1] - last).abs() > (near[1] - first).abs()
    {
        iso.endpoints.swap(0, 1);
    }
    Ok(())
}

fn adjust_closed_endpoint(
    point: &mut Point2,
    coordinate: i32,
    range: (f64, f64),
    near: f64,
) -> Result<(), Failure> {
    let parameter = point.coord(coordinate)?;
    if parameter != range.0 && parameter != range.1 {
        return Ok(());
    }
    let parameter = if (near - range.0).abs() < (near - range.1).abs() {
        range.0
    } else {
        range.1
    };
    point.set_coord(coordinate, parameter)
}
