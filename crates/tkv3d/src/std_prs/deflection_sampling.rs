//! Native chordal-deviation sampling, including C1 interval joins and append ordering.
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::{Array1, ArrayError, Sequence},
    precision,
    standard::{Failure, Handle, real::real_last},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::{
    bnd_lib::Add3dCurve,
    gcpnts::{TangentialCriteria, TangentialDeflection},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    bvh::{std_max, std_min},
    geom_abs::{CurveType, Shape},
    gp::Point3,
};
use rustycad_tkservice::{aspect::AspectTypeOfDeflection, graphic3d::*};
pub(super) fn drawer_deflection(
    curve: &dyn Adaptor3dCurve,
    bounds: [f64; 2],
    drawer: &Prs3dDrawer,
) -> Result<f64, Failure> {
    if drawer.type_of_deflection() != AspectTypeOfDeflection::Relative {
        return Ok(drawer.maximal_chordial_deviation());
    }
    let mut bbox = BoundingBox3::default();
    Add3dCurve::add_range(curve, bounds[0], bounds[1], 0., &mut bbox)?;
    let b = bbox.get()?;
    // Native initializes from X only when closed; an open X retains RealLast.
    let mut length = real_last();
    if !bbox.is_open_xmin() && !bbox.is_open_xmax() {
        length = (b[3] - b[0]).abs();
    }
    if !bbox.is_open_ymin() && !bbox.is_open_ymax() {
        length = std_max(length, (b[4] - b[1]).abs());
    }
    if !bbox.is_open_zmin() && !bbox.is_open_zmax() {
        length = std_max(length, (b[5] - b[2]).abs());
    }
    length = std_max(
        std_min(length, drawer.maximal_parameter_value()),
        precision::confusion(),
    );
    Ok(length * drawer.deviation_coefficient())
}
pub(super) fn sample_intervals(
    curve: &dyn Adaptor3dCurve,
    bounds: [f64; 2],
    deflection: f64,
    angle: f64,
) -> Result<Vec<Point3>, Failure> {
    let count = curve.nb_intervals(Shape::C1)?;
    let mut knots = Array1::with_bounds(1, count + 1).map_err(interval_failure)?;
    curve.intervals(&mut knots, Shape::C1)?;
    let mut points = Vec::new();
    for index in 1..=count {
        let (first, last) = (
            *knots.value(index).map_err(interval_failure)?,
            *knots.value(index + 1).map_err(interval_failure)?,
        );
        if last > bounds[0] && first < bounds[1] {
            let distribution = TangentialDeflection::with_range(
                curve,
                std_max(first, bounds[0]),
                std_min(last, bounds[1]),
                TangentialCriteria::new(angle, deflection),
            )?;
            for index in 1..=distribution.nb_points() {
                points.push(distribution.value(index)?);
            }
        }
    }
    Ok(points)
}
pub(super) fn draw_deflection(
    curve: &dyn Adaptor3dCurve,
    group: &Handle<dyn Graphic3dGroup>,
    bounds: [f64; 2],
    deflection: f64,
    angle: f64,
    points: &mut Sequence<Point3>,
) -> Result<(), Failure> {
    let line = curve.get_type()? == CurveType::Line;
    let sampled = if line {
        vec![curve.value(bounds[0])?, curve.value(bounds[1])?]
    } else {
        sample_intervals(curve, bounds, deflection, angle)?
    };
    // Native lines append before allocation; general curves allocate before appending.
    if line {
        for point in &sampled {
            points.append(*point);
        }
    }
    let array = if group.is_null() {
        None
    } else {
        Some(Handle::new(Graphic3dArrayOfPrimitives::new(
            if line {
                Graphic3dTypeOfPrimitiveArray::Segments
            } else {
                Graphic3dTypeOfPrimitiveArray::Polylines
            },
            sampled.len() as i32,
            0,
            0,
            0,
        )?))
    };
    for point in sampled {
        if !line {
            points.append(point);
        }
        if let Some(array) = &array {
            array.add_vertex_pnt(&point)?;
        }
    }
    if let Some(array) = array {
        group.add_primitive_array(&array, true);
    }
    Ok(())
}

fn interval_failure(error: ArrayError) -> Failure {
    error.into_failure("StdPrs_DeflectionCurve C1 interval bounds")
}
