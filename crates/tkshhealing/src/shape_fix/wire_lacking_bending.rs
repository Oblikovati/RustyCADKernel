//! `TryBendingPCurve` and the edge tolerance probe used by `FixLacking`.
use super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom2d_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Point2;

pub(super) struct BentPCurve {
    pub(super) curve: Handle<dyn Geom2dCurve>,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) tolerance: f64,
}

struct BendingTarget {
    point: Point2,
    at_end: bool,
    first: f64,
    last: f64,
}

impl ShapeFixWire {
    pub(super) fn bending_candidates(
        &self,
        previous: &Edge,
        current: &Edge,
        face: &Face,
        first: Point2,
        last: Point2,
    ) -> Result<Option<(BentPCurve, BentPCurve)>, Failure> {
        if !self.geometry_mode()
            || BRepTool::is_closed_on_face(previous, face)?
            || BRepTool::is_closed_on_face(current, face)?
        {
            return Ok(None);
        }
        let midpoint = Point2::new((first.x() + last.x()) * 0.5, (first.y() + last.y()) * 0.5);
        let left = self.try_bending_pcurve(
            previous,
            face,
            midpoint,
            previous.orientation() == Orientation::Forward,
        )?;
        let right = self.try_bending_pcurve(
            current,
            face,
            midpoint,
            current.orientation() == Orientation::Reversed,
        )?;
        if let (Some(left), Some(right)) = (left, right) {
            return Ok(Some((left, right)));
        }
        Ok(None)
    }

    fn try_bending_pcurve(
        &self,
        edge: &Edge,
        face: &Face,
        target: Point2,
        at_end: bool,
    ) -> Result<Option<BentPCurve>, Failure> {
        let Ok((curve, first, last)) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)
        else {
            return Ok(None);
        };
        if curve.is_null() {
            return Ok(None);
        }
        let target = BendingTarget {
            point: target,
            at_end,
            first,
            last,
        };
        let attempt = self.try_bending_spline(edge, face, &target, &curve);
        Ok(attempt.unwrap_or(None))
    }

    fn try_bending_spline(
        &self,
        edge: &Edge,
        face: &Face,
        target: &BendingTarget,
        curve: &Handle<dyn Geom2dCurve>,
    ) -> Result<Option<BentPCurve>, Failure> {
        let Some(spline) = bending_spline(curve, target.first, target.last)? else {
            return Ok(None);
        };
        let parameter = if target.at_end {
            target.last
        } else {
            target.first
        };
        if !move_spline_endpoint(&spline, target.point, parameter, target.first, target.last)? {
            return Ok(None);
        }
        self.try_new_pcurve(edge, face, spline.cast(), target.first, target.last)
    }

    fn try_new_pcurve(
        &self,
        edge: &Edge,
        face: &Face,
        pcurve: Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
    ) -> Result<Option<BentPCurve>, Failure> {
        let (curve, start, end) = BRepTool::curve(edge)?;
        if curve.is_null() {
            return Ok(None);
        }
        let temporary = temporary_edge(&curve, start, end)?;
        attach_temporary_pcurve(&temporary, face, &pcurve, first, last)?;
        ShapeFixEdge::new().fix_same_parameter_on_face(&temporary, face, 0.0)?;
        let fixed = BRepTool::curve_on_face(&temporary, face)?;
        Ok(Some(BentPCurve {
            curve: fixed.pcurve,
            first: fixed.first,
            last: fixed.last,
            tolerance: BRepTool::edge_tolerance(&temporary)?,
        }))
    }

    pub(super) fn apply_bent_pcurves(
        &self,
        previous: &Edge,
        current: &Edge,
        face: &Face,
        left: &BentPCurve,
        right: &BentPCurve,
    ) -> Result<bool, Failure> {
        attach_bent_curve(previous, face, left)?;
        attach_bent_curve(current, face, right)?;
        update_vertex_tolerances(previous, left.tolerance)?;
        update_vertex_tolerances(current, right.tolerance)?;
        self.or_last_fix_status(ShapeExtendStatus::Done5);
        Ok(true)
    }
}

fn temporary_edge(
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Edge, Failure> {
    let mut edge = Edge::default();
    ShapeBuildEdge.make_edge_on_curve_range(
        &mut edge,
        curve,
        &rustycad_tkmath::toploc::Location::default(),
        first,
        last,
    )?;
    Ok(edge)
}

fn attach_temporary_pcurve(
    edge: &Edge,
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, curve, face, 0.0)?;
    builder.range_on_face(edge, face, first, last)?;
    builder.same_range(edge, false)
}

fn attach_bent_curve(edge: &Edge, face: &Face, curve: &BentPCurve) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, &curve.curve, face, curve.tolerance)?;
    builder.range_on_face(edge, face, curve.first, curve.last)
}

fn update_vertex_tolerances(edge: &Edge, tolerance: f64) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let builder = BRepBuilder::default();
    builder.update_vertex_tolerance(&analysis.first_vertex(edge), tolerance)?;
    builder.update_vertex_tolerance(&analysis.last_vertex(edge), tolerance)
}

fn bending_spline(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Option<Handle<Geom2dBSplineCurve>>, Failure> {
    let source = curve.cast::<Geom2dBSplineCurve>();
    if let Some(source) = source.get() {
        return Ok(Some(Handle::new(Geom2dBSplineCurve::copy_of(source)?)));
    }
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
    Ok(Some(curve_to_bspline_curve(
        &trimmed,
        ParameterisationType::QuasiAngular,
    )?))
}

fn move_spline_endpoint(
    spline: &Handle<Geom2dBSplineCurve>,
    target: Point2,
    parameter: f64,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    if set_matching_endpoint(spline, target, parameter)? {
        return Ok(true);
    }
    spline
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire lost its temporary B-spline p-curve"))?
        .segment(first, last, precision::p_confusion())?;
    set_matching_endpoint(spline, target, parameter)
}

fn set_matching_endpoint(
    spline: &Handle<Geom2dBSplineCurve>,
    target: Point2,
    parameter: f64,
) -> Result<bool, Failure> {
    let spline = spline
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire lost its temporary B-spline p-curve"))?;
    if is_spline_endpoint(spline, 1, parameter)? {
        spline.set_pole(1, target)?;
        return Ok(true);
    }
    let last_knot = spline.nb_knots();
    if is_spline_endpoint(spline, last_knot, parameter)? {
        spline.set_pole(spline.nb_poles(), target)?;
        return Ok(true);
    }
    Ok(false)
}

fn is_spline_endpoint(
    spline: &Geom2dBSplineCurve,
    knot_index: i32,
    parameter: f64,
) -> Result<bool, Failure> {
    let endpoint = if knot_index == 1 {
        spline.first_parameter()
    } else {
        spline.last_parameter()
    };
    Ok((endpoint - parameter).abs() < precision::p_confusion()
        && spline.multiplicity(knot_index)? > spline.degree())
}
