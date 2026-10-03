//! OCCT's single-edge `RemoveLoop` curve reconstruction for planar support surfaces.
use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_analysis::{ShapeAnalysisTransferParametersProj, TransferParameters};
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{
    ncollection::{Array1, HSequence},
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d, geom2d::Geom2dCurve, geom2d_adaptor::make_curve,
};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    geom::{
        GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomPlane, GeomSurface, GeomTrimmedCurve,
    },
    geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface},
};
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkgeombase::{
    geom_convert::CompCurveToBSplineCurve, geom_lib::to_3d, proj_lib::ProjectedCurve,
};
use rustycad_tkmath::{
    convert::ParameterisationType,
    geom_abs::CurveType,
    gp::{Axis2, Axis3, Point3, StandardDirection3},
};

struct LoopCurves {
    curve_2d: Handle<dyn Geom2dCurve>,
    curve_3d: Option<Handle<dyn GeomCurve>>,
    first: f64,
    last: f64,
}

type PlaneSupport = Option<(Handle<dyn GeomSurface>, Axis3)>;

impl ShapeFixWire {
    pub(super) fn try_remove_loop(
        &self,
        edge: &Edge,
        face: &Face,
        intersection: &IntersectionPoint,
        tolerance_factor: f64,
        precision: f64,
        remove_3d_loop: bool,
    ) -> Result<bool, Failure> {
        if BRepTool::is_closed_on_face(edge, face)? {
            return Ok(false);
        }
        let Some(curves) = loop_curves(
            edge,
            face,
            intersection,
            tolerance_factor,
            precision,
            remove_3d_loop,
        )?
        else {
            return Ok(false);
        };
        self.finish_loop_repair(edge, face, curves, remove_3d_loop)
    }

    fn finish_loop_repair(
        &self,
        edge: &Edge,
        face: &Face,
        curves: LoopCurves,
        remove_3d_loop: bool,
    ) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        if remove_3d_loop {
            if let Some(curve_3d) = &curves.curve_3d {
                builder.update_edge_curve(edge, curve_3d, 0.0)?;
            }
            builder.update_edge_pcurve_on_face(edge, &curves.curve_2d, face, 0.0)?;
            builder.range_on_face(edge, face, curves.first, curves.last)?;
            builder.same_range(edge, false)?;
            ShapeFixEdge::new().fix_same_parameter_default(edge)?;
            self.or_last_fix_status(ShapeExtendStatus::Done4);
            return Ok(true);
        }
        if let Some(curve_3d) = &curves.curve_3d {
            builder.update_edge_pcurve_on_face(edge, &curves.curve_2d, face, 0.0)?;
            builder.range_on_face(edge, face, curves.first, curves.last)?;
            builder.update_edge_curve(edge, curve_3d, 0.0)?;
        }
        let Some(fixed) = self.try_new_pcurve(
            edge,
            face,
            curves.curve_2d.clone(),
            curves.first,
            curves.last,
        )?
        else {
            return Ok(false);
        };
        if fixed.tolerance > self.max_tolerance().max(self.precision()) {
            return Ok(false);
        }
        builder.update_edge_pcurve_on_face(edge, &fixed.curve, face, 0.0)?;
        builder.range_on_face(edge, face, fixed.first, fixed.last)?;
        let analysis = ShapeAnalysisEdge::new();
        builder.update_vertex_tolerance(&analysis.first_vertex(edge), fixed.tolerance)?;
        builder.update_vertex_tolerance(&analysis.last_vertex(edge), fixed.tolerance)?;
        self.or_last_fix_status(ShapeExtendStatus::Done4);
        Ok(true)
    }
}

fn loop_curves(
    edge: &Edge,
    face: &Face,
    intersection: &IntersectionPoint,
    tolerance_factor: f64,
    precision: f64,
    remove_3d_loop: bool,
) -> Result<Option<LoopCurves>, Failure> {
    let (source_2d, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let (source_3d, first_3d, last_3d) = BRepTool::curve(edge)?;
    if source_2d.is_null() || source_3d.is_null() {
        return Ok(None);
    }
    let mut range = LoopRange::new(first, last, first_3d, last_3d, intersection)?;
    if range
        .adjusted(&source_3d, tolerance_factor, precision)?
        .is_none()
    {
        return Ok(None);
    }
    if remove_3d_loop {
        return remove_true_3d_loop(edge, face, &source_2d, &source_3d, range);
    }
    remove_projected_loop(edge, face, &source_3d, &source_2d, range)
}

#[derive(Clone, Copy)]
struct LoopRange {
    first: f64,
    last: f64,
    curve_first: f64,
    curve_last: f64,
    cut_first: f64,
    cut_last: f64,
    parameter_shift: f64,
}

impl LoopRange {
    fn new(
        first: f64,
        last: f64,
        curve_first: f64,
        curve_last: f64,
        intersection: &IntersectionPoint,
    ) -> Result<Self, Failure> {
        let a = intersection.param_on_first();
        let b = intersection.param_on_second();
        Ok(Self {
            first,
            last,
            curve_first,
            curve_last,
            cut_first: a.min(b),
            cut_last: a.max(b),
            parameter_shift: 0.0,
        })
    }

    fn adjusted(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        factor: f64,
        tolerance: f64,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let adaptor = GeomAdaptorCurve::with_range(curve, self.curve_first, self.curve_last)?;
        let shift = factor * adaptor.resolution(tolerance)?;
        self.parameter_shift = shift;
        let first = self.cut_first - shift;
        let last = self.cut_last + shift;
        if first <= self.first || last >= self.last {
            return Ok(None);
        }
        self.cut_first = first;
        self.cut_last = last;
        Ok(Some((first, last)))
    }
}

fn remove_projected_loop(
    _edge: &Edge,
    face: &Face,
    curve: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: LoopRange,
) -> Result<Option<LoopCurves>, Failure> {
    let plane = support_plane(face)?;
    let source = if plane.is_some() {
        curve.clone()
    } else {
        to_3d(
            Axis2::with_standard(Point3::default(), StandardDirection3::Z),
            pcurve,
        )?
    };
    let spline = concatenate_3d_loop(
        &source,
        range.first,
        range.last,
        range.cut_first,
        range.cut_last,
    )?;
    let result_2d = project_to_plane(&spline, &plane)?;
    let (first, last) = (range.first, range.last);
    Ok(Some(LoopCurves {
        curve_2d: result_2d,
        curve_3d: plane.map(|_| spline),
        first,
        last,
    }))
}

fn remove_true_3d_loop(
    edge: &Edge,
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    curve: &Handle<dyn GeomCurve>,
    range: LoopRange,
) -> Result<Option<LoopCurves>, Failure> {
    let plane = support_plane(face)?;
    let source = loop_source_curve(pcurve, curve, &plane)?;
    let pcurve_3d = concatenate_3d_loop(
        &source,
        range.first,
        range.last,
        range.cut_first,
        range.cut_last,
    )?;
    let new_pcurve = project_to_plane(&pcurve_3d, &plane)?;
    let (first_3d, last_3d, middle_3d) =
        transfer_loop_parameters(edge, face, range.cut_first, range.cut_last)?;
    let crossing = loop_midpoint(face, pcurve, range.cut_first, range.cut_last)?;
    let deviation = loop_deviations(crossing, curve, (first_3d, last_3d, middle_3d))?;
    let replacement_3d = if deviation.middle > deviation.first.max(deviation.last) {
        Some(reconstruct_3d_without_loop(
            curve,
            range,
            first_3d - range.parameter_shift,
            last_3d + range.parameter_shift,
        )?)
    } else {
        None
    };
    Ok(Some(LoopCurves {
        curve_2d: new_pcurve,
        curve_3d: replacement_3d,
        first: range.first,
        last: range.last,
    }))
}

fn loop_source_curve(
    pcurve: &Handle<dyn Geom2dCurve>,
    curve: &Handle<dyn GeomCurve>,
    plane: &Option<(Handle<dyn GeomSurface>, Axis3)>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if plane.is_some() {
        return Ok(curve.clone());
    }
    to_3d(
        Axis2::with_standard(Point3::default(), StandardDirection3::Z),
        pcurve,
    )
}

fn transfer_loop_parameters(
    edge: &Edge,
    face: &Face,
    first: f64,
    last: f64,
) -> Result<(f64, f64, f64), Failure> {
    let tool = ShapeAnalysisTransferParametersProj::from_edge(edge, face)?;
    let parameters = Handle::new(HSequence::default());
    parameters.append(first);
    parameters.append(last);
    parameters.append((first + last) * 0.5);
    let transferred = tool.perform(&parameters, false)?;
    Ok((
        sequence_value(&transferred, 1)?,
        sequence_value(&transferred, 2)?,
        sequence_value(&transferred, 3)?,
    ))
}

fn sequence_value(sequence: &Handle<HSequence<f64>>, index: usize) -> Result<f64, Failure> {
    let values = sequence
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire RemoveLoop lost transferred parameters"))?;
    values.sequence(|items| items.value(index).copied())
}

fn loop_midpoint(
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Point3, Failure> {
    let surface = BRepTool::surface(face)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let start = pcurve.value(first)?;
    let end = pcurve.value(last)?;
    let first_point = adaptor.value(start.x(), start.y())?;
    let last_point = adaptor.value(end.x(), end.y())?;
    Ok(average_points(first_point, last_point))
}

struct LoopDeviations {
    first: f64,
    last: f64,
    middle: f64,
}

fn loop_deviations(
    midpoint: Point3,
    curve: &Handle<dyn GeomCurve>,
    parameters: (f64, f64, f64),
) -> Result<LoopDeviations, Failure> {
    Ok(LoopDeviations {
        first: midpoint.distance(&curve.value(parameters.0)?),
        last: midpoint.distance(&curve.value(parameters.1)?),
        middle: midpoint.distance(&curve.value(parameters.2)?),
    })
}

fn reconstruct_3d_without_loop(
    curve: &Handle<dyn GeomCurve>,
    range: LoopRange,
    cut_first: f64,
    cut_last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    concatenate_3d_loop(
        curve,
        range.curve_first,
        range.curve_last,
        cut_first,
        cut_last,
    )
}

fn average_points(first: Point3, last: Point3) -> Point3 {
    Point3::new(
        (first.x() + last.x()) * 0.5,
        (first.y() + last.y()) * 0.5,
        (first.z() + last.z()) * 0.5,
    )
}

fn concatenate_3d_loop(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    cut_first: f64,
    cut_last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let first = Handle::new(GeomTrimmedCurve::new(curve, first, cut_first, true, true)?).cast();
    let mut composite =
        CompCurveToBSplineCurve::with_curve(&first, ParameterisationType::RationalC1)?;
    let patch = linear_patch_3d(
        curve.value(cut_first)?,
        curve.value(cut_last)?,
        cut_first,
        cut_last,
    )?;
    let last = Handle::new(GeomTrimmedCurve::new(curve, cut_last, last, true, true)?).cast();
    if !composite.add(&patch, precision::p_confusion(), true, false, 0)?
        || !composite.add(&last, precision::p_confusion(), true, false, 0)?
    {
        return Err(Failure::new(
            "ShapeFix_Wire RemoveLoop could not join trimmed 3D segments",
        ));
    }
    Ok(composite.bspline_curve().cast())
}

fn linear_patch_3d(
    first: Point3,
    last: Point3,
    first_parameter: f64,
    last_parameter: f64,
) -> Result<Handle<dyn GeomBoundedCurve>, Failure> {
    let pole_values = [first, last];
    let knot_values = [first_parameter, last_parameter];
    let multiplicities = [2, 2];
    let poles = Array1::from_shared(&pole_values, 1);
    let knots = Array1::from_shared(&knot_values, 1);
    let mults = Array1::from_shared(&multiplicities, 1);
    Ok(Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast())
}

fn support_plane(face: &Face) -> Result<PlaneSupport, Failure> {
    let (surface, _location) = BRepTool::surface_with_location(face)?;
    let plane_handle = surface.cast::<GeomPlane>();
    let Some(plane) = plane_handle.get() else {
        return Ok(None);
    };
    let position = plane.pln().position();
    Ok(Some((surface, position)))
}

fn project_to_plane(
    curve: &Handle<dyn GeomCurve>,
    plane: &Option<(Handle<dyn GeomSurface>, Axis3)>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let support = plane.as_ref().map_or_else(
        || Handle::new(GeomPlane::new(Axis3::default())).cast(),
        |(surface, _)| surface.clone(),
    );
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&support)?).cast();
    let spatial: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(curve)?).cast();
    let projected = ProjectedCurve::with_curve(&surface, &spatial)?;
    if matches!(
        projected.get_type()?,
        CurveType::OffsetCurve | CurveType::OtherCurve
    ) {
        return Err(Failure::new(
            "ShapeFix_Wire RemoveLoop projection is not a standard curve",
        ));
    }
    make_curve(&projected)
}
