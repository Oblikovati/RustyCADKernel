use super::{integer, real};
use crate::bean_fast_analytic::{self, AnalyticCurve, AnalyticSurface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};

fn axis(values: &[f64], offset: usize) -> Result<Axis1, Failure> {
    let location = Point3::new(values[offset], values[offset + 1], values[offset + 2]);
    let direction = Direction3::new(values[offset + 3], values[offset + 4], values[offset + 5])?;
    Ok(Axis1::new(location, direction))
}

pub(super) fn run(values: &[f64]) -> Result<(), Failure> {
    let curve_axis = axis(values, 2)?;
    let surface_axis = axis(values, 9)?;
    let curve = AnalyticCurve {
        kind: values[0] as i32,
        axis: curve_axis,
        radius: values[8],
    };
    let surface = AnalyticSurface {
        kind: values[1] as i32,
        axis: surface_axis,
        radius: values[15],
    };
    let classified =
        bean_fast_analytic::classify(curve, surface, values[16], values[17], values[18]);
    integer(i32::from(classified.successful));
    integer(i32::from(classified.coincident_range.is_some()));
    if let Some(range) = classified.coincident_range {
        real(range.first());
        real(range.last());
    }
    Ok(())
}
