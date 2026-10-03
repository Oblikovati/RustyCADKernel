use super::algorithm::{interior_values, split_values};
use crate::{
    shape_analysis::{ShapeAnalysisCurve, ShapeAnalysisEdge},
    shape_extend::ShapeExtendWireData,
};
use rustycad_tkbrep::topods::{self, Face, ShapeIterator};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

type ParameterRange = (f64, f64);

struct CurveIntervals {
    x: ParameterRange,
    y: ParameterRange,
}

pub(super) fn seam_cuts(
    face: &Face,
    split_points: i32,
) -> Result<Option<(bool, super::super::SurfaceSplitValues)>, Failure> {
    for child in ShapeIterator::new(&face.clone().into_shape(), true, true) {
        if child.shape_type()? != rustycad_tkg3d::top_abs::ShapeEnum::Wire {
            continue;
        }
        let wire = topods::wire(&child)?;
        if let Some(cuts) = first_seam_cuts(face, &wire, split_points)? {
            return Ok(Some(cuts));
        }
    }
    Ok(None)
}

fn first_seam_cuts(
    face: &Face,
    wire: &rustycad_tkbrep::topods::Wire,
    split_points: i32,
) -> Result<Option<(bool, super::super::SurfaceSplitValues)>, Failure> {
    let wire_data = ShapeExtendWireData::new();
    wire_data.init_wire(wire, true, true)?;
    for index in 1..=wire_data.nb_edges() {
        if !wire_data.is_seam(index) {
            continue;
        }
        return seam_cut_values(face, &wire_data.edge(index)?, split_points).map(Some);
    }
    Ok(None)
}

fn seam_cut_values(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    split_points: i32,
) -> Result<(bool, super::super::SurfaceSplitValues), Failure> {
    let Some((first, second)) = seam_curve_pair(face, edge)? else {
        return Ok((false, split_values(&[])));
    };
    if first.curve == second.curve {
        return Ok((false, split_values(&[])));
    }
    cuts_between_curves(&first, &second, split_points)
}

fn seam_curve_pair(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
) -> Result<Option<(CurveRange, CurveRange)>, Failure> {
    let first = pcurve(edge, face)?;
    let second_edge = topods::edge(&edge.clone().into_shape().reversed())?;
    let Some(first) = first else { return Ok(None) };
    let Some(second) = pcurve(&second_edge, face)? else {
        return Ok(None);
    };
    Ok(Some((first, second)))
}

fn cuts_between_curves(
    first: &CurveRange,
    second: &CurveRange,
    split_points: i32,
) -> Result<(bool, super::super::SurfaceSplitValues), Failure> {
    let intervals = curve_separating_intervals(first, second)?;
    let split_u = range_length(intervals.x) > range_length(intervals.y);
    let interval = if split_u { intervals.x } else { intervals.y };
    let values = interior_values(interval, split_points);
    Ok((split_u, split_values(&values)))
}

fn curve_separating_intervals(
    first: &CurveRange,
    second: &CurveRange,
) -> Result<CurveIntervals, Failure> {
    let first_bounds = curve_bounds(&first.curve, first.first, first.last)?;
    let second_bounds = curve_bounds(&second.curve, second.first, second.last)?;
    let x = separating_interval(
        first_bounds[0],
        first_bounds[2],
        second_bounds[0],
        second_bounds[2],
    );
    let y = separating_interval(
        first_bounds[1],
        first_bounds[3],
        second_bounds[1],
        second_bounds[3],
    );
    Ok(CurveIntervals { x, y })
}

fn range_length(range: ParameterRange) -> f64 {
    range.1 - range.0
}

struct CurveRange {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

fn pcurve(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
) -> Result<Option<CurveRange>, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    if curve.is_null() {
        return Ok(None);
    }
    Ok(Some(CurveRange { curve, first, last }))
}

fn curve_bounds(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<[f64; 4], Failure> {
    let mut bounds = BoundingBox2::default();
    ShapeAnalysisCurve.fill_bnd_box(curve, first, last, 20, true, &mut bounds)?;
    bounds.get()
}

fn separating_interval(
    first_min: f64,
    first_max: f64,
    second_min: f64,
    second_max: f64,
) -> (f64, f64) {
    if first_min < second_min {
        (first_max, second_min)
    } else {
        (second_max, first_min)
    }
}
